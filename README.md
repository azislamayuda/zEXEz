<div align="center">

# zExEz

### Run Windows applications on Linux, with a native Rust experience.

**A planned open-source Windows application manager for Linux.**

Rust · GTK4 · Libadwaita · Wine · AppImage

[Overview](#overview) · [Planned Features](#planned-features) · [Roadmap](#roadmap) · [Contributing](#contributing) · [License](#license)

</div>

> [!IMPORTANT]
> **Project status: Planning / early development.** zExEz is not yet a working installer or Windows application runner. Features and release milestones below are proposals, not functionality available today.

## Overview

**zExEz** aims to make installing, launching, and managing selected Windows `.exe` and `.msi` applications on Linux easier through a lightweight, native graphical interface built with Rust.

The goal is a **reliable, application-focused workflow**: create an isolated environment for each Windows application, install it, adjust its compatibility settings when necessary, and launch it again from a single library.

zExEz is **not** an EXE-to-Linux binary converter. Rust will power the Linux application and its management logic; running Windows binaries will rely on a compatibility layer such as **Wine**. Not every Windows program will work, and compatibility depends on the program, its dependencies, the runtime, and the Linux system.

## Project Goals

- Provide a clean, native Linux GUI rather than a web-based desktop shell.
- Prioritize stability for a practical set of Windows applications over claims of universal compatibility.
- Isolate application settings and dependencies in separate Wine prefixes.
- Offer clear installation progress, launch controls, and useful diagnostic logs.
- Keep application data separate from the zExEz executable so updates do not unnecessarily affect installed programs.
- Distribute Linux builds as AppImage once the application is ready, and publish source code and releases on GitHub.

## Planned Features

| Feature | Status |
| --- | --- |
| Native application window using Rust, GTK4, and Libadwaita | Planned |
| Windows EXE and MSI installer launcher | Planned |
| Portable EXE launcher | Planned |
| Per-application Wine prefixes | Planned |
| Installed application library | Planned |
| Installation and runtime logs | Planned |
| Wine runtime selection and management | Future |
| Per-application compatibility profiles | Future |
| Optional dependency helpers (such as .NET components) | Future |
| Prefix backup and restore | Future |
| Linux desktop shortcuts | Future |
| AppImage packaging and GitHub release automation | Future |

*Planned* and *Future* do not mean implemented or tested.

## How It Is Intended to Work

```text
Windows installer (.exe / .msi)
             |
             v
       zExEz (Rust)
       GTK4 / Libadwaita
             |
     +-------+--------+
     |                |
 Installer Manager  App Library
     |                |
     +-------+--------+
             |
       Wine Runtime
             |
  Isolated Wine Prefix
             |
     Windows Program
         on Linux
```

Each application will ideally keep its own Wine prefix, configuration, and metadata. A runtime change for one application should not silently change the settings of another.

## Proposed Architecture

| Component | Responsibility |
| --- | --- |
| Rust | Core orchestration, configuration, application lifecycle, and process management |
| GTK4 + Libadwaita | Native Linux graphical interface |
| Wine | Windows API compatibility layer used to execute Windows software |
| Prefix Manager | Creation and management of isolated Wine prefixes |
| Runtime Manager | Selection and management of Wine versions (planned) |
| AppImage | Portable Linux packaging for zExEz (planned) |
| GitHub Actions | Build, test, and release workflows (planned) |

The first implementation will target **Ubuntu on x86_64**. Broader Linux distribution and architecture support should follow successful testing rather than be assumed.

## Roadmap

Roadmap versions describe **targets, not published releases**. Scope may change as the project evolves.

### v0.1.0 — Foundation

- [ ] Set up the Rust project and GTK4/Libadwaita application shell.
- [ ] Add an application library and basic configuration storage.
- [ ] Detect a supported local Wine installation.
- [ ] Create and manage per-application Wine prefixes.
- [ ] Launch EXE/MSI installers and portable EXE files.
- [ ] Collect installer output and basic error logs.

### v0.2.0 — Compatibility Controls

- [ ] Add runtime selection and application-specific profiles.
- [ ] Support configurable environment variables and launch arguments.
- [ ] Explore dependency installation assistance and troubleshooting guidance.

### v0.3.0 — Application Management

- [ ] Support backup and restore of application prefixes.
- [ ] Improve desktop integration, shortcuts, and application metadata.
- [ ] Improve diagnostics and recovery workflows.

### v0.4.0 — Distribution

- [ ] Package stable builds as AppImage.
- [ ] Add CI checks and reproducible build documentation.
- [ ] Prepare GitHub Releases with changelogs and release artifacts.

## System Requirements (Initial Target)

These are **development targets**, not validated minimum requirements.

- A recent Ubuntu desktop running **x86_64**.
- GTK4 and Libadwaita development libraries for building the GUI.
- The Rust toolchain (`rustup`, `cargo`) for source builds.
- A compatible Wine installation to execute Windows applications.
- Sufficient disk space for separate Wine prefixes and Windows software.

The exact supported Ubuntu versions, Wine versions, runtime dependency strategy, and build instructions will be documented after the first executable implementation is available.

## Building and Installing

**Build instructions are not available yet.** There is currently no verified `cargo build` or AppImage installation procedure in this documentation. Once the initial source tree and dependencies are committed, this section will include reproducible commands and tested environment details.

Do not download or execute unverified binaries claiming to be zExEz. When releases are available, use the official project's GitHub Releases page.

## Compatibility and Limitations

- **Wine is required for the planned Windows execution path.** zExEz will not natively translate Windows executables into Linux ELF binaries.
- **Compatibility is not guaranteed.** Applications requiring Windows kernel drivers, certain DRM/anti-cheat systems, or unsupported OS features may fail.
- **Wine is not a security sandbox.** Windows programs can access resources available to the Linux user unless separately restricted. Run only software you trust.
- Some programs require additional fonts, runtime components, graphics drivers, or licensing.
- AppImage packaging alone does not guarantee that every Windows application, Wine runtime, or Linux distribution will behave identically.

## Contributing

Contributions, testing results, and constructive feedback are welcome once the repository is open for development.

1. Check existing issues before opening a new bug report or feature request.
2. Include your Linux distribution, CPU architecture, Wine version, and steps to reproduce when reporting compatibility problems.
3. Remove private information, license keys, personal file paths, and confidential documents from attached logs.
4. Keep proposed changes focused and explain the expected behavior in pull requests.

Please do not upload proprietary Windows installers or other software you do not have permission to redistribute.

## License

zExEz source code is intended to be distributed under the **MIT License**. See [LICENSE](LICENSE) for the complete terms.

Third-party components, including Wine and any bundled dependencies, retain their **own licenses**. The MIT License for zExEz does not replace or relicense them.

## Disclaimer

zExEz is an independent open-source project and is not affiliated with, endorsed by, or sponsored by Microsoft, WineHQ, or CodeWeavers. Windows and other product names belong to their respective owners.
