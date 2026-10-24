//! Subprocess adapter for the optional Python differential suite (not a runtime dependency).

use std::path::PathBuf;

use anyhow::{Result, ensure};
use serde_json::{Value, json};

fn paths(result: Result<Vec<PathBuf>>) -> Value {
    match result {
        Ok(paths) => json!({ "paths": paths }),
        Err(error) => json!({ "error": error.to_string() }),
    }
}

fn path(result: Result<PathBuf>) -> Value {
    paths(result.map(|path| vec![path]))
}

fn main() -> Result<()> {
    ensure!(cfg!(target_os = "linux"), "The differential probe requires Linux");
    ensure!(
        !cfg!(feature = "development"),
        "Build the probe with --no-default-features"
    );
    let request: Value = serde_json::from_reader(std::io::stdin())?;
    let app = request["app"].as_str();
    println!(
        "{}",
        json!({
            "state": path(platform_dirs::user_state_dir(app)),
            "data": path(platform_dirs::user_share_dir(app)),
            "site": paths(platform_dirs::system_share_dirs(app)),
            "log_default": path(platform_dirs::user_log_dir(app, None)),
            "log_true": path(platform_dirs::user_log_dir(app, Some(true))),
            "log_false": path(platform_dirs::user_log_dir(app, Some(false))),
        })
    );
    Ok(())
}
