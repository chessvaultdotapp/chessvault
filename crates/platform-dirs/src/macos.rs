//! macOS Library defaults with XDG overrides for state and shared data.

use std::ffi::OsString;
use std::path::PathBuf;

#[cfg(any(not(all(debug_assertions, feature = "development")), test))]
use anyhow::{Context, Result};

#[cfg(all(target_os = "macos", not(all(debug_assertions, feature = "development"))))]
pub(crate) fn user_state_dir() -> Result<PathBuf> {
    resolve_user_dir(
        std::env::var_os("XDG_STATE_HOME"),
        std::env::home_dir,
        "Application Support",
    )
}

#[cfg(all(target_os = "macos", not(all(debug_assertions, feature = "development"))))]
pub(crate) fn user_share_dir() -> Result<PathBuf> {
    resolve_user_dir(
        std::env::var_os("XDG_DATA_HOME"),
        std::env::home_dir,
        "Application Support",
    )
}

#[cfg(all(target_os = "macos", not(all(debug_assertions, feature = "development"))))]
pub(crate) fn user_log_dir() -> Result<PathBuf> {
    resolve_user_dir(None, std::env::home_dir, "Logs")
}

#[cfg(target_os = "macos")]
pub(crate) fn system_share_dirs() -> Vec<PathBuf> {
    resolve_system_share_dirs(std::env::var_os("XDG_DATA_DIRS"))
}

fn resolve_system_share_dirs(xdg: Option<OsString>) -> Vec<PathBuf> {
    match xdg.filter(|value| !value.is_empty()) {
        Some(value) => std::env::split_paths(&value)
            .filter(|path| path.is_absolute())
            .collect(),
        None => vec![PathBuf::from("/Library/Application Support")],
    }
}

#[cfg(any(not(all(debug_assertions, feature = "development")), test))]
fn resolve_user_dir(xdg: Option<OsString>, home: impl FnOnce() -> Option<PathBuf>, directory: &str) -> Result<PathBuf> {
    if let Some(path) = xdg.map(PathBuf::from).filter(|path| path.is_absolute()) {
        return Ok(path);
    }
    let home = home().filter(|path| path.is_absolute()).with_context(|| {
        format!("Cannot resolve user {directory} directory: no absolute home directory could be determined")
    })?;
    Ok(home.join("Library").join(directory))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn user_directories() {
        for directory in ["Application Support", "Logs"] {
            for xdg in [None, Some(OsString::new()), Some(OsString::from("relative"))] {
                assert_eq!(
                    resolve_user_dir(xdg.clone(), || Some(PathBuf::from("/Users/player")), directory).unwrap(),
                    PathBuf::from("/Users/player/Library").join(directory)
                );
                for home in [None, Some(PathBuf::new()), Some(PathBuf::from("relative"))] {
                    assert!(resolve_user_dir(xdg.clone(), || home, directory).is_err());
                }
            }
        }
    }

    #[test]
    fn absolute_override_does_not_look_up_home() {
        assert_eq!(
            resolve_user_dir(
                Some(OsString::from("/custom data ")),
                || panic!("home lookup"),
                "Application Support"
            )
            .unwrap(),
            PathBuf::from("/custom data ")
        );
    }

    #[test]
    fn preserves_non_unicode_paths() {
        let path = PathBuf::from(OsString::from_vec(b"/Users/player-\xff".to_vec()));
        assert_eq!(
            resolve_user_dir(None, || Some(path.clone()), "Logs").unwrap(),
            path.join("Library/Logs")
        );
        assert_eq!(
            resolve_user_dir(Some(path.clone().into_os_string()), || None, "Application Support").unwrap(),
            path
        );
        assert_eq!(
            resolve_system_share_dirs(Some(path.clone().into_os_string())),
            vec![path]
        );
    }

    #[test]
    fn system_directories() {
        for value in [None, Some(OsString::new())] {
            assert_eq!(
                resolve_system_share_dirs(value),
                vec![PathBuf::from("/Library/Application Support")]
            );
        }
        assert_eq!(
            resolve_system_share_dirs(Some(OsString::from(":relative:/second:/first:/second:/space :"))),
            ["/second", "/first", "/second", "/space "].map(PathBuf::from).to_vec()
        );
        assert!(resolve_system_share_dirs(Some(OsString::from(":relative:"))).is_empty());
    }
}
