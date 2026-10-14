//! Framebuffer indexado y seguimiento de cambios, sin acceso a hardware.
use alloc::{boxed::Box, vec};

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 192;
pub const PIXELS: usize = WIDTH * HEIGHT;

/// Color de 15 bits. El bit de opacidad lo añade el presentador superior.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Color(u16);

impl Color {
    pub const BLACK: Self = Self(0);
    pub const WHITE: Self = Self(0x7fff);
    pub const fn rgb5(r: u8, g: u8, b: u8) -> Self {
        Self((r as u16 & 31) | ((g as u16 & 31) << 5) | ((b as u16 & 31) << 10))
    }
    pub const fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgb5(r >> 3, g >> 3, b >> 3)
    }
    pub const fn bits(self) -> u16 {
        self.0
    }
    pub const fn to_rgb8(self) -> [u8; 3] {
        const fn expand(v: u16) -> u8 {
            ((v << 3) | (v >> 2)) as u8
        }
        [
            expand(self.0 & 31),
            expand((self.0 >> 5) & 31),
            expand((self.0 >> 10) & 31),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawError {
    OutsideScreen,
    WrongLength,
}

/// Rectángulo contenido en 256 × 192. Se permiten regiones vacías.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Result<Self, DrawError> {
        if x > WIDTH || y > HEIGHT || width > WIDTH - x || height > HEIGHT - y {
            return Err(DrawError::OutsideScreen);
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }
    pub const fn x(self) -> usize {
        self.x
    }
    pub const fn y(self) -> usize {
        self.y
    }
    pub const fn width(self) -> usize {
        self.width
    }
    pub const fn height(self) -> usize {
        self.height
    }
    pub const fn len(self) -> usize {
        self.width * self.height
    }
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[derive(Clone, Copy)]
pub(super) struct Span {
    pub start: u16,
    pub end: u16,
}

impl Span {
    const EMPTY: Self = Self {
        start: WIDTH as u16,
        end: 0,
    };
    const FULL: Self = Self {
        start: 0,
        end: WIDTH as u16,
    };
}

/// 48 KiB de índices y una paleta de 256 colores por pantalla.
/// Cada fila conserva un único intervalo pendiente de transferencia.
pub struct IndexedFrame {
    pixels: Box<[u8]>,
    palette: [Color; 256],
    pub(super) dirty: [Span; HEIGHT],
    pub(super) palette_dirty: bool,
}

impl Default for IndexedFrame {
    fn default() -> Self {
        Self {
            pixels: vec![0; PIXELS].into_boxed_slice(),
            palette: [Color::BLACK; 256],
            dirty: [Span::FULL; HEIGHT],
            palette_dirty: true,
        }
    }
}

impl IndexedFrame {
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    /// Acceso para renderizadores existentes. Marca la pantalla completa.
    pub fn pixels_mut(&mut self) -> &mut [u8] {
        self.invalidate();
        &mut self.pixels
    }
    pub fn palette(&self) -> &[Color; 256] {
        &self.palette
    }
    pub fn set_color(&mut self, index: u8, color: Color) {
        if self.palette[index as usize] != color {
            self.palette[index as usize] = color;
            self.palette_dirty = true;
        }
    }
    /// Devuelve false si el punto queda fuera o ya tenía ese índice.
    pub fn plot(&mut self, x: i32, y: i32, color: u8) -> bool {
        if !(0..WIDTH as i32).contains(&x) || !(0..HEIGHT as i32).contains(&y) {
            return false;
        }
        let (x, y) = (x as usize, y as usize);
        let pixel = &mut self.pixels[y * WIDTH + x];
        if *pixel == color {
            return false;
        }
        *pixel = color;
        self.dirty[y].start = self.dirty[y].start.min(x as u16);
        self.dirty[y].end = self.dirty[y].end.max(x as u16 + 1);
        true
    }
    pub fn fill(&mut self, color: u8) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.plot(x as i32, y as i32, color);
            }
        }
    }
    pub fn fill_rect(&mut self, rect: Rect, color: u8) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.plot(x as i32, y as i32, color);
            }
        }
    }
    /// Copia índices consecutivos; valida toda la longitud antes de modificar.
    pub fn blit(&mut self, rect: Rect, pixels: &[u8]) -> Result<(), DrawError> {
        if pixels.len() != rect.len() {
            return Err(DrawError::WrongLength);
        }
        if rect.is_empty() {
            return Ok(());
        }
        for (y, row) in pixels.chunks_exact(rect.width).enumerate() {
            for (x, &color) in row.iter().enumerate() {
                self.plot((rect.x + x) as i32, (rect.y + y) as i32, color);
            }
        }
        Ok(())
    }
    /// Segmento entre dos puntos dentro de la pantalla, sin asignaciones.
    pub fn line(&mut self, from: (i32, i32), to: (i32, i32), color: u8) -> Result<(), DrawError> {
        for (x, y) in [from, to] {
            if !(0..WIDTH as i32).contains(&x) || !(0..HEIGHT as i32).contains(&y) {
                return Err(DrawError::OutsideScreen);
            }
        }
        let (mut x, mut y) = from;
        let dx = (to.0 - x).abs();
        let dy = -(to.1 - y).abs();
        let sx = if x < to.0 { 1 } else { -1 };
        let sy = if y < to.1 { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            self.plot(x, y, color);
            if (x, y) == to {
                break;
            }
            let twice = 2 * error;
            if twice >= dy {
                error += dy;
                x += sx;
            }
            if twice <= dx {
                error += dx;
                y += sy;
            }
        }
        Ok(())
    }
    pub fn invalidate(&mut self) {
        self.dirty.fill(Span::FULL);
    }
    pub(super) fn mark_presented(&mut self) {
        self.dirty.fill(Span::EMPTY);
        self.palette_dirty = false;
    }
}
