//! Read-only Windows executable format inspection.
//! We do not execute, decompress, or trust the inspected binary.

use std::fmt;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const MSI_MAGIC: [u8; 8] = [0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramKind {
    ExeX86,
    ExeX64,
    Msi,
}

impl fmt::Display for ProgramKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExeX86 => write!(f, "Windows PE executable · x86 (32-bit)"),
            Self::ExeX64 => write!(f, "Windows PE executable · x64 (64-bit)"),
            Self::Msi => write!(f, "Windows MSI installer · architecture not inferred"),
        }
    }
}

pub fn inspect(path: &Path) -> Result<ProgramKind, String> {
    if !path.is_file() {
        return Err(format!("Not a regular file: {}", path.display()));
    }
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let mut file = File::open(path).map_err(|e| format!("Cannot open file: {e}"))?;
    let size = file.metadata().map_err(|e| e.to_string())?.len();

    if extension == "msi" {
        let mut magic = [0u8; 8];
        file.read_exact(&mut magic)
            .map_err(|_| "MSI file is too short".to_string())?;
        return if magic == MSI_MAGIC {
            Ok(ProgramKind::Msi)
        } else {
            Err("The .msi file does not have a valid MSI compound-file header".into())
        };
    }
    if extension != "exe" {
        return Err("Choose a Windows .exe or .msi file".into());
    }
    if size < 64 {
        return Err("EXE is too short to contain a PE header".into());
    }

    let mut dos = [0u8; 64];
    file.read_exact(&mut dos)
        .map_err(|e| format!("Cannot read DOS header: {e}"))?;
    if &dos[0..2] != b"MZ" {
        return Err("Not a Windows PE executable (missing MZ signature)".into());
    }
    let offset = u32::from_le_bytes(dos[0x3c..0x40].try_into().unwrap()) as u64;
    // Only a small fixed region is read; oversized or corrupted pointers are rejected.
    if offset < 64 || offset.checked_add(26).is_none_or(|end| end > size) {
        return Err("Invalid Windows PE header offset".into());
    }
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| format!("Cannot seek to PE header: {e}"))?;
    let mut header = [0u8; 26];
    file.read_exact(&mut header)
        .map_err(|e| format!("Cannot read PE header: {e}"))?;
    if &header[..4] != b"PE\0\0" {
        return Err("Invalid Windows PE signature".into());
    }
    let machine = u16::from_le_bytes([header[4], header[5]]);
    let optional_size = u16::from_le_bytes([header[20], header[21]]);
    let magic = u16::from_le_bytes([header[24], header[25]]);
    if optional_size < 2 {
        return Err("Missing Windows PE optional header".into());
    }
    match (machine, magic) {
        (0x014c, 0x010b) => Ok(ProgramKind::ExeX86),
        (0x8664, 0x020b) => Ok(ProgramKind::ExeX64),
        _ => Err(format!(
            "Unsupported PE architecture: machine 0x{machine:04x}, magic 0x{magic:04x}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(name: &str, content: &[u8]) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("zexez-pe-{}-{nonce}-{name}", std::process::id()));
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    fn fake_pe(machine: u16, magic: u16) -> Vec<u8> {
        let mut bytes = vec![0; 256];
        bytes[0..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&128u32.to_le_bytes());
        bytes[128..132].copy_from_slice(b"PE\0\0");
        bytes[132..134].copy_from_slice(&machine.to_le_bytes());
        bytes[148..150].copy_from_slice(&96u16.to_le_bytes());
        bytes[152..154].copy_from_slice(&magic.to_le_bytes());
        bytes
    }

    #[test]
    fn detects_x86() {
        let path = temp_file("setup.exe", &fake_pe(0x014c, 0x010b));
        assert_eq!(inspect(&path).unwrap(), ProgramKind::ExeX86);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn detects_x64() {
        let path = temp_file("setup.EXE", &fake_pe(0x8664, 0x020b));
        assert_eq!(inspect(&path).unwrap(), ProgramKind::ExeX64);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_arm64_and_incorrect_optional_header() {
        let path = temp_file("arm.exe", &fake_pe(0xaa64, 0x020b));
        assert!(inspect(&path).unwrap_err().contains("Unsupported"));
        std::fs::remove_file(path).unwrap();
        let path = temp_file("bad.exe", &fake_pe(0x014c, 0x020b));
        assert!(inspect(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_broken_header_and_renamed_text() {
        let path = temp_file("broken.exe", b"Hello World");
        assert!(inspect(&path).is_err());
        std::fs::remove_file(path).unwrap();
        let mut bytes = fake_pe(0x014c, 0x010b);
        bytes[0x3c..0x40].copy_from_slice(&u32::MAX.to_le_bytes());
        let path = temp_file("offset.exe", &bytes);
        assert!(inspect(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn verifies_msi_signature_without_guessing_architecture() {
        let path = temp_file("installer.msi", &MSI_MAGIC);
        assert_eq!(inspect(&path).unwrap(), ProgramKind::Msi);
        std::fs::remove_file(path).unwrap();
        let path = temp_file("fake.msi", b"not an msi file");
        assert!(inspect(&path).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
