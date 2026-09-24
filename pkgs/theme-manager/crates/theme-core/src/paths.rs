use anyhow::{Context, Result, bail};
use std::{
    env, fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

fn home() -> Result<PathBuf> {
    env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .context("HOME is not set")
}

pub fn xdg_config_home() -> Result<PathBuf> {
    match env::var_os("XDG_CONFIG_HOME").filter(|value| !value.is_empty()) {
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(home()?.join(".config")),
    }
}

pub fn xdg_cache_home() -> Result<PathBuf> {
    match env::var_os("XDG_CACHE_HOME").filter(|value| !value.is_empty()) {
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(home()?.join(".cache")),
    }
}

pub fn xdg_state_home() -> Result<PathBuf> {
    match env::var_os("XDG_STATE_HOME").filter(|value| !value.is_empty()) {
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(home()?.join(".local/state")),
    }
}

pub fn xdg_data_home() -> Result<PathBuf> {
    match env::var_os("XDG_DATA_HOME").filter(|value| !value.is_empty()) {
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(home()?.join(".local/share")),
    }
}

pub fn default_config_path() -> Result<PathBuf> {
    let user_config = xdg_config_home()?.join("theme-manager/config.toml");
    if user_config.is_file() {
        return Ok(user_config);
    }

    if let Some(data_dir) = env::var_os("THEME_MANAGER_DATA_DIR").filter(|value| !value.is_empty())
    {
        let bundled_config = PathBuf::from(data_dir).join("config.toml");
        if bundled_config.is_file() {
            return Ok(bundled_config);
        }
    }

    // Keep the conventional user path in the eventual error when neither a
    // user config nor packaged defaults are available (for example, in an
    // unconfigured development build).
    Ok(user_config)
}

pub fn expand_path(input: &str) -> Result<PathBuf> {
    if input.is_empty() {
        bail!("path may not be empty");
    }

    for prefix in [
        "$XDG_CONFIG_HOME",
        "$XDG_CACHE_HOME",
        "$XDG_STATE_HOME",
        "$XDG_DATA_HOME",
    ] {
        let rest = if input == prefix {
            Some(None)
        } else {
            input.strip_prefix(&format!("{prefix}/")).map(Some)
        };
        let Some(rest) = rest else { continue };

        let base = match prefix {
            "$XDG_CONFIG_HOME" => xdg_config_home()?,
            "$XDG_CACHE_HOME" => xdg_cache_home()?,
            "$XDG_STATE_HOME" => xdg_state_home()?,
            "$XDG_DATA_HOME" => xdg_data_home()?,
            _ => unreachable!(),
        };
        return Ok(match rest {
            Some(rest) => base.join(rest),
            None => base,
        });
    }

    let data_dir_rest = if input == "$THEME_MANAGER_DATA_DIR" {
        Some(None)
    } else {
        input.strip_prefix("$THEME_MANAGER_DATA_DIR/").map(Some)
    };
    if let Some(rest) = data_dir_rest {
        let base = env::var_os("THEME_MANAGER_DATA_DIR")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .context("THEME_MANAGER_DATA_DIR is not set")?;
        return Ok(match rest {
            Some(rest) => base.join(rest),
            None => base,
        });
    }

    if input == "~" {
        return home();
    }
    if let Some(rest) = input.strip_prefix("~/") {
        return Ok(home()?.join(rest));
    }

    Ok(PathBuf::from(input))
}

/// Return a stable collision key without requiring the output file to exist.
///
/// Components are resolved from left to right so parent components after a
/// directory symlink keep their filesystem meaning.
pub fn canonical_output_path(path: &Path) -> Result<PathBuf> {
    let mut resolved = if path.is_absolute() {
        PathBuf::new()
    } else {
        fs::canonicalize(env::current_dir().context("failed to resolve current directory")?)
            .context("failed to canonicalize current directory")?
    };

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            Component::Prefix(_) | Component::RootDir => {
                resolved.push(component.as_os_str());
            }
            Component::Normal(_) => {
                let candidate = resolved.join(component.as_os_str());
                match fs::symlink_metadata(&candidate) {
                    Ok(_) => {
                        resolved = fs::canonicalize(&candidate).with_context(|| {
                            format!("failed to resolve output path {}", candidate.display())
                        })?;
                    }
                    Err(err) if err.kind() == ErrorKind::NotFound => {
                        resolved = candidate;
                    }
                    Err(err) => {
                        return Err(err).with_context(|| {
                            format!("failed to inspect output path {}", candidate.display())
                        });
                    }
                }
            }
        }
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::canonical_output_path;
    use std::{
        fs,
        path::Path,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn output_paths_are_lexically_normalized() {
        let actual = canonical_output_path(Path::new("alpha/../output.conf")).unwrap();
        let expected = std::fs::canonicalize(std::env::current_dir().unwrap())
            .unwrap()
            .join("output.conf");
        assert_eq!(actual, expected);
    }

    #[cfg(unix)]
    #[test]
    fn output_paths_resolve_symlinks_before_parent_components() {
        use std::os::unix::fs::symlink;

        let id = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "theme-manager-path-test-{}-{id}",
            std::process::id()
        ));
        let actual = root.join("actual");
        let nested = actual.join("nested");
        fs::create_dir_all(&nested).unwrap();
        let link = root.join("link");
        symlink(&nested, &link).unwrap();

        let resolved = canonical_output_path(&link.join("../output.conf")).unwrap();
        assert_eq!(resolved, actual.join("output.conf"));

        fs::remove_dir_all(root).unwrap();
    }
}
