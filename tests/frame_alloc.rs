//! El dibujo conserva los buffers; no añade asignaciones después de crearlos.
use devy::prelude::*;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Counting;
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

#[test]
fn repeated_frame_edits_do_not_allocate() {
    let mut frame = IndexedFrame::default();
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for i in 0..100 {
        frame.set_color(1, Color::rgb8(i, 100, 200));
        frame.fill_rect(Rect::new(i as usize, 8, 16, 16).unwrap(), 1);
        frame.line((0, i as i32), (255, i as i32), 1).unwrap();
        frame
            .blit(Rect::new(3, 4, 2, 2).unwrap(), &[1, 2, 3, 4])
            .unwrap();
    }
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(before, after);
}
