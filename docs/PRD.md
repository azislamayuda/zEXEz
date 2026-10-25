# zExEz — Product Requirements Document (PRD)

**Tagline:** Windows Application Compatibility Manager for Linux  
**Document version:** 1.0 (Draft)  
**Date:** 25 October 2026  
**Project:** zExEz  
**Target platform:** Ubuntu Linux x86_64 (first-class), X11/Wayland via GTK4  
**Core stack:** Rust, GTK4, Libadwaita, Wine-compatible runtime  
**Repository destination:** `/home/lamayuda/ZDATA/ME/GITHUB/zExEz/docs/PRD.md`  
**Product status:** Planning — no implementation or application compatibility has been verified yet.

> **Important:** zExEz will have a native Linux interface, but it does not convert Windows `.exe` files into native Linux binaries. Windows applications will run through a compatibility runtime (initially Wine). Running an installer, opening the application, and communicating safely with industrial hardware are separate milestones. No Yaskawa application is claimed to work before version-specific testing.

---

## 1. Executive summary

zExEz is an open-source, Rust-based desktop application designed to install, launch, diagnose, and maintain a **small set of user-selected Windows programs** on Ubuntu. The priority use case is industrial engineering software such as Yaskawa DriveWizard, SigmaWin+, and MPE720, which may include Win32 (x86) or Win64 (x64) executables and legacy runtimes.

The project prioritizes **repeatable compatibility, offline operation, individual application isolation, stable runtime versions, recoverability, and transparent diagnostics** over broad but unreliable support for thousands of Windows programs.

The zExEz GUI and orchestration are native Linux; actual Win32/Win64 APIs are handled by Wine. In the initial phase, Wine 11 stable is a *candidate baseline*, not a guaranteed best version for every program. Runtime selection must remain profile-specific.

### Problem statement

An Ubuntu user needs a handful of Windows engineering applications for daily work, but installation often fails or becomes fragile due to dependencies, 32-/64-bit differences, runtime changes, vendor USB drivers, or hidden Windows assumptions. A generic Wine configuration is difficult to reproduce, troubleshoot, and maintain. zExEz should make each application's environment explicit, individually manageable, backed up, and testable.

### Product positioning

- **What it is:** A native Rust/GTK application manager and Wine compatibility orchestrator.
- **What it is not:** A Windows emulator, a Windows VM, a Windows kernel-driver host, an EXE-to-ELF compiler, or a guarantee of industrial-device connectivity.
- **Who it serves first:** An Ubuntu engineering workstation whose owner already has legitimate Windows installers and licenses.

## 2. Objectives and measurable outcomes

| ID | Goal | Proposed acceptance target |
|---|---|---|
| G-01 | Install common EXE and MSI packages | Supported test installers complete and produce a registered launch target |
| G-02 | Run x86 and x64 Windows applications | Separate reference PE32 and PE32+ programs launch successfully in an x86_64 Ubuntu test system |
| G-03 | Isolate application environments | Each registered application has its own Wine prefix, config, runtime pin, and logs by default |
| G-04 | Make installation reproducible | Exported profile + runtime version + recorded dependencies permit repeat installation on the same supported system |
| G-05 | Recover from broken configuration | Backup/restore passes an automated prefix integrity and application relaunch test |
| G-06 | Keep desktop UI responsive | Installation, download, and log collection never block the GTK main thread |
| G-07 | Limit privacy exposure | No mandatory account, telemetry, or background upload; log export requires an explicit action |
| G-08 | Establish per-application confidence | Every tested Yaskawa version has an evidence-backed status: untested / launches / offline functional / online validated / blocked |
| G-09 | Release via GitHub | CI checks succeed and a manual draft-release workflow attaches the built AppImage and checksum |

**Success is measured for the selected programs and tested versions, not for all EXE files.** A target application that launches but cannot talk to hardware is counted as *offline functional*, not *fully compatible*.

## 3. Initial target application research

This is a **candidate list**, not a claim about the user's installed versions. Test entries are populated only when the actual installer version, product model, connection method, and license are known.

