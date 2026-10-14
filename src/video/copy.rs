//! Escrituras legales de VRAM, comprobadas también con memoria del host.
use super::Color;
use core::ptr::write_volatile;

/// # Safety
/// dst debe estar alineado a 2 y ser exclusivo y válido para pixels.len()
/// halfwords. Origen y destino no se solapan. VRAM permite palabras de 32 bits.
pub(crate) unsafe fn rgb555(dst: *mut u16, pixels: &[u8], palette: &[Color; 256]) {
    let mut i = 0;
    if !dst.cast::<u32>().is_aligned() && !pixels.is_empty() {
        unsafe { write_volatile(dst, palette[pixels[0] as usize].bits() | 0x8000) };
        i = 1;
    }
    while i + 1 < pixels.len() {
        let lo = u32::from(palette[pixels[i] as usize].bits() | 0x8000);
        let hi = u32::from(palette[pixels[i + 1] as usize].bits() | 0x8000);
        // SAFETY: el prólogo alineó el destino; aún quedan dos píxeles.
        unsafe { write_volatile(dst.add(i).cast::<u32>(), lo | (hi << 16)) };
        i += 2;
    }
    if i < pixels.len() {
        unsafe { write_volatile(dst.add(i), palette[pixels[i] as usize].bits() | 0x8000) };
    }
}

/// # Safety
/// dst debe estar alineado a 2 y ser exclusivo y válido para pixels.len()/2
/// halfwords. La longitud debe ser par; origen y destino no se solapan.
pub(crate) unsafe fn indexed(dst: *mut u16, pixels: &[u8]) {
    assert!(pixels.len().is_multiple_of(2));
    for (i, pair) in pixels.as_chunks::<2>().0.iter().enumerate() {
        unsafe { write_volatile(dst.add(i), u16::from_le_bytes([pair[0], pair[1]])) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn packed_rgb_matches_scalar_reference_for_every_width_and_alignment() {
        let palette =
            core::array::from_fn(|i| Color::rgb8(i as u8, (i ^ 0x55) as u8, (255 - i) as u8));
        let source: alloc::vec::Vec<u8> = (0..256).map(|i| (i * 37) as u8).collect();
        for width in 0..=256 {
            for offset in [4, 5] {
                let mut output = vec![0xdead_u16; 268];
                unsafe { rgb555(output.as_mut_ptr().add(offset), &source[..width], &palette) };
                for (i, &index) in source[..width].iter().enumerate() {
                    assert_eq!(output[offset + i], palette[index as usize].bits() | 0x8000);
                }
                assert!(output[..offset].iter().all(|v| *v == 0xdead));
                assert!(output[offset + width..].iter().all(|v| *v == 0xdead));
            }
        }
    }

    #[test]
    fn indexed_copy_preserves_guards_and_pixel_order() {
        let source: alloc::vec::Vec<u8> = (0..256).map(|i| (i * 37) as u8).collect();
        for width in (0..=256).step_by(2) {
            let mut output = vec![0xdead_u16; 132];
            unsafe { indexed(output.as_mut_ptr().add(2), &source[..width]) };
            let bytes: alloc::vec::Vec<u8> = output[2..2 + width / 2]
                .iter()
                .flat_map(|p| p.to_le_bytes())
                .collect();
            assert_eq!(bytes, source[..width]);
            assert_eq!(&output[..2], &[0xdead; 2]);
            assert!(output[2 + width / 2..].iter().all(|v| *v == 0xdead));
        }
    }
}
