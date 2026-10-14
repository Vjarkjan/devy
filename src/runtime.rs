//! Bucle ARM9, allocator y sección crítica. En host no se instala runtime.
use bevy_app::App;
use bevy_ecs::schedule::ScheduleLabel;

/// Transferencia de pantalla. El runner la ejecuta después de esperar VBlank.
#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Present;

/// Presenta un frame ya calculado; no espera VBlank. Útil en pruebas de host.
/// En DS, quien llame directamente debe sincronizar la presentación.
pub fn present(app: &mut App) {
    let _ = app.world_mut().try_run_schedule(Present);
}

/// Ejecuta la aplicación en DS. La lógica precede a la espera y a la copia.
#[cfg(target_vendor = "nintendo")]
pub fn run(mut app: App) -> ! {
    app.finish();
    app.cleanup();
    loop {
        app.update();
        crate::cothread::yield_until_vblank();
        present(&mut app);
    }
}

#[cfg(target_vendor = "nintendo")]
mod nds {
    use core::sync::atomic::{Ordering, compiler_fence};

    struct InterruptSection;
    critical_section::set_impl!(InterruptSection);

    // SAFETY: ARM9 es el único núcleo que accede al World. No se comparten
    // atómicos Rust con ARM7. Restaurar IME permite secciones anidadas.
    unsafe impl critical_section::Impl for InterruptSection {
        unsafe fn acquire() -> bool {
            let previous = unsafe { crate::platform::ffi::devy_enter_critical() };
            compiler_fence(Ordering::SeqCst);
            previous != 0
        }

        unsafe fn release(previous: bool) {
            compiler_fence(Ordering::SeqCst);
            unsafe { crate::platform::ffi::devy_leave_critical(i32::from(previous)) };
        }
    }

    #[cfg(feature = "allocator")]
    struct SdkAllocator;

    #[cfg(feature = "allocator")]
    // SAFETY: el CRT de BlocksDS inicializa el heap antes de main; memalign
    // respeta la alineación solicitada y free recibe el mismo puntero.
    unsafe impl core::alloc::GlobalAlloc for SdkAllocator {
        unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
            unsafe { crate::platform::ffi::devy_alloc(layout.align().max(8), layout.size()) }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, _: core::alloc::Layout) {
            unsafe { crate::platform::ffi::devy_free(ptr) };
        }
    }

    #[cfg(feature = "allocator")]
    #[global_allocator]
    static ALLOCATOR: SdkAllocator = SdkAllocator;

    #[cfg(feature = "panic-handler")]
    #[panic_handler]
    fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
        // No depende de que la consola de texto haya terminado su arranque.
        unsafe { crate::platform::ffi::devy_panic() };
        loop {
            crate::platform::wait_vblank();
        }
    }
}