| Candidate | Typical use | Known dependencies / connection surface | Initial assessment |
|---|---|---|---|
| **Yaskawa DriveWizard Industrial** | Inverter drive configuration, monitoring, reports | Windows application; serial/USB/option-card Ethernet connectivity; some variants use direct USB | UI and offline project workflows first; test serial/Ethernet later; direct USB may be blocked |
| **Yaskawa SigmaWin+ Ver. 7** | Servo setup, parameter viewing, tuning | Vendor documentation specifies Microsoft .NET Framework 4.6.2 and USB/Ethernet connection variants; separate vendor USB drivers exist | Installation and offline features are plausible research targets; direct device USB is high-risk |
| **Yaskawa MPE720 Ver. 7** | Machine controller engineering | Windows-oriented development tool; version-dependent OS/runtime compatibility and serial/USB/Ethernet connectivity | Requires specific version and controller test; may have unsupported driver-dependent functions |
| **Other legitimate Windows EXE/MSI** | Selected engineering and utility programs | Depends on application and installer | Evaluate case by case, never label supported based only on PE architecture |

**Reference facts:**

- Yaskawa describes DriveWizard Industrial as a Windows application and documents serial, direct USB, and Ethernet connectivity: https://www.yaskawa.com/products/drives/industrial-ac-drives/industrial-software-tools/drivewizard-industrial/-/content/_9c662f0e-c82d-4dfd-b240-f2c0de6e7bdb_Features
- Yaskawa currently lists SigmaWin+ 7.54 (document dated 17 August 2026): https://www.yaskawa.com/downloads/search-index/details?docnum=SigmaWinPlus_Ver.7&showType=details
- Yaskawa SigmaWin+ documentation lists .NET Framework 4.6.2 and Windows x86/x64 system requirements: https://www.e-mechatronics.com/download/tool/servo/sgmwinplsver7/en/SIETS80000134.html
- Yaskawa documents a `CpUSB.sys` Windows driver for certain SigmaWin+ USB workflows: https://www.yaskawa.com/about-us/contact-us?_com_yaskawa_contact_us_contactUsActionPortlet_safetyText=true&_com_yaskawa_contact_us_contactUsActionPortlet_selectedDocumentId=MTN-9AYJSQ
- MotionWorks IEC is a **separate** Yaskawa application family; it is not in the first release scope: https://www.yaskawa.com/products/motion/machine-controllers/motionworksiec-requirements

## 4. Target users and primary workflows

**Persona A — industrial engineer (primary).** Uses Ubuntu as the main desktop; occasionally needs to inspect parameters or projects for Yaskawa inverter and servo tools; may work offline near machines; must avoid accidental writes to physical equipment.

**Persona B — Linux developer/maintainer.** Needs clear runtime configuration, reproducible bug reports, minimal dependencies, GitHub automation, and debuggable failures.

### Workflow A: Install a Windows application

1. Open zExEz and choose **Install Windows App**.
2. Select an existing `.exe` or `.msi`; file is analyzed locally for PE type (x86/x64) and hash; mismatched or unreadable inputs are reported.
3. Choose application name and profile, or allow zExEz to create a clean one.
4. Review planned runtime, dependencies, disk usage, filesystem access, and any expected compatibility limitations.
5. zExEz runs the installer in a child process within the selected prefix, shows progress/logs, and handles cancellation.
6. After installation, detect or manually select the launchable executable; save its path, parameters, icon, and profile.
7. Launch the application; optionally create a desktop shortcut.
8. If it fails, offer diagnostics, restore/rollback, and a manual report export.

### Workflow B: Open an already-installed Windows program

- Open the app from the library with its pinned runtime, arguments, configured Windows version, selected prefix, and environment.
- zExEz reports launch outcome and process exit; it does not claim the application is healthy merely because a process starts.

### Workflow C: Connect to Yaskawa hardware

1. Select the application and its **Hardware** page.
2. Choose connection type: **serial**, **Ethernet**, or **direct USB**.
3. Show detected Linux port/interface and required OS permissions without silently modifying privileged settings.
4. Perform **non-destructive checks only** (port availability, permission check, network reachability where allowed) and label their limitations.
5. Require explicit acknowledgement before the user starts the vendor application's own communication operations.
6. Keep live-device tests out of unattended CI; record results against precise application version, device model, firmware, adapter/cable, and Linux kernel.

**Safety:** zExEz cannot generally intercept every write or motion command made inside a Windows engineering application. There is no universal enforceable “read-only engineering mode” provided by zExEz. Hardware testing must take place on a controlled bench with the equipment safely isolated or otherwise professionally safeguarded.

## 5. Functional requirements

Priority labels: **P0** must exist in first public MVP; **P1** needed for Yaskawa-focused beta; **P2** later.

