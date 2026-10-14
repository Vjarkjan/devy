//! Aplicación de referencia importada del árbol local kts-nds.
#![allow(dead_code)]
// El ejemplo conserva los índices compartidos de los algoritmos y los
// parámetros explícitos de los sistemas y renderizadores del editor.
#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::needless_range_loop
)]
#![cfg_attr(
    not(target_vendor = "nintendo"),
    allow(unused_imports, unused_variables)
)]
pub mod analysis;
pub mod bottom2d;
pub mod dag;
pub mod editor;
pub mod graph;
pub mod project;
pub mod stats;
pub mod top2d;
pub mod ui;
