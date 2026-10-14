//! FAT/SD explícito y bloqueante. No se monta durante el arranque del plugin.
use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use core::ffi::CStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageError {
    Unavailable,
    Io,
    BufferTooSmall,
    Unsupported,
}

/// No almacena archivos abiertos ni búferes internos de Rust.
#[derive(Resource, Default)]
pub struct Storage {
    mounted: bool,
}

impl Storage {
    pub fn mount(&mut self) -> Result<(), StorageError> {
        if self.mounted {
            return Ok(());
        }
        #[cfg(target_vendor = "nintendo")]
        {
            status(unsafe { crate::platform::ffi::devy_storage_init() })?;
            self.mounted = true;
            Ok(())
        }
        #[cfg(not(target_vendor = "nintendo"))]
        Err(StorageError::Unsupported)
    }
    /// Si no cabe, devuelve BufferTooSmall antes de modificar el buffer.
    pub fn read_into(&mut self, path: &CStr, buffer: &mut [u8]) -> Result<usize, StorageError> {
        self.mount()?;
        #[cfg(target_vendor = "nintendo")]
        {
            let mut count = 0;
            status(unsafe {
                crate::platform::ffi::devy_storage_read(
                    path.as_ptr(),
                    buffer.as_mut_ptr(),
                    buffer.len(),
                    &mut count,
                )
            })?;
            Ok(count)
        }
        #[cfg(not(target_vendor = "nintendo"))]
        {
            let _ = (path, buffer);
            Err(StorageError::Unsupported)
        }
    }
    /// Reemplaza el archivo. La operación no es atómica ante pérdida de energía.
    pub fn write(&mut self, path: &CStr, bytes: &[u8]) -> Result<(), StorageError> {
        self.mount()?;
        #[cfg(target_vendor = "nintendo")]
        {
            status(unsafe {
                crate::platform::ffi::devy_storage_write(path.as_ptr(), bytes.as_ptr(), bytes.len())
            })
        }
        #[cfg(not(target_vendor = "nintendo"))]
        {
            let _ = (path, bytes);
            Err(StorageError::Unsupported)
        }
    }
    /// Crea un directorio; los padres deben existir.
    pub fn create_dir(&mut self, path: &CStr) -> Result<(), StorageError> {
        self.mount()?;
        #[cfg(target_vendor = "nintendo")]
        {
            status(unsafe { crate::platform::ffi::devy_storage_mkdir(path.as_ptr()) })
        }
        #[cfg(not(target_vendor = "nintendo"))]
        {
            let _ = path;
            Err(StorageError::Unsupported)
        }
    }
}

#[cfg(target_vendor = "nintendo")]
fn status(code: i32) -> Result<(), StorageError> {
    match code {
        0 => Ok(()),
        -1 => Err(StorageError::Unavailable),
        -3 => Err(StorageError::BufferTooSmall),
        _ => Err(StorageError::Io),
    }
}

pub struct StoragePlugin;
impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Storage>();
    }
}
