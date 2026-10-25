# zEXEz — Sprint 01: Native Foundation

Status: **implementation candidate, awaiting validation on Ubuntu**.

## Scope

- Native Rust application window with GTK4 and Libadwaita.
- Read-only diagnostics for Wine, Winetricks, Wine 32-bit/64-bit package status,
  and `wineserver` location.
- Refresh button that re-runs those diagnostics.
- No EXE/MSI installation, application launching, prefix creation, USB/serial
  access, or Windows program compatibility is claimed in this sprint.

## Target environment

- Ubuntu 26.04.1 LTS, GNOME on Wayland
- Rust 1.99.1, GTK4 4.22.4, Libadwaita 1.9.1
- Wine 10.0 from Ubuntu repositories

## Compile and run

```bash
cd "$HOME/ZDATA/ME/GITHUB/zEXEz"
bash scripts/verify-sprint01.sh
cargo run
```

The verification script executes `cargo fmt --check`, `cargo test`, and
`cargo check`. Cargo downloads Rust crate dependencies on the first build.
The GTK4/Libadwaita development packages need to be installed in the system.

## Wine server note

On Ubuntu 26.04, Wine 10 no longer necessarily provides `/usr/bin/wineserver`.
The actual server is normally at
`/usr/lib/x86_64-linux-gnu/wine/wineserver` and Wine starts it automatically.
Do **not** add system symlinks just to make the `wineserver` command available.

## Design safeguards

- Never run Wine as root.
- Never modify the default Wine prefix `~/.wine`.
- Prefixes in future versions will live in a dedicated zEXEz data directory.
- Windows software and vendor USB drivers require separate validation;
  a successful installer does not prove industrial equipment connectivity.
- Application source is MIT-licensed; bundled Wine, Microsoft components,
  and third-party packages have their own terms and must be reviewed later.

## Next sprint

Sprint 02: PE executable architecture detection (x86/x64), isolated per-app
prefixes, Wine process invocation, and non-destructive logs. Build runtime
features incrementally and validate them against the actual vendor installers.

## Hotfix 01 — ToolbarView compile error

`adw::ToolbarView` requires the libadwaita Rust binding feature `v1_4`.
This release enables that feature in `Cargo.toml` while keeping
`libadwaita = 0.9.2` and `gtk4 = 0.11.5` pinned. The target system
provides libadwaita 1.9.1, so no Ubuntu package change is necessary.

Build verification on Ubuntu:

```bash
bash scripts/verify-sprint01.sh
cargo run
```

This hotfix does not install EXE/MSI software or initialize Wine prefixes.
