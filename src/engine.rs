//! Isolated Wine prefix lifecycle and Windows process launcher.
//! Wine is not a sandbox. No shell, sudo, or implicit access to ~/.wine.

use crate::pe::{self, ProgramKind};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};

pub const DEFAULT_DPI: u32 = 120;
pub const DPI_OPTIONS: [u32; 5] = [96, 120, 144, 168, 192];

#[derive(Clone, Debug)]
pub struct Selection {
    pub path: PathBuf,
    pub id: String,
    pub kind: ProgramKind,
}

pub struct LaunchInfo {
    pub pid: u32,
    pub log: PathBuf,
}

pub fn select_file(path: &Path) -> Result<Selection, String> {
    let kind = pe::inspect(path)?;
    let absolute = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve file path: {e}"))?;
    let id = suggested_id(&absolute);
    Ok(Selection {
        path: absolute,
        id,
        kind,
    })
}

/// IDs are single safe directory names, not arbitrary paths.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 48
        && id
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
        && !id.starts_with('-')
        && !id.ends_with('-')
}

pub fn suggested_id(path: &Path) -> String {
    let stem = path
        .file_stem()
        .unwrap_or_else(|| std::ffi::OsStr::new("application"))
        .to_string_lossy();
    let mut slug = String::new();
    for c in stem.chars() {
        if slug.len() >= 24 {
            break;
        }
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    let slug = if slug.is_empty() { "application" } else { slug };
    let hash = path
        .to_string_lossy()
        .bytes()
        .fold(0xcbf29ce484222325u64, |hash, b| {
            (hash ^ u64::from(b)).wrapping_mul(0x100000001b3)
        });
    format!("{slug}-{:08x}", hash as u32)
}

fn xdg_home(variable: &str, fallback: &str) -> PathBuf {
    env::var_os(variable)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| {
            let home = env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/nonexistent"));
            home.join(fallback)
        })
}

fn prefixes_root() -> PathBuf {
    xdg_home("XDG_DATA_HOME", ".local/share")
        .join("zexez")
        .join("prefixes")
}

fn logs_root() -> PathBuf {
    xdg_home("XDG_STATE_HOME", ".local/state")
        .join("zexez")
        .join("logs")
}

pub fn prefix_path(id: &str) -> Result<PathBuf, String> {
    if !valid_id(id) {
        return Err("Invalid app ID: lowercase a-z, 0-9 and hyphens, max 48 chars".into());
    }
    Ok(prefixes_root().join(id))
}

fn refuse_symlink(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_symlink() => Err(format!(
            "Refusing symlink as managed path: {}",
            path.display()
        )),
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("Cannot inspect {}: {e}", path.display())),
    }
}

fn private_dir(path: &Path) -> Result<(), String> {
    refuse_symlink(path)?;
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    builder.mode(0o700);
    builder
        .create(path)
        .map_err(|e| format!("Cannot create {}: {e}", path.display()))?;
    refuse_symlink(path)?;
    Ok(())
}

/// Structural verification is not proof that a Windows app will be compatible.
/// Require the registry and Windows drive, rather than only `system.reg`.
fn missing_components(prefix: &Path) -> Vec<&'static str> {
    let mut missing = Vec::new();
    for (relative, is_dir) in [
        ("drive_c", true),
        ("drive_c/windows", true),
        ("system.reg", false),
        ("user.reg", false),
        ("userdef.reg", false),
    ] {
        let path = prefix.join(relative);
        let exists = if is_dir {
            path.is_dir()
        } else {
            path.is_file()
        };
        if !exists {
            missing.push(relative);
        }
    }
    missing
}

fn prefix_ready_at(prefix: &Path) -> bool {
    missing_components(prefix).is_empty()
}

pub fn prefix_ready(id: &str) -> bool {
    prefix_path(id)
        .ok()
        .and_then(|path| refuse_symlink(&path).ok().map(|_| path))
        .is_some_and(|prefix| prefix.is_dir() && prefix_ready_at(&prefix))
}