| ID | Priority | Requirement | Testable behavior |
|---|---|---|---|
| FR-01 | P0 | Native GTK4/Libadwaita application library | Add, search, view, launch, delete entry without GUI freeze |
| FR-02 | P0 | EXE and MSI installer workflows | Start/stop installers safely, capture exit status and sanitized logs |
| FR-03 | P0 | PE architecture detection | Distinguish x86 PE32 and x64 PE32+; flag unsupported architectures and unknown installers |
| FR-04 | P0 | Wine prefix per installed app | Unique registry/C: drive and settings, never mutate another profile by default |
| FR-05 | P0 | Managed runtime selection | Detect installed Wine; show exact version/build; pin before creating prefix |
| FR-06 | P0 | Launch target discovery and manual override | Save verified target executable, working directory, arguments |
| FR-07 | P0 | Installation/launch logs | Filter severity, copy diagnostics, export only when requested |
| FR-08 | P0 | Failure recovery and cancellation | Stop owned process trees, clean partial state with explicit confirmation |
| FR-09 | P0 | Offline use | Reopen/run already configured apps without zExEz account or network |
| FR-10 | P1 | Dependency manager | Offer prevalidated .NET, VC++ runtimes and fonts per profile; allow user-supplied offline installers |
| FR-11 | P1 | Backup and restore | Consistent per-prefix snapshots with checksums and compatibility metadata |
| FR-12 | P1 | Windows version / override management | Profile-specific Windows version and DLL overrides, explicit change history |
| FR-13 | P1 | Serial device mapping | Map permitted `/dev/ttyUSB*`, `/dev/ttyACM*` or `/dev/ttyS*` to Windows COM ports where feasible |
| FR-14 | P1 | Ethernet configuration assistance | Display adapter/IP/network checks; avoid raw packet or firewall changes without consent |
| FR-15 | P1 | Application compatibility report | Versioned, reproducible evidence for install/launch/offline/hardware stages |
| FR-16 | P1 | Safe runtime transition | Clone/backup prefix before testing a runtime upgrade; provide rollback plan |
| FR-17 | P2 | Direct USB research module | Report device USB ID, kernel driver ownership, access; mark vendor-driver limitations |
| FR-18 | P2 | Portable EXE profile | Register existing portable EXE without full installation |
| FR-19 | P2 | Import/export compatibility profile | Machine-readable manifest without bundling proprietary installers |
| FR-20 | P2 | GitHub release update checker | Explicit/manual update check and verifiable release artifacts |

### Dependency manager safeguards

- Do not install Microsoft components automatically without user consent and a license-compatible distribution path.
- Record source, version, checksum, and installation outcome of each package.
- Not every .NET Framework version works equivalently under Mono or Wine; support must be tested per application.
- Do not conflate a native Linux `.NET` runtime with the Windows `.NET Framework` requirements inside a Wine prefix.

## 6. Technical architecture

```text
+----------------------------------------------------------+
| zExEz AppImage: Native Rust + GTK4 + Libadwaita            |
| Dashboard | Install Wizard | Library | Profiles | Logs     |
+-----------------------------+----------------------------+
                              |
                 Rust application services
      installer | launch | runtime | prefix | backups | IO
                              |
               command/process abstraction
                              |
+-----------------------------v----------------------------+
| Wine runtime (system-installed or separately managed)    |
| Individual Wine prefixes (separate C: + registry + DLLs) |
+-----------------------------+----------------------------+
                              |
                Windows EXE/MSI user applications
                              |
              Optional Linux device/network access
                   (explicit, tested, limited)
```

### Rust components / planned modules

```text
src/
  main.rs                 # GTK initialization
  application.rs          # application startup/lifecycle
  ui/                     # GTK4 + Libadwaita views and dialogs
  domain/                 # app, runtime, profile, compatibility models
  services/
    pe_inspect.rs         # read PE architecture / metadata
    installer.rs          # exe/msi install jobs
    launch.rs             # process ownership, launch, exit handling
    runtime.rs            # Wine runtime detection/pinning
    prefix.rs             # creation, metadata, management
    dependencies.rs       # approved dependency installation
    backup.rs             # prefix snapshots, checksums, restore
    devices.rs            # read-only Linux device inventory, COM mapping
    diagnostics.rs        # log collection, redaction, export
  storage/                # SQLite app catalog + TOML manifests
  security/               # path checks, safe process execution
```

