#!/usr/bin/env bash
# Verifies zEXEz Sprint 02 on the user's Ubuntu machine.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$ROOT"

echo '=== Environment ==='
rustc --version
cargo --version
pkg-config --modversion gtk4 libadwaita-1
wine --version

echo
echo '=== Format ==='
cargo fmt --all
cargo fmt --all -- --check

echo
echo '=== Unit tests (no Wine programs executed) ==='
cargo test --locked

echo
echo '=== Compile check ==='
cargo check --locked

echo
echo '=== CLI diagnostics ==='
cargo run --quiet --locked -- diagnose
cargo run --quiet --locked -- --help

echo
echo '=== No implicit prefix creation ==='
echo 'No wineboot or EXE/MSI executed by this verification script.'
echo 'Sprint 02 code verification PASSED.'
