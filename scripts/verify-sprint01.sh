#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo '=== zEXEz Sprint 01: Environment ==='
command -v cargo >/dev/null || { echo 'ERROR: cargo is missing'; exit 1; }
command -v rustc >/dev/null || { echo 'ERROR: rustc is missing'; exit 1; }
command -v pkg-config >/dev/null || { echo 'ERROR: pkg-config is missing'; exit 1; }

rustc --version
cargo --version
pkg-config --modversion gtk4
pkg-config --modversion libadwaita-1

echo
echo '=== Wine read-only diagnostics ==='
if command -v wine >/dev/null 2>&1; then
    wine --version
else
    echo 'Wine not installed (GUI will report unavailable)'
fi

for path in \
    /usr/lib/x86_64-linux-gnu/wine/wineserver \
    /usr/lib/i386-linux-gnu/wine/wineserver; do
    if [[ -f "$path" ]]; then
        printf 'Wine server: %s\n' "$path"
    fi
done

echo
echo '=== Rust format ==='
cargo fmt --all -- --check

echo
echo '=== Rust tests ==='
cargo test

echo
echo '=== Rust compile check ==='
cargo check

echo
echo 'Sprint 01 verification completed successfully.'