**Asynchronous execution:** Use GTK/GLib main loop for UI; run Wine and file/network operations outside the GTK main thread with controlled worker tasks. Invoke binaries via typed process arguments (`std::process::Command` or equivalent), **never concatenate untrusted EXE paths into a shell command**.

**Persistence proposal:** SQLite for app registry/history; TOML manifests for portable profile settings; data and cache use XDG directories.

```text
~/.config/zexez/                    # global preferences
~/.local/share/zexez/apps/         # application metadata
~/.local/share/zexez/prefixes/     # isolated Wine prefixes
~/.local/share/zexez/runtimes/     # separately managed runtimes
~/.local/share/zexez/backups/      # local snapshots
~/.cache/zexez/logs/               # transient diagnostics
```

No installer EXE, Microsoft runtime, proprietary Yaskawa software, or Windows DLL will be committed to the public zExEz repository.

## 7. x86 / x64 compatibility strategy

**Host:** Ubuntu x86_64. **Guest application:** Windows x86 or Windows x64; these are application bitness categories, not distinct Linux host architectures.

- Use a supported 64-bit Wine prefix as the default, with Wine's WoW64 functionality for x86 Windows applications where viable.
- Wine 11.0 considers its new WoW64 architecture fully supported, but that does **not** imply full application or Windows-driver compatibility.
- For legacy software that specifically depends on an old 32-bit prefix, consider a separately managed *legacy runtime* only if available, secure, and demonstrably necessary. Do not assume `WINEARCH=win32` works with every modern Wine distribution.
- Architecture of an existing prefix is determined at creation; changing it later is not supported as a general in-place operation. Create a new prefix/migrate data instead.
- Validate install-time prerequisites and launch-time modules: PE type, child processes, 32-/64-bit DLL alignment, Windows version, registry entries, COM components, fonts, and .NET/VC++ requirements.
- Detect runtime and prefix incompatibilities before launch; do not silently upgrade prefixes.

References: Wine 11.0 release https://www.winehq.org/news/2026011301 and Wine manual https://man.archlinux.org/man/wine.1.en.

## 8. Industrial hardware support and boundaries

| Path | Feasibility assessment | Required validation |
|---|---|---|
| **Ethernet TCP/UDP via existing Linux network stack** | Often the best initial hardware candidate where vendor software uses standard networking | Discovery, subnet selection, ports, firewall, controller protocol and actual connection |
| **USB-to-serial adapter (`ttyUSB`/`ttyACM`)** | Potentially feasible if Linux recognizes adapter and application accepts COM serial | Device permissions, stable port mapping, speed/framing, unplug/replug, read/write on bench |
| **Physical RS-232 (`ttyS`)** | Potentially feasible where supported | Cabling, baud rate, serial parity and actual vendor software |
| **Native USB device using proprietary Windows `.sys` driver** | **High risk / often blocked**; Wine does not provide arbitrary Windows kernel drivers on Linux | Identify USB protocol and whether a supported Linux user-space interface exists; otherwise do not promise support |
| **PCI/PCIe / specialized controller drivers** | Generally outside MVP; often not viable via Wine | Hardware-specific research; use supported Windows system if needed |
| **Servo tuning, flash firmware, parameter write, live motion** | Safety-critical | Explicit professional bench validation; never included in automated acceptance without controlled equipment |

For Yaskawa SigmaWin+ especially, vendor instructions reference a Windows `CpUSB.sys` driver. Installing that `.sys` inside Wine is **not equivalent** to installing a native Linux kernel driver. No feature should be represented as 'hardware supported' without a documented device test.

**Fallback when hardware cannot be supported:** use a genuine Windows installation or licensed Windows VM with appropriately supported USB passthrough, subject to vendor requirements. This is an external fallback, not a feature guarantee of zExEz.

## 9. Non-functional requirements

