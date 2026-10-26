# zEXEz v0.1.0 · Sprint 02 Hotfix 02

**Purpose:** Correct Wine DPI reporting for existing prefixes and verify DPI persistence for newly created prefixes. This is a parser/readback correction, not a Wine upgrade or new installer implementation.

## Evidence from Ubuntu

- Prefix `test-sprint02`: `dpi show` reported 120 DPI and Notepad was usable.
- Prefix `test-hotfix01`: `prefix create` returned ready and requested 120 DPI.
- `user.reg` contains **two** `"LogPixels"=dword:00000078` entries (0x78 = 120).
- `system.reg` contains `"LogPixels"=dword:00000060` (0x60 = 96); this system default must not override the user preference.
- Previous `dpi show test-hotfix01` did not detect the stored setting.

## Changes

- Read the user preference in `user.reg` using section normalization and support for `Control Panel\Desktop` and `Software\Wine\Fonts`.
- For unusual registry encodings, use a consensus-only fallback if all found `LogPixels` values agree; do not infer a value if they conflict. Never substitute a `system.reg` default for a user's DPI.
- After writing DPI with `wine reg add`, wait up to five seconds to confirm the requested value appears in `user.reg`. Otherwise return a warning instead of an unsupported success claim.
- Add 3 regression tests around duplicate values, precedence, and invalid/conflicting values.
- Leave already initialized prefixes, `README.md`, `LICENSE`, `docs/PRD.md`, `Cargo.lock`, and `~/.wine` alone.

## Acceptance test on Ubuntu

```bash
cd "$HOME/ZDATA/ME/GITHUB/zEXEz" || exit 1
bash scripts/verify-sprint02-hotfix02.sh
cargo run -- dpi show test-sprint02
cargo run -- dpi show test-hotfix01
cargo run -- prefix verify test-hotfix01
cargo run
```

Expect **120 DPI (125%)** from both prefixes; they are read-only checks. GUI should still open. No need to run `winecfg` again.

If values are not detected, inspect the actual registry section labels around `LogPixels` without editing user.reg:

```bash
awk '/^\[/{section=$0} /"LogPixels"=dword:/{print section; print $0}' "$HOME/.local/share/zexez/prefixes/test-hotfix01/user.reg"
```

Only after these tests pass, create the separate `yaskawa-dps10` prefix for controlled installation tests. Keep the Yaskawa x86 setup files together and do not connect to production equipment.

## Limitations

- Rust compilation is verified on Ubuntu by the included script; successful packaging checks elsewhere are not a substitute.
- Prefix structural readiness and Windows Notepad functionality do not guarantee Yaskawa installer success.
- Wine is not a sandbox; do not run untrusted Windows executables.
- Default DPI remains 120 for new prefixes; existing DPI is preserved.
