# zEXEz v0.1.0 · Sprint 02 Hotfix 01

**Status:** Source candidate; validate on Ubuntu 26.04.1 LTS before committing.

## Problem observed

`zexez prefix create test-sprint02` returned **“Wine prefix initialization did not complete (exit status: 0)”**, even though `drive_c/`, `system.reg`, `user.reg`, and `userdef.reg` subsequently existed. Notepad opened successfully at 120 DPI, and the 32-bit SysWOW64 Notepad command returned exit code 0 (with a WoW64 diagnostic warning).

This proves a structurally usable test prefix and a successful Notepad smoke test, **not** compatibility with Yaskawa installers or device drivers. Wine's `err:ole`, `err:setupapi`, and experimental WoW64 logs may still need app-specific investigation.

## Hotfix scope

- Retry a structural readiness check for up to 20 seconds after `wineboot` exits (handles late registry/file creation).
- A ready prefix is never reinitialized by `prefix create` or `prefix repair`.
- For incomplete existing prefixes, `prefix create` / `prefix repair` uses `wineboot -u`, not deletion/recreation. Both are explicit actions.
- New prefixes receive default **120 DPI** using the prefix-local `HKCU\\Control Panel\\Desktop\\LogPixels` registry setting. Existing ready prefixes keep their own scaling.
- New commands: `prefix verify`, `prefix repair`, `dpi show`, `dpi set`.
- GUI has a DPI selector and Apply DPI button for selected, initialized prefixes.
- DPI failures on a new prefix are reported as warnings, not as a false prefix failure. Wine logs remain under `~/.local/state/zexez/logs/`.
- Adds pure Rust tests for prefix structure and DPI registry parsing.

DPI changes take effect after Windows applications restart. The registry reader may report “not explicitly detected” if the value is absent or has not been flushed yet; `winecfg` can confirm the effective setting.

## Commands to verify (safe; no Wine prefix modified)

```bash
cd "$HOME/ZDATA/ME/GITHUB/zEXEz" || exit 1
bash scripts/verify-sprint02-hotfix01.sh
cargo run -- prefix verify test-sprint02
cargo run -- dpi show test-sprint02
```

## New test prefix (intentional writes)

```bash
cargo run -- prefix create test-hotfix01
cargo run -- prefix verify test-hotfix01
cargo run -- dpi show test-hotfix01
```

The expected result for a newly created prefix is a validated Wine filesystem and a request to set **120 DPI**. Keep the older `test-sprint02` prefix untouched. `prefix repair test-sprint02` should perform *no writes* when the existing prefix passes validation.

## First test: Yaskawa DPS 10

The user-provided file is an x86 PE32 `setup.exe` of roughly 51 MB. Its path contains spaces, so **always quote it**:

```bash
EXE="$HOME/ZDATA/MESHINDO JAYATAMA/MAIN PRODUCTS/YASKAWA/MENTAHAN/YASKAWA - DRIVE PROGRAMMING SIMULATOR/Yaskawa_DPS_10/setup.exe"
cargo run -- inspect "$EXE"
# After a successful prefix test and only if you trust the installer:
cargo run -- prefix create yaskawa-dps10
cargo run -- prefix verify yaskawa-dps10
cargo run -- run yaskawa-dps10 "$EXE"
```

Wine is not a security sandbox; keep the vendor installer files together. Running the installer can change files accessible to your Linux user. Do not connect to active industrial hardware during early tests. No guarantee is made for vendor driver installation, licensing, fieldbus, USB, serial, or Ethernet features.

**Important:** The GUI generates an application ID derived from the installer file path; it can be different from the explicit CLI ID `yaskawa-dps10`. For this first test, use the CLI consistently so the same prefix is reused.

## Safe installation procedure

Use the verified ZIP + installer script. The installer backs up modified files and preserves `README.md`, `LICENSE`, `docs/PRD.md`, and `Cargo.lock`. It does not use `sudo` or touch `~/.wine`.

The ZIP includes the full Sprint 02 source tree (all current Rust sources, original Sprint 01/02 scripts and docs, and hotfix scripts/docs), not just a patch. `target/` and Wine prefixes are deliberately excluded.

## Acceptance criteria

- [ ] `cargo fmt --all` followed by `cargo fmt --all -- --check` passes.
- [ ] `cargo test --locked` passes all tests.
- [ ] `cargo check --locked` passes.
- [ ] `cargo run -- prefix verify test-sprint02` reports structurally ready.
- [ ] `cargo run -- dpi show test-sprint02` reports 120 DPI, or an explainable unrecorded value.
- [ ] A brand new test prefix is initialized at 120 DPI.
- [ ] Native GUI opens and displays DPI controls.
- [ ] Yaskawa installer launches only after deliberate user action, and a launch log is saved.

No GitHub release tag and no claim of Yaskawa compatibility until real tests pass.