/// Read-only prefix report. Never invokes Wine or repairs files implicitly.
pub fn verify_prefix(id: &str) -> Result<String, String> {
    let prefix = prefix_path(id)?;
    refuse_symlink(&prefix)?;
    if !prefix.exists() {
        return Err(format!(
            "Prefix does not exist: {}. Run: zexez prefix create {id}",
            prefix.display()
        ));
    }
    if !prefix.is_dir() {
        return Err(format!("Prefix is not a directory: {}", prefix.display()));
    }
    let missing = missing_components(&prefix);
    if !missing.is_empty() {
        return Err(format!(
            "Incomplete prefix: {} · Missing: {}. Run: zexez prefix repair {id}",
            prefix.display(),
            missing.join(", ")
        ));
    }
    let dpi = match read_prefix_dpi(&prefix) {
        Some(dpi) => format!("{dpi} DPI"),
        None => "not explicitly detected".into(),
    };
    Ok(format!(
        "Prefix structurally ready: {} · Display DPI: {} · Runtime compatibility not guaranteed",
        prefix.display(),
        dpi
    ))
}

fn log_file(id: &str, action: &str) -> Result<(File, PathBuf), String> {
    if !valid_id(id) {
        return Err("Invalid app ID".into());
    }
    let root = logs_root();
    private_dir(&root)?;
    let path = root.join(format!("{id}-{action}.log"));
    refuse_symlink(&path)?;
    let mut options = OpenOptions::new();
    options.create(true).append(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let file = options
        .open(&path)
        .map_err(|e| format!("Cannot open log: {e}"))?;
    Ok((file, path))
}

struct InitLock(PathBuf);
impl Drop for InitLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn acquire_lock(prefix: &Path) -> Result<InitLock, String> {
    let lock = prefix.join(".zexez-init.lock");
    refuse_symlink(&lock)?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    options.open(&lock).map_err(|e| {
        format!(
            "This prefix may already be in use ({e}). Check lock: {}",
            lock.display()
        )
    })?;
    Ok(InitLock(lock))
}

fn wait_for_prefix(prefix: &Path, timeout: Duration) -> Result<(), String> {
    let start = Instant::now();
    loop {
        if prefix_ready_at(prefix) {
            return Ok(());
        }
        if start.elapsed() >= timeout {
            return Err(format!(
                "Registry/drive not ready after Wine exited. Missing: {}",
                missing_components(prefix).join(", ")
            ));
        }
        thread::sleep(Duration::from_millis(250));
    }
}

/// Explicitly initialize a new prefix. Existing ready prefixes remain untouched.
/// Wine may finish some filesystem writes after `wineboot -i` exits.
pub fn initialize_prefix(id: &str) -> Result<String, String> {
    let prefix = prefix_path(id)?;
    private_dir(&prefixes_root())?;
    refuse_symlink(&prefix)?;
    let is_new = !prefix.exists();
    private_dir(&prefix)?;
    if prefix_ready_at(&prefix) {
        return Ok(format!("{} · Already initialized", verify_prefix(id)?));
    }
    let _lock_guard = acquire_lock(&prefix)?;
    let (mut log_file, log_path) = log_file(id, "init")?;
    writeln!(
        log_file,
        "\n=== zEXEz · wineboot {} · {} ===",
        if is_new { "-i" } else { "-u" },
        prefix.display()
    )
    .ok();
    let status = Command::new("wine")
        .arg("wineboot")
        .arg(if is_new { "-i" } else { "-u" })
        .env("WINEPREFIX", &prefix)
        .env("WINEARCH", "win64")
        .stdout(Stdio::from(
            log_file.try_clone().map_err(|e| e.to_string())?,
        ))
        .stderr(Stdio::from(log_file))
        .status()
        .map_err(|e| format!("Cannot execute wineboot: {e}"))?;
    if !status.success() {
        return Err(format!(
            "wineboot failed ({status}). Log: {}",
            log_path.display()
        ));
    }
    wait_for_prefix(&prefix, Duration::from_secs(20)).map_err(|e| {
        format!(
            "{e} · Wine exit status: {status} · Log: {}",
            log_path.display()
        )
    })?;

    // Set 120 DPI only for NEW prefixes. Never rewrite a user's existing DPI.
    if is_new {
        match apply_dpi(id, DEFAULT_DPI) {
            Ok(_) => Ok(format!(
                "Prefix ready: {} · Default DPI: {} · Log: {}",
                prefix.display(),
                DEFAULT_DPI,
                log_path.display()
            )),
            Err(error) => Ok(format!(
                "Prefix ready: {} · DPI auto-configuration warning: {error} · Log: {}",
                prefix.display(),
                log_path.display()
            )),
        }
    } else {
        Ok(format!(
            "Existing prefix repaired/initialized: {} · Existing DPI preserved · Log: {}",
            prefix.display(),
            log_path.display()
        ))
    }
}

/// Recovery is opt-in, non-destructive, and does not recreate a ready prefix.
pub fn repair_prefix(id: &str) -> Result<String, String> {
    let prefix = prefix_path(id)?;
    refuse_symlink(&prefix)?;
    if !prefix.is_dir() {
        return Err(format!(
            "Prefix missing: {}. Use prefix create first",
            prefix.display()
        ));
    }
    if prefix_ready_at(&prefix) {
        return verify_prefix(id);
    }
    initialize_prefix(id)
}

/// Read Wine's user.reg without invoking Wine. Wine commonly records DPI in
/// `Control Panel\\Desktop` and sometimes `Software\\Wine\\Fonts`. Keep the
/// HKCU Desktop value authoritative; never use system.reg as a fallback.
fn read_dpi_from_registry(contents: &str) -> Option<u32> {
    let mut section = String::new();
    let mut desktop_dpi = None;
    let mut wine_fonts_dpi = None;
    // Some Wine registry snapshots use a different escaping style. A fallback
    // is safe only when ALL LogPixels entries in user.reg agree on one value.
    let mut common_value = None;
    let mut conflicting_values = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix('[') {
            if let Some((header, _timestamp)) = rest.split_once(']') {
                section = header
                    .replace("\\\\", "\\")
                    .replace("\\\\", "\\")
                    .to_ascii_lowercase();
            } else {
                section.clear();
            }
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case("\"LogPixels\"") {
            continue;
        }
        let value = value.trim();
        let Some(hex) = value
            .strip_prefix("dword:")
            .or_else(|| value.strip_prefix("DWORD:"))
        else {
            continue;
        };
        let hex = hex.trim();
        if hex.is_empty() || hex.len() > 8 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            continue;
        }
        let Ok(dpi) = u32::from_str_radix(hex, 16) else {
            continue;
        };
        // Bound the display value to reasonable Windows DPI settings.
        if !(48..=768).contains(&dpi) {
            continue;
        }
        if let Some(previous) = common_value {
            if previous != dpi {
                conflicting_values = true;
            }
        } else {
            common_value = Some(dpi);
        }
        match section.as_str() {
            "control panel\\desktop" => desktop_dpi = Some(dpi),
            "software\\wine\\fonts" => wine_fonts_dpi = Some(dpi),
            _ => {}
        }
    }

    desktop_dpi.or(wine_fonts_dpi).or_else(|| {
        if conflicting_values {
            None
        } else {
            common_value
        }
    })
}

