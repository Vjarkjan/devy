//! Aritmética 20.12 para las rutas de cálculo de kts-nds.
//!
//! `Fx32` conserva doce bits fraccionarios. La división y la raíz utilizan
//! el coprocesador de DS; en host tienen una implementación por software.
//! Los vectores, matrices, selector radial y joystick táctil comparten ese
//! formato. Los resultados deben mantenerse dentro del rango representable;
//! el punto fijo no elimina el desbordamiento.

mod fx;
mod fx_vec;
pub mod hw;
pub mod mat;
pub mod radial;
pub mod stick;

pub use fx::{FRAC_BITS, Fx32, ONE_RAW};
pub use fx_vec::{FxVec2, FxVec3};
pub use mat::model_matrix;
pub use radial::{nearest_spoke, spoke_dir};
pub use stick::{StickConfig, stick_vector};
