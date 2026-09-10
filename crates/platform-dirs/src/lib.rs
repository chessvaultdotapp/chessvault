use std::path::PathBuf;

use anyhow::Result;

#[cfg(target_os = "linux")]
mod linux;

/// Returns the user state directory.
///
/// Returns an error on unsupported platforms (currently everything except Linux).
pub fn user_state_dir() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        linux::user_state_dir()
    }

    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("User state directory is unsupported on this platform")
    }
}
