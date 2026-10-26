#!/usr/bin/env bash
# Format, test, check; does NOT initialize Wine or execute Windows binaries.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$ROOT"

echo '=== zEXEz Sprint 02 Hotfix 01 ==='
rustc --version
cargo --version
pkg-config --modversion gtk4 libadwaita-1
wine --version

echo
echo '=== Rust formatting ==='
cargo fmt --all
cargo fmt --all -- --check

echo
echo '=== Rust unit tests ==='
cargo test --locked

echo
echo '=== Cargo compile check ==='
cargo check --locked

echo
echo '=== Non-executing CLI checks ==='
cargo run --quiet --locked -- --help
cargo run --quiet --locked -- diagnose
cargo run --quiet --locked -- prefix path test-sprint02

echo
echo 'PASS: no wineboot, winecfg, Windows installer, or prefix mutation performed by this script.'
