use std::ffi::OsString;
use std::path::PathBuf;

use anyhow::{Context, Result};

#[cfg(not(all(debug_assertions, feature = "development")))]
pub(crate) fn user_state_dir() -> Result<PathBuf> {
    resolve_user_state_dir(std::env::var_os("XDG_STATE_HOME"), std::env::home_dir)
}

fn resolve_user_state_dir(
    xdg_state_home: Option<OsString>,
    home: impl FnOnce() -> Option<PathBuf>,
) -> Result<PathBuf> {
    match xdg_state_home.map(PathBuf::from) {
        Some(path) if path.is_absolute() => Ok(path),
        _ => {
            let home = home()
                .filter(|path| path.is_absolute())
                .context("Cannot resolve user state directory: XDG_STATE_HOME is unset or invalid and no absolute home directory could be determined.")?;
            Ok(home.join(".local").join("state"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::os::unix::ffi::OsStringExt;

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
        let path = resolve_user_state_dir(xdg.map(OsString::from), || {
            Some(PathBuf::from("/srv/users/alice"))
        })
        .unwrap();

        assert_eq!(path, PathBuf::from("/srv/users/alice/.local/state"));
    }

    #[rstest]
    fn missing_or_invalid_home_returns_error_when_fallback_is_needed(
        #[values(None, Some(""), Some("relative/state"))] xdg: Option<&str>,
        #[values(None, Some(""), Some("relative/home"))] home: Option<&str>,
    ) {
        let error = resolve_user_state_dir(xdg.map(OsString::from), || home.map(PathBuf::from))
            .unwrap_err();

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
        let path =
            resolve_user_state_dir(xdg.map(|bytes| OsString::from_vec(bytes.to_vec())), || {
                home.map(|bytes| PathBuf::from(OsString::from_vec(bytes.to_vec())))
            })
            .unwrap();

        assert_eq!(path, PathBuf::from(OsString::from_vec(expected.to_vec())));
    }
}
