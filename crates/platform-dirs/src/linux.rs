//! Linux user directory resolution following the XDG base directory policy.

use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{Context, Result};

/// Resolves the state directory using the process environment and OS home lookup.
#[cfg(not(all(debug_assertions, feature = "development")))]
pub(crate) fn user_state_dir() -> Result<PathBuf> {
    resolve_user_state_dir(std::env::var_os("XDG_STATE_HOME"), std::env::home_dir)
}

/// Resolves the data directory using the process environment and OS home lookup.
#[cfg(not(all(debug_assertions, feature = "development")))]
pub(crate) fn user_share_dir() -> Result<PathBuf> {
    resolve_user_share_dir(std::env::var_os("XDG_DATA_HOME"), std::env::home_dir)
}

fn resolve_user_share_dir(xdg: Option<OsString>, home: impl FnOnce() -> Option<PathBuf>) -> Result<PathBuf> {
    resolve_user_dir(xdg, home, "share", "XDG_DATA_HOME")
}

/// Prefers an absolute XDG state path, otherwise appends `.local/state` to home.
///
/// The home lookup is lazy and runs only when the XDG value is missing or not
/// absolute. Returns an error if the fallback home is missing or not absolute.
/// OS-native path values preserve non-Unicode names; injected inputs let tests
/// exercise the policy without changing the process environment.
fn resolve_user_state_dir(xdg_state_home: Option<OsString>, home: impl FnOnce() -> Option<PathBuf>) -> Result<PathBuf> {
    resolve_user_dir(xdg_state_home, home, "state", "XDG_STATE_HOME")
}

fn resolve_user_dir(
    xdg: Option<OsString>,
    home: impl FnOnce() -> Option<PathBuf>,
    directory: &str,
    variable: &str,
) -> Result<PathBuf> {
    match xdg.map(PathBuf::from) {
        Some(path) if path.is_absolute() => return Ok(path),
        Some(path) => tracing::warn!(
            variable,
            path = ?path,
            "XDG variable is not an absolute path; ignoring it as required by XDG and falling back to the default directory"
        ),
        None => tracing::info!(
            variable,
            "XDG variable is not set; falling back to the default directory"
        ),
    }

    let home = home().filter(|path| path.is_absolute()).with_context(|| {
        format!(
            "Cannot resolve user {directory} directory: \
             {variable} is unset or invalid and no absolute home directory could be determined."
        )
    })?;

    Ok(home.join(".local").join(directory))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn share_directory_resolution() {
        let non_unicode = OsString::from_vec(b"/data/\xff".to_vec());
        assert_eq!(
            resolve_user_share_dir(Some(non_unicode.clone()), || panic!("unexpected home lookup")).unwrap(),
            PathBuf::from(non_unicode.clone())
        );
        for xdg in [None, Some(""), Some("relative/data")] {
            assert_eq!(
                resolve_user_share_dir(xdg.map(OsString::from), || Some(PathBuf::from(non_unicode.clone()))).unwrap(),
                PathBuf::from(non_unicode.clone()).join(".local/share")
            );
            for home in [None, Some(""), Some("relative/home")] {
                let error = resolve_user_share_dir(xdg.map(OsString::from), || home.map(PathBuf::from)).unwrap_err();
                assert!(error.to_string().contains("XDG_DATA_HOME"));
            }
        }
    }

    #[test]
    fn absolute_xdg_state_home_does_not_need_home() {
        let path = resolve_user_state_dir(Some(OsString::from("/custom/state")), || {
            panic!("Home directory lookup should not run when XDG_STATE_HOME is valid")
        })
        .unwrap();

        assert_eq!(path, PathBuf::from("/custom/state"));
    }

    #[rstest]
    #[case::missing(None)]
    #[case::empty(Some(""))]
    #[case::relative(Some("relative/state"))]
    fn missing_or_invalid_xdg_state_home_falls_back_to_home(#[case] xdg: Option<&str>) {
        let path = resolve_user_state_dir(xdg.map(OsString::from), || Some(PathBuf::from("/srv/users/alice"))).unwrap();

        assert_eq!(path, PathBuf::from("/srv/users/alice/.local/state"));
    }

    #[rstest]
    fn missing_or_invalid_home_returns_error_when_fallback_is_needed(
        #[values(None, Some(""), Some("relative/state"))] xdg: Option<&str>,
        #[values(None, Some(""), Some("relative/home"))] home: Option<&str>,
    ) {
        let error = resolve_user_state_dir(xdg.map(OsString::from), || home.map(PathBuf::from)).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("no absolute home directory could be determined")
        );
    }

    #[rstest]
    #[case::xdg(Some(b"/srv/user-\xff".as_slice()), None, b"/srv/user-\xff")]
    #[case::home_fallback(None, Some(b"/srv/user-\xff".as_slice()), b"/srv/user-\xff/.local/state")]
    fn non_unicode_paths_are_preserved(
        #[case] xdg: Option<&[u8]>,
        #[case] home: Option<&[u8]>,
        #[case] expected: &[u8],
    ) {
        let path = resolve_user_state_dir(xdg.map(|bytes| OsString::from_vec(bytes.to_vec())), || {
            home.map(|bytes| PathBuf::from(OsString::from_vec(bytes.to_vec())))
        })
        .unwrap();

        assert_eq!(path, PathBuf::from(OsString::from_vec(expected.to_vec())));
    }
}