fn read_prefix_dpi(prefix: &Path) -> Option<u32> {
    // A Wine registry is usually small; cap reads for accidental/malicious huge files.
    let path = prefix.join("user.reg");
    if fs::metadata(&path).ok()?.len() > 8 * 1024 * 1024 {
        return None;
    }
    let contents = fs::read(&path).ok()?;
    read_dpi_from_registry(&String::from_utf8_lossy(&contents))
}

pub fn prefix_dpi(id: &str) -> Option<u32> {
    let prefix = prefix_path(id).ok()?;
    refuse_symlink(&prefix).ok()?;
    if !prefix_ready_at(&prefix) {
        return None;
    }
    read_prefix_dpi(&prefix)
}

pub fn dpi_summary(id: &str) -> Result<String, String> {
    verify_prefix(id)?;
    Ok(match prefix_dpi(id) {
        Some(dpi) => format!("{dpi} DPI ({}%)", dpi * 100 / 96),
        None => "DPI not explicitly found in Wine registry (use winecfg to inspect)".to_string(),
    })
}

/// Set an explicit user-chosen DPI through Wine registry (no shell commands).
/// Apps already running must be restarted to see the new scale.
pub fn apply_dpi(id: &str, dpi: u32) -> Result<String, String> {
    if !DPI_OPTIONS.contains(&dpi) {
        return Err("Unsupported DPI. Choose 96, 120, 144, 168, or 192".into());
    }
    let prefix = prefix_path(id)?;
    refuse_symlink(&prefix)?;
    if !prefix_ready_at(&prefix) {
        return Err(format!("Prefix is not initialized: {}", prefix.display()));
    }
    let (mut output, log_path) = log_file(id, "dpi")?;
    writeln!(
        output,
        "\n=== zEXEz set DPI {dpi} · {} ===",
        prefix.display()
    )
    .ok();
    let dpi_argument = dpi.to_string();
    let status = Command::new("wine")
        .args([
            "reg",
            "add",
            "HKEY_CURRENT_USER\\Control Panel\\Desktop",
            "/v",
            "LogPixels",
            "/t",
            "REG_DWORD",
            "/d",
            dpi_argument.as_str(),
            "/f",
        ])
        .env("WINEPREFIX", &prefix)
        .env("WINEARCH", "win64")
        .stdout(Stdio::from(output.try_clone().map_err(|e| e.to_string())?))
        .stderr(Stdio::from(output))
        .status()
        .map_err(|e| format!("Cannot execute Wine registry command: {e}"))?;
    if !status.success() {
        return Err(format!(
            "Wine reg add failed ({status}). Log: {}",
            log_path.display()
        ));
    }
    // wine reg may return before wineserver flushes user.reg to disk.
    // Do not claim a persisted DPI setting unless we can read it back.
    let started = Instant::now();
    while read_prefix_dpi(&prefix) != Some(dpi) {
        if started.elapsed() >= Duration::from_secs(5) {
            return Err(format!(
                "Wine accepted {dpi} DPI but persistence was not verified in user.reg. Check winecfg. Log: {}",
                log_path.display()
            ));
        }
        thread::sleep(Duration::from_millis(200));
    }
    Ok(format!(
        "Verified {dpi} DPI in {}. Restart Windows apps to apply. Log: {}",
        prefix.display(),
        log_path.display()
    ))
}

