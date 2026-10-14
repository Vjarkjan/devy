#!/usr/bin/env bash
# Conversores de kts-nds; sólo se ejecutan en el ordenador.
set -euo pipefail
devy_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
export BLOCKSDS="${BLOCKSDS:-/opt/wonderful/thirdparty/blocksds/core}"
cd "$devy_root"
cargo +stable build --locked -p obj2dl -p png2sprite -p png2bg -p wav2bank -p scene2bin
python3 scripts/bake_assets.py