| ID | Requirement | Quality bar |
|---|---|---|
| NFR-01 | UX | Responsive GTK4 layout; keyboard navigation, clear error messages, progress and cancellation |
| NFR-02 | Performance | Initial *measurement target*: app shell opens within 3s on reference workstation and stays under 250 MiB at idle; benchmarks may revise targets |
| NFR-03 | Reliability | Changing settings for App A never changes App B unless the user explicitly shares resources |
| NFR-04 | Recovery | Prefix backup metadata is checksum-verified; interrupted restore is recoverable or fails cleanly |
| NFR-05 | Privacy | No telemetry by default; all diagnostics local; user previews exported logs |
| NFR-06 | Security | Never ask to run the whole app or a Windows installer with `sudo`; validate/download verified runtime artifacts |
| NFR-07 | Licensing | zExEz core MIT; preserve Wine and other third-party license obligations; no unlicensed redistribution |
| NFR-08 | Offline-first | Already-provisioned runtime, dependencies, and apps can operate without internet subject to their own licensing/services |
| NFR-09 | Compatibility communication | No blanket claim that `x86/x64 supported` means any Windows app will function |
| NFR-10 | Testability | Unit/integration/GUI smoke tests, reference installers, and manually verified industrial device scenarios |

### Security model

A Wine prefix is **not a sandbox**; Windows applications may access host files permitted to the Linux user. zExEz must warn before executing unknown binaries, discourage running as root, verify downloaded runtime hashes/signatures when available, and never infer that a clean prefix provides security isolation. An optional restricted filesystem/container mode may be researched later, but it must not be advertised as a full security boundary without testing (including GPU, networking, Wayland/X11 and hardware integration).

Backups and exported logs may contain project names, registry data, license information, host paths, or device identifiers. Export should include a preview and redaction options; users must review logs before publishing them to GitHub.

## 10. UX / information architecture

- **Dashboard:** installed applications, statuses, quick launch, recent failures.
- **Install:** choose EXE/MSI, compatibility checks, runtime, dependencies, preview, progress.
- **Library:** app-specific launch, paths, backup, uninstall, shortcut.
- **Compatibility:** architecture, Windows version, DLL overrides, runtime pin, profile notes.
- **Hardware:** serial/Ethernet/direct USB diagnostics, clear 'unsupported/unverified' indicators.
- **Diagnostics:** process logs, installer result, environment summary, export report.
- **Settings:** theme/system, storage paths, cache management, privacy, release updates.

Statuses must remain descriptive and evidence-based:

1. **Untested** — no result collected.
2. **Installable** — installer completes.
3. **Launchable** — executable opens.
4. **Offline functional** — selected non-hardware workflows work.
5. **Hardware validated** — specified connection and operation were verified on test equipment.
6. **Blocked** — reproducible failure with diagnostic reference.

A status applies to an **exact application build + runtime + OS + connection method**, not every version.

## 11. Compatibility and safety test matrix

| Test | Windows app | Host/prefix | Expected result for release gate |
|---|---|---|---|
| T01 | Small known-good PE32 GUI program | x86_64 Ubuntu + WoW64 | Launch and clean exit |
| T02 | Small known-good PE32+ GUI program | x86_64 Ubuntu + 64-bit prefix | Launch and clean exit |
| T03 | Reference EXE setup (32-bit) | Clean profile | Install/launch/remove behavior recorded |
| T04 | Reference MSI setup | Clean profile | Installer outcome and launch target recorded |
| T05 | Missing prerequisite/.NET | Clean profile | Clear explanation, safe retry and dependency tracking |
| T06 | Portable Windows executable | Dedicated profile | Start and preserve configuration |
| T07 | Invalid/unsupported PE/installer | N/A | Graceful error; nothing executed |
| T08 | Runtime upgrade attempt | Existing prefix | Snapshot, compatibility warning, safe rollback |
| T09 | Power-loss/interrupted install simulation | Test prefix | No registry corruption in other prefixes |
| T10 | Offline restart | Existing local runtime | Launch without network |
| T11 | DriveWizard exact version | No industrial hardware | Install/launch and offline workflows documented; not presumed to pass |
| T12 | SigmaWin+ exact version | No industrial hardware | Installer/.NET behavior and offline workflows documented; not presumed to pass |
| T13 | MPE720 exact version | No industrial hardware | Installer/launch/offline workflow documented; not presumed to pass |
| T14 | USB-serial on isolated bench | Specific adapter + Yaskawa device | Serial permission and communication tested; record any failure |
| T15 | Ethernet on isolated bench | Specific Yaskawa controller/drive | Read-only/non-destructive verified communication where vendor tool permits |
| T16 | Vendor direct USB on isolated bench | Specific model/cable/driver | Mark supported only after actual test; otherwise blocked/unverified |
| T17 | Application restart after crash | Any test app | Diagnostics available, unrelated apps unaffected |
| T18 | AppImage fresh user | Ubuntu 24.04/26.04 LTS where supported by dependencies | GUI starts, correct runtime onboarding, no bundled proprietary components |

