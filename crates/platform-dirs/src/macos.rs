//! Native macOS directories. XDG environment variables do not affect this policy.

use std::path::PathBuf;

#[cfg(any(not(all(debug_assertions, feature = "development")), test))]
use anyhow::{Context, Result};

#[cfg(all(target_os = "macos", not(all(debug_assertions, feature = "development"))))]
pub(crate) fn user_share_dir() -> Result<PathBuf> {
    resolve_user_dir(std::env::home_dir(), "Application Support")
}

#[cfg(all(target_os = "macos", not(all(debug_assertions, feature = "development"))))]
pub(crate) fn user_log_dir() -> Result<PathBuf> {
    resolve_user_dir(std::env::home_dir(), "Logs")
}

pub(crate) fn system_share_dirs() -> Vec<PathBuf> {
    vec![PathBuf::from("/Library/Application Support")]
}

#[cfg(any(not(all(debug_assertions, feature = "development")), test))]
fn resolve_user_dir(home: Option<PathBuf>, directory: &str) -> Result<PathBuf> {
    let home = home.filter(|path| path.is_absolute()).with_context(|| {
        format!("Cannot resolve user {directory} directory: no absolute home directory could be determined")
    })?;
    Ok(home.join("Library").join(directory))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn user_directories() {
        for directory in ["Application Support", "Logs"] {
            assert_eq!(
                resolve_user_dir(Some(PathBuf::from("/Users/player")), directory).unwrap(),
                PathBuf::from("/Users/player/Library").join(directory)
            );
            for home in [None, Some(PathBuf::new()), Some(PathBuf::from("relative"))] {
                assert!(resolve_user_dir(home, directory).is_err());
            }
        }
    }

    #[test]
    fn preserves_non_unicode_home() {
        let home = PathBuf::from(OsString::from_vec(b"/Users/player-\xff".to_vec()));
        assert_eq!(
            resolve_user_dir(Some(home.clone()), "Logs").unwrap(),
            home.join("Library/Logs")
        );
    }

    #[test]
    fn system_directories_exclude_user_data() {
        assert_eq!(system_share_dirs(), vec![PathBuf::from("/Library/Application Support")]);
    }
}
