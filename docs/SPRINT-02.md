# zEXEz — Sprint 02: Isolated Wine Runtime Engine

**Status:** Source candidate for verification on Ubuntu 26.04.1 LTS. Not a production release.
**Version:** 0.1.0 (development, no release tag yet).

## Scope

- Preserve working Sprint 01 native GTK4/Libadwaita GUI and Wine diagnostics.
- Inspect `.exe` PE headers for Intel x86 (`0x014c`, PE32) and AMD64 (`0x8664`, PE32+), rejecting unsupported architectures such as ARM64. Inspection never runs the executable.
- Verify `.msi` compound-file magic; MSI target architecture is **unknown** without deeper inspection.
- Suggest unique stable per-file app IDs and restrict user-provided IDs to safe directory components.
- Explicitly create one isolated Wine `win64` prefix per application with `wine wineboot -i`; Wine 10's 32-bit and 64-bit packages are installed separately on Ubuntu. A 32-bit EXE can run in a 64-bit prefix with the appropriate Wine support.
- Explicit launch of `.exe` with `wine FILE`, and `.msi` with `wine msiexec /i FILE`.
- Capture Wine stdout/stderr to per-app log files; no shell interpolation or root privileges.
- Keep GUI responsive while creating prefixes using a worker thread.
- Pure Rust unit tests that don't execute Windows software.

## GUI workflow

1. Select EXE / MSI. Format and architecture are inspected without executing the file.
2. Review the suggested isolated prefix location and status.
3. Click **Initialize Prefix**. This runs Wine and may show Wine configuration or component-install prompts.
4. Click **Run Selected File**. For MSI this opens the Wine MSI installer. For EXE this runs the selected file; it may be an installer or portable application.
5. Read the operation status and Wine log path shown in the GUI.

**Important:** This sprint does not scan the registry for installed Windows applications or build a persistent application library. For regular installed applications, selecting and launching their installed `.exe` in the same prefix is still manual (CLI `run ID FILE`).

## CLI interface

```bash
cd ~/ZDATA/ME/GITHUB/zEXEz
cargo run -- diagnose
cargo run -- inspect '/path/to/DriveWizardSetup.exe'
cargo run -- prefix path test-yaskawa
cargo run -- prefix create test-yaskawa
cargo run -- run test-yaskawa '/path/to/DriveWizardSetup.exe'
```

Only `prefix create` initializes a prefix and only `run` launches Windows code. `inspect`, `prefix path`, `diagnose`, `--help`, `cargo test`, and `cargo check` are non-executing operations.

GUI derives an app ID from the exact file path; CLI accepts a chosen ID. To use one prefix for an installer and its subsequently installed application, keep using the same CLI ID.

## Managed locations

- Prefixes: `${XDG_DATA_HOME:-$HOME/.local/share}/zexez/prefixes/ID/`
- Logs: `${XDG_STATE_HOME:-$HOME/.local/state}/zexez/logs/ID-init.log` and `ID-launch.log`
- Installer backups: `$HOME/.local/share/zexez/backups/sprint02-*/`
- **Never use `~/.wine` as a managed prefix.** No uninstall/delete functionality yet.

## Security and limitations

Wine is **not a sandbox**. Windows programs inherit the permissions of your Linux user and might read/write files your user can access. **Only run trusted applications**. A prefix separates configuration and DLLs, not Linux filesystem security. Do not run Wine as root.

- Running an EXE does not guarantee it will install or operate correctly.
- PE classification describes the outer `.exe`, not the architecture of any bundled program or dependency.
- Industrial Yaskawa DriveWizard / SigmaWin+ / MPE720 support is **unverified**. Vendor USB kernel drivers, licensing dongles, drivers, .NET, and device communication may fail. Do not connect to live equipment during early testing.
- Serial/USB/Ethernet hardware integration, dependencies, runtime version selection, app library, uninstall, and AppImage packaging are out of scope.
- Running an installer may trigger Windows component prompts, network access, or software changes **inside its prefix**.
- Wine logs may contain file paths and environment information; review them before sharing.
- Existing, stale `.zexez-init.lock` indicates interrupted initialization; inspect the prefix and Wine processes before removing the lock manually.

## Verification

```bash
bash scripts/verify-sprint02.sh
cargo run -- --help
cargo run -- inspect /path/to/a/trusted/EXE
cargo run
```

Verification formats Rust sources and should complete without creating a prefix. Only test `prefix create` and `run` after you have reviewed the selected Windows file and wish to execute it.

## Acceptance criteria

- [ ] `cargo fmt --all -- --check` succeeds on Ubuntu.
- [ ] `cargo test --locked` passes all unit tests.
- [ ] `cargo check --locked` succeeds using GTK4 4.22 and Libadwaita 1.9.
- [ ] `cargo run -- diagnose` detects Wine 10.0 and x86/x64 packages.
- [ ] GUI displays the workspace and opens file selection without crashing.
- [ ] PE x86 / x64 inspection and malformed file rejection verified.
- [ ] Prefix initialization is opt-in and uses an isolated XDG path.
- [ ] EXE/MSI launch produces a log and starts only after an explicit user action.
- [ ] README.md, LICENSE, docs/PRD.md and Cargo.lock remain intact.

**No GitHub release or compatibility claim for Yaskawa until additional testing.**
