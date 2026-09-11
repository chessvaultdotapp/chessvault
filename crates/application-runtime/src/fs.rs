use std::path::PathBuf;

use anyhow::{Context, Result};

pub fn application_state_dir() -> Result<PathBuf> {
    resolve_application_state_dir(platform_dirs::user_state_dir())
}

fn resolve_application_state_dir(user_state_dir: Result<PathBuf>) -> Result<PathBuf> {
    match user_state_dir.context("Failed to get user state directory") {
        Ok(path) => Ok(path.join("chessvault")),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_name_is_appended_to_user_state_directory() {
        let user_state_dir = PathBuf::from("custom").join("state");
        let path = resolve_application_state_dir(Ok(user_state_dir.clone())).unwrap();

        assert_eq!(path, user_state_dir.join("chessvault"));
    }

    #[test]
    fn lookup_error_retains_its_source_and_adds_context() {
        let source = std::io::Error::new(std::io::ErrorKind::NotFound, "no home directory");
        let error = resolve_application_state_dir(Err(source.into())).unwrap_err();

        assert_eq!(error.to_string(), "Failed to get user state directory");
        let source = error.downcast_ref::<std::io::Error>().unwrap();
        assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        assert_eq!(source.to_string(), "no home directory");
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_user_state_directory_is_preserved() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let user_state_dir = PathBuf::from(OsString::from_vec(b"/srv/user-\xff/state".to_vec()));
        let path = resolve_application_state_dir(Ok(user_state_dir)).unwrap();

        assert_eq!(
            path,
            PathBuf::from(OsString::from_vec(
                b"/srv/user-\xff/state/chessvault".to_vec()
            ))
        );
    }
}
