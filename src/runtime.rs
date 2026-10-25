//! Read-only Wine runtime discovery for Ubuntu and other Linux distributions.
//!
//! `wine --version` and `winetricks --version` are the only external commands
//! executed here. We intentionally do not run `wineboot` or touch ~/.wine.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct RuntimeReport {
    wine_version: Option<String>,
    server_path: Option<PathBuf>,
    winetricks_version: Option<String>,
    wine32_installed: Option<bool>,
    wine64_installed: Option<bool>,
}

impl RuntimeReport {
    pub fn detect() -> Self {
        Self {
            wine_version: command_version("wine"),
            server_path: detect_wineserver(),
            winetricks_version: command_version("winetricks"),
            wine32_installed: debian_package_installed("wine32:i386"),
            wine64_installed: debian_package_installed("wine64"),
        }
    }

    pub fn wine_summary(&self) -> String {
        self.wine_version
            .as_ref()
            .map(|v| format!("Installed · {v}"))
            .unwrap_or_else(|| "Not detected in PATH".to_string())
    }

    pub fn server_summary(&self) -> String {
        self.server_path
            .as_ref()
            .map(|p| format!("Found · {}", p.display()))
            .unwrap_or_else(|| "Not found in PATH or known Linux locations".to_string())
    }

    pub fn arch_summary(&self) -> String {
        format!(
            "x86 (32-bit): {} · x64 (64-bit): {}",
            package_label(self.wine32_installed),
            package_label(self.wine64_installed),
        )
    }

    pub fn winetricks_summary(&self) -> String {
        self.winetricks_version
            .as_ref()
            .map(|v| format!("Installed · {v}"))
            .unwrap_or_else(|| "Not detected in PATH".to_string())
    }
}

fn package_label(installed: Option<bool>) -> &'static str {
    match installed {
        Some(true) => "installed",
        Some(false) => "not installed",
        None => "unknown (not a Debian-based system)",
    }
}

fn command_version(command: &str) -> Option<String> {
    let output = Command::new(command).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    first_version_line(&stdout).or_else(|| first_version_line(&stderr))
}

fn first_version_line(output: &str) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .find(|line| line.chars().any(|c| c.is_ascii_digit()))
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn debian_package_installed(package: &str) -> Option<bool> {
    // Avoid attempting dpkg-query on distributions without dpkg.
    if find_on_path("dpkg-query").is_none() {
        return None;
    }
    let output = Command::new("dpkg-query")
        .args(["-W", "-f=${db:Status-Abbrev}", package])
        .output()
        .ok()?;
    if !output.status.success() {
        return Some(false);
    }
    Some(String::from_utf8_lossy(&output.stdout).starts_with("ii"))
}

fn detect_wineserver() -> Option<PathBuf> {
    if let Some(path) = env::var_os("WINESERVER") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    if let Some(found) = find_on_path("wineserver") {
        return Some(found);
    }

    // Ubuntu 26.04 / Debian Wine 10 deliberately keep wineserver under /usr/lib.
    let standard_locations = [
        "/usr/lib/x86_64-linux-gnu/wine/wineserver",
        "/usr/lib/i386-linux-gnu/wine/wineserver",
        "/usr/lib/wine/wineserver",
    ];
    standard_locations
        .iter()
        .map(PathBuf::from)
        .find(|p| p.is_file())
}

fn find_on_path(filename: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path)
        .map(|dir| dir.join(filename))
        .find(|file| file.is_file() && is_executable(file))
}

#[cfg(unix)]
fn is_executable(file: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    file.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(file: &Path) -> bool {
    file.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_version_line() {
        assert_eq!(
            first_version_line("Executing cd /usr/bin\n20250102 - sha256sum: abc"),
            Some("20250102 - sha256sum: abc".to_string())
        );
        assert_eq!(first_version_line("no version here"), None);
    }

    #[test]
    fn formats_package_state() {
        assert_eq!(package_label(Some(true)), "installed");
        assert_eq!(package_label(Some(false)), "not installed");
        assert!(package_label(None).starts_with("unknown"));
    }

    #[test]
    fn missing_wine_report_has_clear_messages() {
        let r = RuntimeReport {
            wine_version: None,
            server_path: None,
            winetricks_version: None,
            wine32_installed: None,
            wine64_installed: None,
        };
        assert!(r.wine_summary().contains("Not detected"));
        assert!(r.server_summary().contains("Not found"));
    }
}