**Industrial tests must not automate writing parameters, moving actuators, resetting drives, or flashing firmware.** Release criteria can be satisfied with *honestly documented hardware limitations*; do not label a version production-ready for industrial programming until the user's actual workflow has been validated.

## 12. Acceptance criteria and release gates

### MVP — v0.1.0 (developer preview)

- Native GTK4 interface launches from source on reference Ubuntu system.
- Installer and portable launcher support tested `.exe` and `.msi` files.
- Test application PE32 and PE32+ execute using documented Wine configuration.
- Every app gets a dedicated prefix and tracked configuration.
- Failed installs report actionable errors; cancellation does not affect other profiles.
- Local logs exist; no telemetry or background tracking.
- Rust formatting, lint/static checks, tests, and AppImage smoke test pass.
- Documentation states that Yaskawa/hardware support is **not yet verified**.

### Yaskawa-focused beta — v0.2.0

- Support per-app dependency recipes with user approval.
- Backups and restore pass test suite.
- At least **one exact user-selected Yaskawa installer** is tested through install, launch, and agreed offline workflows; record any blockers honestly.
- Report serial/Ethernet capability and device compatibility status separately from installer status.
- Publish compatibility test records (excluding confidential installers and sensitive device data).

### Hardware research — v0.3.0

- USB-to-serial/COM mapping and Ethernet diagnostics work on defined Linux test hardware.
- Controlled bench results documented for selected Yaskawa version/device pair(s), whether successful or blocked.
- No unsafe/unattended hardware-writing test is introduced.

### Stable UX and distribution — v0.4.0

- Repeatable AppImage build pipeline and artifact checksums.
- Separate manual GitHub **Draft Release** workflow aggregates successful artifact outputs and attaches files.
- Upgrade/rollback and profile migration tested against documented version pairs.
- Documentation, privacy/permissions notes, license notices, screenshots, and migration guidance complete.

### Broader compatibility — v0.5.0+

- More per-application profiles and verified compatibility data.
- Advanced diagnostics and optional filesystem restrictions, only when accurately tested.
- Additional packaging (DEB/RPM/Arch) **only if maintenance cost is justified**.

## 13. CI/CD and repository plan

Proposed repository layout:

```text
zExEz/
├── README.md
├── LICENSE
├── Cargo.toml                     # after project initialization
├── src/                           # after implementation begins
├── assets/                        # icons, metadata
├── docs/
│   ├── PRD.md                     # this document
│   ├── ARCHITECTURE.md             # later
│   ├── COMPATIBILITY.md            # later
│   └── TEST-PLAN.md               # later
├── packaging/appimage/             # later
└── .github/workflows/
    ├── ci.yml                      # fmt, clippy, test, audit/checks
    ├── appimage.yml                # build, smoke test, upload artifact
    └── draft-release.yml           # workflow_dispatch; draft release
```

CI/CD security gates:

1. Separate CI from publish permissions; pinned actions and least-privilege tokens.
2. Version tag/release input must match built artifact metadata.
3. Draft release must attach only verified build outputs, not rebuild unrelated binaries silently.
4. Publish SHA-256 checksums and dependency/license notices.
5. Never upload user's installers, purchased binaries, product keys, local prefix contents, or proprietary Yaskawa files.
6. Keep Wine runtime externally provisioned initially; if a runtime is redistributed later, satisfy upstream licensing and notices.

## 14. Dependencies and distribution model

- **Rust source:** MIT license as selected for zExEz, subject to final copyright metadata.
- **GTK4/Libadwaita:** native Linux desktop UI; versions pinned for build reproducibility.
- **Wine:** third-party compatibility runtime; comply with its open-source licenses and source/notice obligations when distributed.
- **Winetricks / dependency installers:** external tools/components, opt-in and independently licensed; do not run arbitrary community install scripts without review.
- **Yaskawa software:** proprietary and not shipped with zExEz. End user supplies legally acquired installer and valid license.
- **Microsoft runtimes:** licensing/distribution terms respected; user-supplied or properly authorized downloads only.
- **AppImage:** first packaging target is **native zExEz GUI + runtime manager**. A huge all-in-one Wine AppImage is not promised; hardware integration still relies on host Linux facilities. Building against compatible system libraries is essential.

AppImage references: https://docs.appimage.org/reference/best-practices.html and https://docs.appimage.org/introduction/concepts.html.

