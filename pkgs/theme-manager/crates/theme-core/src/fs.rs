use anyhow::{Context, Result};
use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

pub fn contents_changed(path: &Path, contents: &str) -> Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) if !metadata.is_file() => {
            anyhow::bail!("output {} is not a regular file", path.display())
        }
        Ok(_) => fs::read(path)
            .map(|current| current != contents.as_bytes())
            .with_context(|| format!("failed to read output {}", path.display())),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(true),
        Err(err) => {
            Err(err).with_context(|| format!("failed to inspect output {}", path.display()))
        }
    }
}

pub fn write_if_changed(path: &Path, contents: &str) -> Result<bool> {
    if !contents_changed(path, contents)? {
        return Ok(false);
    }

    let destination = write_destination(path)?;

    if let Some(parent) = destination
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let file_name = destination
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("output");
    let (tmp, mut file) = create_temp_file(&destination, file_name)?;
    if let Err(err) = file.write_all(contents.as_bytes()) {
        drop(file);
        let _ = fs::remove_file(&tmp);
        return Err(err)
            .with_context(|| format!("failed to write temporary file {}", tmp.display()));
    }
    drop(file);
    if let Ok(metadata) = fs::metadata(&destination)
        && let Err(err) = fs::set_permissions(&tmp, metadata.permissions())
    {
        let _ = fs::remove_file(&tmp);
        return Err(err).with_context(|| {
            format!(
                "failed to preserve permissions for {}",
                destination.display()
            )
        });
    }
    if let Err(err) = fs::rename(&tmp, &destination) {
        let _ = fs::remove_file(&tmp);
        return Err(err).with_context(|| format!("failed to replace {}", destination.display()));
    }
    Ok(true)
}

fn create_temp_file(destination: &Path, file_name: &str) -> Result<(PathBuf, fs::File)> {
    for _ in 0..100 {
        let sequence = NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed);
        let tmp = destination.with_file_name(format!(
            ".{file_name}.{}.{sequence}.tmp",
            std::process::id()
        ));
        match OpenOptions::new().write(true).create_new(true).open(&tmp) {
            Ok(file) => return Ok((tmp, file)),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("failed to create temporary file {}", tmp.display()));
            }
        }
    }
    anyhow::bail!(
        "failed to create a unique temporary file beside {}",
        destination.display()
    )
}

fn write_destination(path: &Path) -> Result<PathBuf> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => fs::canonicalize(path)
            .with_context(|| format!("failed to resolve output symlink {}", path.display())),
        Ok(_) => Ok(path.to_path_buf()),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(path.to_path_buf()),
        Err(err) => {
            Err(err).with_context(|| format!("failed to inspect output {}", path.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    #[test]
    fn atomic_write_preserves_output_symlinks() {
        use super::write_if_changed;
        use std::{fs, os::unix::fs::symlink};

        let root =
            std::env::temp_dir().join(format!("theme-manager-write-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let target = root.join("target.conf");
        let link = root.join("output.conf");
        fs::write(&target, "old").unwrap();
        symlink(&target, &link).unwrap();

        assert!(write_if_changed(&link, "new").unwrap());
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_to_string(target).unwrap(), "new");

        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_preserves_output_permissions() {
        use super::write_if_changed;
        use std::{fs, os::unix::fs::PermissionsExt};

        let root = std::env::temp_dir().join(format!(
            "theme-manager-permissions-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let output = root.join("private.conf");
        fs::write(&output, "old").unwrap();
        fs::set_permissions(&output, fs::Permissions::from_mode(0o600)).unwrap();

        assert!(write_if_changed(&output, "new").unwrap());
        assert_eq!(
            fs::metadata(&output).unwrap().permissions().mode() & 0o777,
            0o600
        );

        fs::remove_dir_all(root).unwrap();
    }
}
