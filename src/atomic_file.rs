//! Small transactional file writer for authored project/editor documents.
//!
//! `std::fs::rename(temp, destination)` does not replace an existing file on
//! every supported platform. Havenwild therefore stages beside the destination,
//! moves the old file to a same-directory backup, promotes the staged file, and
//! restores the backup if promotion fails.

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let name = path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "havenwild-document".to_owned());
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    path.with_file_name(format!(
        ".{name}.{}.{}.{}",
        std::process::id(),
        nonce,
        suffix
    ))
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let staged = sibling(path, "tmp");
    let backup = sibling(path, "bak");

    let stage_result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
            .map_err(|error| error.to_string())?;
        file.write_all(bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        Ok(())
    })();
    if let Err(error) = stage_result {
        let _ = fs::remove_file(&staged);
        return Err(format!("failed to stage {}: {error}", path.display()));
    }

    let had_original = path.is_file();
    if had_original {
        if let Err(error) = fs::rename(path, &backup) {
            let _ = fs::remove_file(&staged);
            return Err(format!(
                "failed to protect existing {} before save: {error}",
                path.display()
            ));
        }
    }

    match fs::rename(&staged, path) {
        Ok(()) => {
            if had_original {
                let _ = fs::remove_file(&backup);
            }
            Ok(())
        }
        Err(promote_error) => {
            let _ = fs::remove_file(&staged);
            if had_original {
                match fs::rename(&backup, path) {
                    Ok(()) => Err(format!(
                        "failed to promote new {}: {promote_error}; previous file restored",
                        path.display()
                    )),
                    Err(restore_error) => Err(format!(
                        "failed to promote new {}: {promote_error}; backup restore also failed: {restore_error}; backup remains at {}",
                        path.display(),
                        backup.display()
                    )),
                }
            } else {
                Err(format!(
                    "failed to promote new {}: {promote_error}",
                    path.display()
                ))
            }
        }
    }
}

/// Create an immutable first v2 draft without replacing a competing existing
/// document. Stage/sync in the same directory; hard-link atomically with
/// CREATE_NEW-style semantics, then remove the temporary directory entry.
/// On filesystems without hard-link support, fail safely without touching v1.
pub fn write_atomic_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let staged = sibling(path, "new.tmp");
    let result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
            .map_err(|e| format!("Cannot stage new draft: {e}"))?;
        file.write_all(bytes)
            .map_err(|e| format!("Cannot write new draft: {e}"))?;
        file.sync_all()
            .map_err(|e| format!("Cannot sync new draft: {e}"))?;
        drop(file);
        fs::hard_link(&staged, path).map_err(|e| format!("Cannot publish new draft without overwrite (possibly existing destination or unsupported hard links): {e}"))?;
        Ok(())
    })();
    let _ = fs::remove_file(&staged);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transactional_write_can_replace_existing_file() {
        let root = std::env::temp_dir().join(format!(
            "havenwild-atomic-write-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let path = root.join("scene.json");
        write_atomic(&path, b"first\n").expect("initial write");
        write_atomic(&path, b"second\n").expect("replacement write");
        assert_eq!(fs::read(&path).expect("read result"), b"second\n");
        let protected = root.join("never_overwrite.json");
        write_atomic_new(&protected, b"first").expect("atomic create");
        assert!(write_atomic_new(&protected, b"replacement").is_err());
        assert_eq!(fs::read(&protected).unwrap(), b"first");
        fs::remove_dir_all(root).expect("remove test directory");
    }
}
