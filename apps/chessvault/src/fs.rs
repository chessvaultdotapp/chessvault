use std::path::PathBuf;

use anyhow::{Context, Result};

pub(crate) fn window_recreate_info_filepath() -> Result<PathBuf> {
    resolve_window_recreate_info_filepath(application_runtime::fs::application_state_dir())
}

fn resolve_window_recreate_info_filepath(state_dir: Result<PathBuf>) -> Result<PathBuf> {
    state_dir
        .context("Failed to resolve application state directory")
        .map(|path| path.join("recreate"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recreate_filename_is_appended_to_application_state_directory() {
        let state_dir = PathBuf::from("custom").join("state").join("chessvault");
        let path = resolve_window_recreate_info_filepath(Ok(state_dir.clone())).unwrap();

        assert_eq!(path, state_dir.join("recreate"));
    }

    #[test]
    fn lookup_error_retains_its_source_and_adds_context() {
        let source = std::io::Error::new(std::io::ErrorKind::NotFound, "no state directory");
        let error = resolve_window_recreate_info_filepath(Err(source.into())).unwrap_err();

        assert_eq!(error.to_string(), "Failed to resolve application state directory");
        let source = error.downcast_ref::<std::io::Error>().unwrap();
        assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
        assert_eq!(source.to_string(), "no state directory");
    }

    #[cfg(unix)]
    #[test]
    fn non_unicode_paths_are_preserved() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let state_dir = PathBuf::from(OsString::from_vec(b"/srv/user-\xff/state/chessvault".to_vec()));
        let path = resolve_window_recreate_info_filepath(Ok(state_dir)).unwrap();

        assert_eq!(
            path,
            PathBuf::from(OsString::from_vec(b"/srv/user-\xff/state/chessvault/recreate".to_vec()))
        );
    }
}
