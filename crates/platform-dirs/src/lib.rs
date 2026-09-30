//! Platform-aware resolution of user state directories.
//!
//! Use [`user_state_dir`] to locate the base directory for persistent user state.
//! This crate only resolves paths: callers append any application-specific
//! subdirectory and create directories as needed.
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

/// Returns the absolute base directory for persistent user state.
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
/// Does not create the directory, check whether it exists or is writable, or
/// append an application-specific name.
///
/// # Errors
///
/// Returns an error if:
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
/// match platform_dirs::user_state_dir() {
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
/// let state_dir = platform_dirs::user_state_dir()?.join("my-app");
/// std::fs::create_dir_all(&state_dir)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn user_state_dir() -> Result<PathBuf> {
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

#[cfg(all(test, debug_assertions, feature = "development"))]
mod tests {
    #[test]
    fn debug_state_dir_uses_current_working_directory() {
        let cwd = std::env::current_dir().unwrap();

        assert_eq!(super::user_state_dir().unwrap(), cwd.join(".local/state"));
    }
}
