//! Platform-aware resolution of user state, share, and log directories.
//!
//! Use [`user_share_dir`] for persistent user data, [`user_state_dir`] for
//! persistent user state, or [`user_log_dir`] for logs beneath the state directory
//! by default.
//! An optional application name selects a subdirectory beneath that base.
//! This crate only resolves paths: callers create directories as needed.
//!
//! # Features and platform support
//!
//! The default-enabled `development` feature makes builds with debug assertions
//! use `<current working directory>/.local/state` for state and
//! `<current working directory>/.local/share` for data on every platform. Builds
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
    with_app_name(base_state_dir()?, app_name)
}

fn with_app_name(mut path: PathBuf, app_name: Option<&str>) -> Result<PathBuf> {
    if let Some(name) = app_name {
        anyhow::ensure!(
            !name.is_empty() && name != "." && name != ".." && !name.contains(['/', '\\', ':', '\0']),
            "Application name must be a single nonempty directory component"
        );

        path.push(name);
    }

    Ok(path)
}

/// Returns the absolute directory for persistent user data.
///
/// With debug assertions and the default-enabled `development` feature, uses
/// `<current working directory>/.local/share` on every platform. Otherwise, on
/// Linux, prefers an absolute `XDG_DATA_HOME`, falling back to `.local/share`
/// beneath an absolute home directory. Non-Unicode paths are preserved.
///
/// Pass `None` for the base or an application name validated as in
/// [`user_state_dir`]. Does not create directories or check their existence.
///
/// # Errors
///
/// Returns an error if base resolution fails (unavailable working directory,
/// no valid native base, or unsupported platform), or the application name is
/// invalid. Base resolution takes precedence over name validation.
///
/// # Examples
///
/// ```
/// if let Ok(path) = platform_dirs::user_share_dir(Some("my-app")) {
///     assert!(path.is_absolute());
///     assert!(path.ends_with("my-app"));
/// }
/// ```
pub fn user_share_dir(app_name: Option<&str>) -> Result<PathBuf> {
    with_app_name(base_share_dir()?, app_name)
}

fn base_share_dir() -> Result<PathBuf> {
    #[cfg(all(debug_assertions, feature = "development"))]
    {
        use anyhow::Context;

        Ok(std::env::current_dir()
            .context("Cannot resolve user share directory: current working directory is unavailable")?
            .join(".local")
            .join("share"))
    }

    #[cfg(all(not(all(debug_assertions, feature = "development")), target_os = "linux"))]
    {
        linux::user_share_dir()
    }

    #[cfg(all(not(all(debug_assertions, feature = "development")), not(target_os = "linux")))]
    {
        anyhow::bail!("User share directory is unsupported on this platform")
    }
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

/// Returns the absolute directory for user logs.
///
/// Resolves [`user_state_dir`] with the supplied application name, then appends
/// `log` when `opinion` is `None` (the default) or `Some(true)`. With
/// `Some(false)`, returns the state directory unchanged.
///
/// Uses the same development and native resolution rules as [`user_state_dir`].
/// Does not create the directory or check whether it exists or is writable.
///
/// # Errors
///
/// Returns the same resolution and application-name validation errors as
/// [`user_state_dir`].
///
/// # Examples
///
/// ```
/// match platform_dirs::user_log_dir(Some("my-app"), None) {
///     Ok(path) => {
///         assert!(path.is_absolute());
///         assert!(path.ends_with("my-app/log"));
///         println!("User log directory: {}", path.display());
///     }
///     Err(error) => eprintln!("Cannot resolve user log directory: {error}"),
/// }
/// ```
pub fn user_log_dir(app_name: Option<&str>, opinion: Option<bool>) -> Result<PathBuf> {
    let mut base = user_state_dir(app_name)?;
    if opinion.unwrap_or(true) {
        base.push("log");
    }

    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_directory_application_names() {
        let base = base_share_dir();
        for name in [None, Some("chessvault"), Some("my app"), Some("šah")] {
            match &base {
                Ok(base) => assert_eq!(
                    user_share_dir(name).unwrap(),
                    name.map_or_else(|| base.clone(), |name| base.join(name))
                ),
                Err(error) => assert_eq!(format!("{:#}", user_share_dir(name).unwrap_err()), format!("{error:#}")),
            }
        }
        for name in ["", ".", "..", "/app", "a/b", "a\\b", "C:app", "a\0b"] {
            assert!(user_share_dir(Some(name)).is_err());
        }
    }

    #[cfg(all(debug_assertions, feature = "development"))]
    #[test]
    fn debug_share_dir_uses_current_working_directory() {
        assert_eq!(
            user_share_dir(Some("chessvault")).unwrap(),
            std::env::current_dir().unwrap().join(".local/share/chessvault")
        );
    }

    #[test]
    fn log_directory_opinions_and_application_names() {
        for name in [None, Some("chessvault"), Some("my app"), Some("šah")] {
            let state = user_state_dir(name);
            for (opinion, append_log) in [(None, true), (Some(true), true), (Some(false), false)] {
                match &state {
                    Ok(state) => {
                        let expected = if append_log { state.join("log") } else { state.clone() };
                        assert_eq!(user_log_dir(name, opinion).unwrap(), expected);
                    }
                    Err(error) => assert_eq!(
                        format!("{:#}", user_log_dir(name, opinion).unwrap_err()),
                        format!("{error:#}")
                    ),
                }
            }
        }
    }

    #[test]
    fn log_directory_preserves_invalid_name_errors() {
        for name in ["", ".", "..", "/app", "a/b", "a\\b", "C:app", "a\0b"] {
            let expected = format!("{:#}", user_state_dir(Some(name)).unwrap_err());
            for opinion in [None, Some(true), Some(false)] {
                assert_eq!(
                    format!("{:#}", user_log_dir(Some(name), opinion).unwrap_err()),
                    expected
                );
            }
        }
    }

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
