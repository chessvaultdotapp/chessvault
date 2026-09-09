# Repository guide

## Commands and toolchain

- Run Cargo commands from the workspace root. The only member is `apps/chessvault`, package `chessvault`.
- `rust-toolchain.toml` pins Rust **1.98.0**; the package uses edition 2024 and the workspace uses resolver 3.
- Launch the desktop GUI: `cargo run -p chessvault`.
- Check compilation including tests: `cargo check -p chessvault --all-targets`.
- Run tests: `cargo test -p chessvault`. Tests live in the binary's `logs.rs`; there is no library target.
- Run one test: `cargo test -p chessvault --bin chessvault logs::tests::bridged_logs_follow_the_original_source_when_log_is_disabled -- --exact`.
- Formatting: `cargo fmt --all -- --check`. Lint: `cargo clippy -p chessvault --all-targets`.

## App wiring

- `apps/chessvault/src/main.rs` uses Iced **0.14**'s `iced::application` builder with `ChessVault::{update, view, subscription}`. Follow this API rather than older Iced `Application` trait examples.
- Iced's `tokio` feature enables the console's timer subscription. The initial UI is mostly empty; **F12** opens the developer console, which refreshes every 250 ms while open.
- The console uses a read-only `text_editor`: reject editing actions and preserve the selection during periodic refresh so copying remains usable.

## Logging invariants

- `Logs::init()` installs the global tracing subscriber before the GUI starts. Tests instead use scoped subscribers via `tracing::subscriber::with_default`.
- `RUST_LOG` controls capture (default `info`); use `RUST_LOG=chessvault=debug cargo run -p chessvault` for app debug events. Terminal output is enabled only with debug assertions, but in-app capture also works in release builds.
- Console source toggles filter snapshots, not capture; hidden events can be revealed later. `clear()` removes history but preserves source settings. “Copy all” copies the filtered snapshot.
- Sources come from the first `::`-separated component of the event target. Keep `NormalizeEvent` handling for bridged `log` records so dependency events retain their original source instead of becoming `log`.
- Publish each complete formatted event atomically on `LogWriter` drop; the shared buffer retains the latest 2,000 events, not lines.