/// Launching Wine means process startup, NOT proof that the app works.
pub fn launch(id: &str, file: &Path) -> Result<LaunchInfo, String> {
    let prefix = prefix_path(id)?;
    refuse_symlink(&prefix)?;
    if !prefix_ready_at(&prefix) {
        return Err(format!(
            "Prefix not initialized. Run: zexez prefix create {id}"
        ));
    }
    let kind = pe::inspect(file)?;
    let absolute = file
        .canonicalize()
        .map_err(|e| format!("Cannot resolve executable: {e}"))?;
    let parent = absolute
        .parent()
        .ok_or("Cannot determine executable directory")?;
    let (mut output, log) = log_file(id, "launch")?;
    writeln!(
        output,
        "\n=== zEXEz launch · {} · {} ===",
        kind,
        absolute.display()
    )
    .ok();
    let mut command = Command::new("wine");
    if kind == ProgramKind::Msi {
        command.args(["msiexec", "/i"]);
    }
    command.arg(&absolute);
    command.env("WINEPREFIX", &prefix);
    command.env("WINEARCH", "win64");
    command.current_dir(parent);
    command.stdin(Stdio::null());
    command.stdout(Stdio::from(output.try_clone().map_err(|e| e.to_string())?));
    command.stderr(Stdio::from(output));
    let mut child = command
        .spawn()
        .map_err(|e| format!("Cannot start Wine: {e}"))?;
    let pid = child.id();
    thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(LaunchInfo { pid, log })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal_ids() {
        for id in [
            "",
            "..",
            "../abc",
            "a/b",
            "A",
            "two words",
            "-leading",
            "trailing-",
            "a\\b",
        ] {
            assert!(!valid_id(id), "accepted unsafe id: {id}");
        }
        assert!(valid_id("yaskawa-dps10"));
    }

    #[test]
    fn generated_ids_are_stable_and_distinct() {
        let first = suggested_id(Path::new("/tmp/one/setup.exe"));
        assert_eq!(first, suggested_id(Path::new("/tmp/one/setup.exe")));
        assert_ne!(first, suggested_id(Path::new("/tmp/two/setup.exe")));
        assert!(valid_id(&first));
    }

    #[test]
    fn invalid_ids_rejected_before_fs_access() {
        assert!(prefix_path("../../home").is_err());
    }

    #[test]
    fn distinguishes_missing_and_ready_prefix_components() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp = env::temp_dir().join(format!("zexez-test-{}-{nonce}", std::process::id()));
        fs::create_dir_all(temp.join("drive_c/windows")).unwrap();
        for name in ["system.reg", "user.reg"] {
            fs::write(temp.join(name), "test").unwrap();
        }
        assert_eq!(missing_components(&temp), vec!["userdef.reg"]);
        fs::write(temp.join("userdef.reg"), "test").unwrap();
        assert!(prefix_ready_at(&temp));
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn parses_registry_dpi_without_launching_wine() {
        let registry =
            "WINE REGISTRY Version 2\n[Control Panel\\\\Desktop]\n\"LogPixels\"=dword:00000078\n";
        assert_eq!(read_dpi_from_registry(registry), Some(120));
        let timestamped = "[Control Panel\\\\Desktop] 1577836800\n\"LogPixels\"=dword:00000078";
        assert_eq!(read_dpi_from_registry(timestamped), Some(120));
        assert_eq!(
            read_dpi_from_registry("[Other]\\n\"LogPixels\"=dword:00000078"),
            None
        );
    }

    #[test]
    fn reads_two_realistic_120_dpi_entries_even_with_different_headers() {
        // Regression: user's 2026-10-26 prefix had TWO 0x78 values in user.reg
        // while system.reg carried a separate system default of 0x60.
        let user_reg = "WINE REGISTRY Version 2\n\
            [Control Panel\\\\Desktop] 1789000\n\
            \"LogPixels\"=dword:00000078\n\
            [Software\\\\Wine\\\\Fonts] 1789000\n\
            \"LogPixels\"=dword:00000078\n";
        assert_eq!(read_dpi_from_registry(user_reg), Some(120));
        let alternate = "[Unknown\\\\DPI] 123\n\"LogPixels\"=dword:00000078\n\
            [Another] 456\n\"LogPixels\"=dword:00000078";
        assert_eq!(read_dpi_from_registry(alternate), Some(120));
    }

    #[test]
    fn prefers_user_desktop_dpi_when_registry_contains_a_different_value() {
        let registry = "[Software\\\\Wine\\\\Fonts]\n\"LogPixels\"=dword:00000060\n\
             [Control Panel\\\\Desktop] 12345\n\"LogPixels\"=dword:00000078";
        assert_eq!(read_dpi_from_registry(registry), Some(120));
    }

    #[test]
    fn does_not_guess_when_fallback_values_conflict_or_are_invalid() {
        let conflict = "[Unknown]\n\"LogPixels\"=dword:00000060\n\
             [Other]\n\"LogPixels\"=dword:00000078";
        assert_eq!(read_dpi_from_registry(conflict), None);
        assert_eq!(
            read_dpi_from_registry("[Control Panel\\\\Desktop]\n\"LogPixels\"=dword:BAD!"),
            None
        );
        assert_eq!(
            read_dpi_from_registry("[Other]\n\"SomeOtherValue\"=dword:00000078"),
            None
        );
    }

    #[test]
    fn dpi_options_are_whitelisted() {
        assert_eq!(DEFAULT_DPI, 120);
        assert!(DPI_OPTIONS.contains(&120));
        assert!(!DPI_OPTIONS.contains(&999));
    }
}
