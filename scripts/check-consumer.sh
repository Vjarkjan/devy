#!/usr/bin/env bash
set -euo pipefail
devy_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
case "${1:-}" in ''|--nds) ;; *) echo 'Uso: check-consumer.sh [--nds]' >&2; exit 2;; esac
devy_tmp="$(mktemp -d "${TMPDIR:-/tmp}/devy-consumer.XXXXXX")"
trap 'rm -rf -- "$devy_tmp"' EXIT
python3 - "$devy_root" "$devy_tmp" <<'PY'
import json
from pathlib import Path
import sys
root, package = map(Path, sys.argv[1:])
(package / 'src').mkdir()
(package / 'Cargo.toml').write_text('''[package]
name = "devy-consumer"
version = "0.1.0"
edition = "2024"
[dependencies]
devy = { path = ''' + json.dumps(str(root)) + ''' }
[profile.dev]
panic = "abort"
opt-level = 1
[profile.dev.package."*"]
opt-level = 3
[profile.release]
panic = "abort"
opt-level = "s"
lto = true
codegen-units = 1
''')
(package / 'src/main.rs').write_text('''#![cfg_attr(target_vendor = "nintendo", no_std)]
#![cfg_attr(target_vendor = "nintendo", no_main)]
use devy::prelude::*;

#[derive(Component)]
struct Value(u32);
#[derive(Resource, Default)]
struct Total(u32);
#[derive(Event)]
struct Tick;
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
struct Logic;

fn setup(mut commands: Commands) { commands.spawn(Value(7)); }
fn sum(values: Query<&Value>, mut total: ResMut<Total>, mut events: EventWriter<Tick>) {
    total.0 = values.iter().map(|v| v.0).sum();
    events.write(Tick);
}
fn app() -> App {
    let mut app = App::new();
    app.add_plugins(DsCorePlugins).init_resource::<Total>().add_event::<Tick>()
        .add_systems(Startup, setup).add_systems(Update, sum.in_set(Logic));
    app
}
#[cfg(not(target_vendor = "nintendo"))]
fn main() {
    let mut app = app();
    app.update();
    assert_eq!(app.world().resource::<Total>().0, 7);
    println!("Consumidor externo: derives y sistemas correctos");
}
#[cfg(target_vendor = "nintendo")]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> core::ffi::c_int { devy::run(app()) }
''')
PY
cargo +stable generate-lockfile --offline --manifest-path "$devy_tmp/Cargo.toml"
cargo +stable run --locked --offline --manifest-path "$devy_tmp/Cargo.toml" --target-dir "$devy_root/target"
if [[ "${1:-}" == --nds ]]; then
    bash "$devy_root/scripts/nds.sh" build --manifest-path "$devy_tmp/Cargo.toml" \
        --bin devy-consumer --offline --target-dir "$devy_root/target/nds"
fi
