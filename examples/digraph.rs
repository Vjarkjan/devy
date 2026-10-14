#![cfg_attr(target_vendor = "nintendo", no_std)]
#![cfg_attr(target_vendor = "nintendo", no_main)]
#![cfg_attr(not(target_vendor = "nintendo"), allow(unused_imports))]
extern crate alloc;
#[path = "digraph/mod.rs"]
mod digraph;

use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::IntoScheduleConfigs};
use devy::{HardwareVideoPlugin as VideoPlugin, InputPlugin};

use digraph::{
    analysis::{AnalysisState, refresh_analysis},
    bottom2d::BottomViewPlugin,
    editor::{EditorState, editor_input},
    graph::Digraph,
    stats::StatsPlugin,
    top2d::TopViewPlugin,
    ui::{GraphMenuState, UiState},
};

fn seed_graph(mut graph: ResMut<Digraph>) {
    if !graph.vertices.is_empty() {
        return;
    }
    let a = graph.add_vertex(38, 46).unwrap();
    let b = graph.add_vertex(108, 32).unwrap();
    let c = graph.add_vertex(174, 64).unwrap();
    let d = graph.add_vertex(196, 126).unwrap();
    let e = graph.add_vertex(112, 142).unwrap();
    graph.add_edge(a, b, true);
    graph.add_edge(b, c, true);
    graph.add_edge(c, a, true);
    graph.add_edge(c, d, true);
    graph.add_edge(d, e, true);
    graph.add_edge(e, c, true);
}
#[cfg(target_vendor = "nintendo")]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> core::ffi::c_int {
    let mut app = App::new();
    // Keep the boot path minimal: normal video + input only. Writable storage is
    // initialized lazily on the first SAVE/LOAD action via fatInitDefault().
    app.add_plugins(VideoPlugin)
        .add_plugins(InputPlugin)
        .init_resource::<Digraph>()
        .init_resource::<EditorState>()
        .init_resource::<AnalysisState>()
        .init_resource::<UiState>()
        .init_resource::<GraphMenuState>()
        .add_plugins(BottomViewPlugin)
        .add_plugins(TopViewPlugin)
        .add_plugins(StatsPlugin)
        .add_systems(Startup, seed_graph)
        .add_systems(Update, (editor_input, refresh_analysis).chain());
    devy::run(app)
}

#[cfg(not(target_vendor = "nintendo"))]
fn main() {
    let mut app = App::new();
    app.init_resource::<Digraph>()
        .add_systems(Startup, seed_graph);
    app.update();
    assert_eq!(app.world().resource::<Digraph>().vertices.len(), 5);
}
