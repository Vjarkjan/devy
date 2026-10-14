//! Único acceso Rust al SDK. No se compila FFI en host.
use crate::{
    input::InputSample,
    video::{Color, DsScreen},
};
use core::{ffi::CStr, marker::PhantomData};

pub(crate) struct VideoBackend {
    // Impide mover el dispositivo a otro hilo incluso si un consumidor
    // activa multithreading en alguna dependencia Bevy del host.
    _main_thread: PhantomData<*mut ()>,
    #[cfg(target_vendor = "nintendo")]
    top: *mut u16,
    #[cfg(target_vendor = "nintendo")]
    bottom: *mut u16,
}

impl VideoBackend {
    pub(crate) fn take() -> Option<Self> {
        #[cfg(target_vendor = "nintendo")]
        // SAFETY: el shim concede un único propietario para toda la ejecución.
        unsafe {
            if ffi::devy_video_init() == 0 {
                return None;
            }
            Some(Self {
                _main_thread: PhantomData,
                top: ffi::devy_framebuffer(0),
                bottom: ffi::devy_framebuffer(1),
            })
        }
        #[cfg(not(target_vendor = "nintendo"))]
        Some(Self {
            _main_thread: PhantomData,
        })
    }

    pub(crate) fn row(
        &mut self,
        screen: DsScreen,
        y: usize,
        x: usize,
        pixels: &[u8],
        palette: &[Color; 256],
    ) {
        debug_assert!(y < 192 && x + pixels.len() <= 256);
        #[cfg(target_vendor = "nintendo")]
        // SAFETY: los límites proceden de IndexedFrame, los bancos son
        // exclusivos y todas las escrituras de VRAM tienen 16 o 32 bits.
        unsafe {
            match screen {
                DsScreen::Top => {
                    crate::video::copy::rgb555(self.top.add(y * 256 + x), pixels, palette)
                }
                DsScreen::Bottom => {
                    debug_assert!(x.is_multiple_of(2));
                    crate::video::copy::indexed(self.bottom.add((y * 256 + x) / 2), pixels);
                }
            }
        }
        #[cfg(not(target_vendor = "nintendo"))]
        let _ = (screen, y, x, pixels, palette);
    }

    pub(crate) fn palette(&mut self, palette: &[Color; 256]) {
        #[cfg(target_vendor = "nintendo")]
        for (index, color) in palette.iter().enumerate() {
            unsafe { ffi::devy_palette(index as u32, color.bits()) };
        }
        #[cfg(not(target_vendor = "nintendo"))]
        let _ = palette;
    }
    pub(crate) fn brightness(&mut self, top: i8, bottom: i8) {
        #[cfg(target_vendor = "nintendo")]
        unsafe {
            ffi::devy_brightness(
                i32::from(top.clamp(-16, 16)),
                i32::from(bottom.clamp(-16, 16)),
            )
        };
        #[cfg(not(target_vendor = "nintendo"))]
        let _ = (top, bottom);
    }
    pub(crate) fn show(&mut self) {
        #[cfg(target_vendor = "nintendo")]
        unsafe {
            ffi::devy_video_show()
        };
    }
    pub(crate) fn show_console(&mut self, visible: bool) {
        #[cfg(target_vendor = "nintendo")]
        unsafe {
            ffi::devy_console_visible(i32::from(visible))
        };
        #[cfg(not(target_vendor = "nintendo"))]
        let _ = visible;
    }
    pub(crate) fn write_console(&mut self, text: &CStr) {
        #[cfg(target_vendor = "nintendo")]
        unsafe {
            ffi::devy_console_write(text.as_ptr())
        };
        #[cfg(not(target_vendor = "nintendo"))]
        let _ = text;
    }
    pub(crate) fn clear_console(&mut self) {
        #[cfg(target_vendor = "nintendo")]
        unsafe {
            ffi::devy_console_clear()
        };
    }
}

pub(crate) fn read_input() -> Option<InputSample> {
    #[cfg(target_vendor = "nintendo")]
    {
        let (mut held, mut x, mut y) = (0, 0, 0);
        // SAFETY: punteros a escalares válidos del ABI C; el SDK hace un scan.
        unsafe { ffi::devy_input(&mut held, &mut x, &mut y) };
        Some(InputSample {
            held,
            touch: (held & (1 << 12) != 0).then_some((x, y)),
        })
    }
    #[cfg(not(target_vendor = "nintendo"))]
    None
}

pub(crate) fn ticks() -> Option<u64> {
    #[cfg(target_vendor = "nintendo")]
    {
        Some(unsafe { ffi::devy_ticks() })
    }
    #[cfg(not(target_vendor = "nintendo"))]
    None
}

pub(crate) fn outside_vblank() -> bool {
    #[cfg(target_vendor = "nintendo")]
    {
        unsafe { ffi::devy_outside_vblank() != 0 }
    }
    #[cfg(not(target_vendor = "nintendo"))]
    false
}

#[cfg(target_vendor = "nintendo")]
pub(crate) fn wait_vblank() {
    unsafe { ffi::devy_wait_vblank() };
}

#[cfg(target_vendor = "nintendo")]
pub(crate) mod ffi {
    use core::ffi::{c_char, c_int};
    unsafe extern "C" {
        pub fn devy_video_init() -> c_int;
        pub fn devy_framebuffer(screen: u32) -> *mut u16;
        pub fn devy_palette(index: u32, color: u16);
        pub fn devy_video_show();
        pub fn devy_brightness(top: c_int, bottom: c_int);
        pub fn devy_console_visible(visible: c_int);
        pub fn devy_console_write(text: *const c_char);
        pub fn devy_console_clear();
        pub fn devy_input(held: *mut u32, x: *mut u16, y: *mut u16);
        pub fn devy_ticks() -> u64;
        pub fn devy_outside_vblank() -> c_int;
        pub fn devy_wait_vblank();
        pub fn devy_enter_critical() -> c_int;
        pub fn devy_leave_critical(previous: c_int);
        #[cfg(feature = "allocator")]
        pub fn devy_alloc(align: usize, size: usize) -> *mut u8;
        #[cfg(feature = "allocator")]
        pub fn devy_free(ptr: *mut u8);
        #[cfg(feature = "panic-handler")]
        pub fn devy_panic();
        #[cfg(feature = "storage")]
        pub fn devy_storage_init() -> c_int;
        #[cfg(feature = "storage")]
        pub fn devy_storage_read(
            path: *const c_char,
            buffer: *mut u8,
            capacity: usize,
            count: *mut usize,
        ) -> c_int;
        #[cfg(feature = "storage")]
        pub fn devy_storage_write(path: *const c_char, buffer: *const u8, count: usize) -> c_int;
        #[cfg(feature = "storage")]
        pub fn devy_storage_mkdir(path: *const c_char) -> c_int;
    }
}
