#!/usr/bin/env bash
# Funciona también desde otro paquete: --manifest-path /ruta/Cargo.toml --bin juego
set -euo pipefail
devy_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
action="${1:-rom}"
if (($#)); then shift; fi
case "$action" in check|build|clippy|rom) ;; *) echo 'Uso: nds.sh {check|build|clippy|rom} [argumentos Cargo]' >&2; exit 2;; esac
export BLOCKSDS="${BLOCKSDS:-/opt/wonderful/thirdparty/blocksds/core}"
export WONDERFUL_TOOLCHAIN="${WONDERFUL_TOOLCHAIN:-/opt/wonderful}"
export CARGO_TARGET_ARMV5TE_NINTENDO_DS_LINKER="$devy_root/scripts/nds-linker.sh"
test -f "$BLOCKSDS/libs/libnds/lib/libnds9.a"
cargo_action="$action"
[[ "$action" != rom ]] || cargo_action=build
args=("$@")
if ((${#args[@]} == 0)); then args=(--manifest-path "$devy_root/Cargo.toml" --example hello); fi
extra=()
[[ "$action" != clippy ]] || extra=(-- -D warnings)
if [[ "$action" == rom ]]; then
    mkdir -p "$devy_root/output"
    manifest_log="$(mktemp)"
    trap 'rm -f "$manifest_log"' EXIT
    cargo "+${NDS_TOOLCHAIN:-nightly}" "$cargo_action" --locked --release \
        --config "$devy_root/.cargo/nds.toml" --target "$devy_root/armv5te-nintendo-ds.json" \
        --message-format=json-render-diagnostics "${args[@]}" > "$manifest_log"
    # Usar el artefacto informado por Cargo evita adivinar CARGO_TARGET_DIR.
    python3 "$devy_root/scripts/package_rom.py" "$manifest_log" "$devy_root/output"
else
    cargo "+${NDS_TOOLCHAIN:-nightly}" "$cargo_action" --locked --release \
        --config "$devy_root/.cargo/nds.toml" --target "$devy_root/armv5te-nintendo-ds.json" \
        "${args[@]}" "${extra[@]}"
fi
