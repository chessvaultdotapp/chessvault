//! Platform-aware resolution of user state directories.
//!
//! Use [`user_state_dir`] to locate the base directory for persistent user state.
//! An optional application name selects a subdirectory beneath that base.
//! This crate only resolves paths: callers create directories as needed.
//!
//! # Features and platform support
//!
//! The default-enabled `development` feature makes builds with debug assertions
//! use `<current working directory>/.local/state` on every platform. Builds
//! without debug assertions, or with this feature disabled, use native platform
//! resolution, currently supported only on Linux.
//!
//! To use native resolution in debug builds, disable default features:
//!
//! ```toml
//! [dependencies]
//! platform-dirs = { version = "0.0.0", default-features = false }
//! ```

use std::path::PathBuf;

use anyhow::Result;

#[cfg(all(target_os = "linux", any(not(all(debug_assertions, feature = "development")), test)))]
mod linux;

/// Returns the absolute directory for persistent user state.
///
/// Pass `None` for the base directory or `Some("my-app")` to append an application
/// name. Names must be a single nonempty directory component: `.` and `..`,
/// slashes, backslashes, colons, and NUL bytes are rejected on every platform.
///
/// With debug assertions and the default-enabled `development` feature, returns
/// `<current working directory>/.local/state` on every platform.
///
/// Otherwise, uses native platform resolution. On Linux, an absolute
/// `XDG_STATE_HOME` takes precedence. If it is unset, empty, or relative, falls
/// back to `.local/state` beneath the home directory returned by
/// [`std::env::home_dir`], which must also be absolute. Non-Unicode paths are
/// preserved.
///
/// Does not create the directory or check whether it exists or is writable.
///
/// # Errors
///
/// Base-directory resolution runs before application-name validation, so resolution
/// errors take precedence over invalid-name errors.
///
/// Returns an error if:
/// - The application name is invalid.
/// - Development resolution cannot read the current working directory.
/// - Native Linux resolution has neither an absolute `XDG_STATE_HOME` nor an
///   absolute home directory.
/// - Native resolution is requested on an unsupported platform (anything other
///   than Linux).
///
/// # Examples
///
/// Resolve a path without creating any directories, handling resolution errors:
///
/// ```
/// match platform_dirs::user_state_dir(None) {
///     Ok(path) => {
///         assert!(path.is_absolute());
///         println!("User state directory: {}", path.display());
///     }
///     Err(error) => eprintln!("Cannot resolve user state directory: {error}"),
/// }
/// ```
///
/// Create an application-specific directory. This example is compiled but not
/// run because it writes to the filesystem:
///
/// ```no_run
/// let state_dir = platform_dirs::user_state_dir(Some("my-app"))?;
/// std::fs::create_dir_all(&state_dir)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn user_state_dir(app_name: Option<&str>) -> Result<PathBuf> {
    let mut path = base_state_dir()?;
    if let Some(name) = app_name {
        anyhow::ensure!(
            !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\', ':', '\0']),
            "Application name must be a single nonempty directory component"
        );

        path.push(name);
    }

    Ok(path)
}

fn base_state_dir() -> Result<PathBuf> {
    #[cfg(all(debug_assertions, feature = "development"))]
    {
        use anyhow::Context;

        Ok(std::env::current_dir()
            .context("Cannot resolve user state directory: current working directory is unavailable")?
            .join(".local")
            .join("state"))
    }

    #[cfg(all(not(all(debug_assertions, feature = "development")), target_os = "linux"))]
    {
        linux::user_state_dir()
    }

    #[cfg(all(not(all(debug_assertions, feature = "development")), not(target_os = "linux")))]
    {
        anyhow::bail!("User state directory is unsupported on this platform")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_application_name() {
        let base = base_state_dir();
        for name in [None, Some("chessvault"), Some("my app"), Some("šah")] {
            match &base {
                Ok(base) => {
                    let expected = name.map_or_else(|| base.clone(), |name| base.join(name));
                    assert_eq!(user_state_dir(name).unwrap(), expected);
                }
                Err(error) => assert_eq!(format!("{:#}", user_state_dir(name).unwrap_err()), format!("{error:#}")),
            }
        }
    }

    #[test]
    fn invalid_names_are_rejected_unless_base_resolution_fails() {
        let expected = match base_state_dir() {
            Ok(_) => "Application name must be a single nonempty directory component".to_owned(),
            Err(error) => format!("{error:#}"),
        };
        for name in [
            "",
            ".",
            "..",
            "/app",
            "a/b",
            "a\\b",
            "a/",
            "C:app",
            "C:\\app",
            "\\\\server\\app",
            "a\0b",
        ] {
            let error = user_state_dir(Some(name)).unwrap_err();
            assert_eq!(format!("{error:#}"), expected);
        }
    }

    #[cfg(all(debug_assertions, feature = "development"))]
    #[test]
    fn debug_state_dir_uses_current_working_directory() {
        let cwd = std::env::current_dir().unwrap();

        assert_eq!(user_state_dir(None).unwrap(), cwd.join(".local/state"));
        assert_eq!(
            user_state_dir(Some("chessvault")).unwrap(),
            cwd.join(".local/state/chessvault")
        );
    }
}
