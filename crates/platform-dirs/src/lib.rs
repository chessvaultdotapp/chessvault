use std::path::PathBuf;

use anyhow::Result;

#[cfg(all(
    target_os = "linux",
    any(not(all(debug_assertions, feature = "development")), test)
))]
mod linux;

/// Returns the user state directory.
///
/// With the default-enabled `development` feature, debug builds return
/// `<current working directory>/.local/state` on every platform. Returns an error
/// if the current working directory cannot be read.
///
/// In release builds, or with `development` disabled, resolves the platform's user
/// state directory, returning an error on unsupported platforms (currently
/// everything except Linux). Set `default-features = false` on the dependency to
/// use platform directories in debug builds too.
/// Does not create the directory.
pub fn user_state_dir() -> Result<PathBuf> {
    #[cfg(all(debug_assertions, feature = "development"))]
    {
        use anyhow::Context;

        Ok(std::env::current_dir()
            .context(
                "Cannot resolve user state directory: current working directory is unavailable",
            )?
            .join(".local")
            .join("state"))
    }

    #[cfg(all(
        not(all(debug_assertions, feature = "development")),
        target_os = "linux"
    ))]
    {
        linux::user_state_dir()
    }

    #[cfg(all(
        not(all(debug_assertions, feature = "development")),
        not(target_os = "linux")
    ))]
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