## 15. Risks, mitigations, and go/no-go decisions

| Risk | Impact | Mitigation / decision |
|---|---|---|
| Proprietary Yaskawa Windows kernel driver required | Critical | Mark hardware function blocked; evaluate supported serial/Ethernet alternative, otherwise external Windows fallback |
| .NET Framework / COM / legacy installer failure | High | Version-specific test profile, log capture, controlled dependency alternatives |
| Wine runtime update breaks legacy app | High | Runtime pinning, backup-before-upgrade, separate prefix and rollback |
| User performs unsafe industrial write | Critical | Explicit warnings, bench-only validation, never claim universal read-only enforcement |
| Wine process accesses private host files | High | Clear permission notices and optional isolation research; Wine prefixes are not security sandboxes |
| AppImage cannot locate graphics/runtime libraries | Medium | Smoke tests on clean supported Ubuntu versions, dependency policy and diagnostics |
| Proprietary redistributions violate terms | High | Do not bundle vendor software, keys, Microsoft DLLs, or unlicensed runtime assets |
| Broad compatibility scope overwhelms maintenance | High | Restrict early milestones to named programs/builds and defined task workflows |
| Original installer fails under Wine due to licensing/DRM | High | Respect vendor licensing; do not bypass activation or protection systems |

**Go/no-go check:** Before promising Yaskawa hardware support, obtain the exact **installer version, 32-/64-bit architecture, Yaskawa device model, firmware, connection cable/interface, and expected workflow**. If the required workflow depends on an unsupported Windows kernel driver, scope it out of the Wine MVP instead of marketing it as compatible.

## 16. Open questions to resolve during implementation

1. Which exact Yaskawa application comes first: DriveWizard Industrial/Plus, SigmaWin+ (which version), MPE720, or another?
2. Which installer format and architecture does each application use?
3. Is the main requirement **offline editing/reporting**, **live monitoring**, **parameter writing**, **tuning**, or **firmware update**?
4. Which model is connected (e.g. A1000, GA500/GA700, Sigma-7/Sigma-X) and over which adapter or cable?
5. Which Ubuntu release, desktop session (Wayland/X11), kernel version and USB adapters are used on the reference workstation?
6. Are activation, software licenses, and installer prerequisites available legally and usable offline?

These questions affect compatibility certification but **do not block starting the generic Rust/GTK foundation**.

## 17. Definition of done

An application is considered **usable in zExEz** only when the selected real-world workflow can be repeated after restart under a pinned runtime, with stable storage, recoverable profile, documented dependencies, and no unexpected effects on other apps.

An application is considered **verified for Yaskawa hardware** only after the exact hardware/interface workflow succeeds under supervised bench testing and the user approves the documented limitations. Neither a completed installer nor a successful splash screen proves hardware compatibility.

---

## 18. Technical references

1. Wine project, *What is Wine?*: https://www.winehq.org/
2. Wine 11.0 release (WoW64): https://www.winehq.org/news/2026011301
3. Wine manual (`WINEPREFIX`, `WINEARCH`): https://man.archlinux.org/man/wine.1.en
4. ArchWiki, *Wine* (prefix/security cautions): https://wiki.archlinux.org/title/Wine
5. Yaskawa, DriveWizard Industrial communication: https://www.yaskawa.com/products/drives/industrial-ac-drives/industrial-software-tools/drivewizard-industrial/-/content/_9c662f0e-c82d-4dfd-b240-f2c0de6e7bdb_Features
6. Yaskawa, SigmaWin+ Ver.7 download/details: https://www.yaskawa.com/downloads/search-index/details?docnum=SigmaWinPlus_Ver.7&showType=details
7. Yaskawa, SigmaWin+ operation manual: https://www.e-mechatronics.com/download/tool/servo/sgmwinplsver7/en/SIETS80000134.html
8. Yaskawa, SigmaWin+ USB driver FAQ: https://www.yaskawa.com/about-us/contact-us?_com_yaskawa_contact_us_contactUsActionPortlet_safetyText=true&_com_yaskawa_contact_us_contactUsActionPortlet_selectedDocumentId=MTN-9AYJSQ
9. AppImage best practices: https://docs.appimage.org/reference/best-practices.html
10. AppImage concepts: https://docs.appimage.org/introduction/concepts.html

**End of PRD v1.0 — Draft for implementation planning.**
