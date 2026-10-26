# Runtime state and logging

## Window persistence and failures

The desktop asks `platform_dirs::user_state_dir(Some("chessvault"))` for its state
directory and appends `recreate` (no extension). With default development features
and debug assertions this is `<cwd>/.local/state/chessvault/recreate`.
Native resolution currently supports Linux only. See the shared
[directory policy](https://chessvaultdotapp.github.io/chessvault/dev/docs/components/platform-dirs/)
for XDG resolution, feature variants, and platform limitations. Resolvers return
paths; the desktop creates directories and performs I/O.

`WindowRecreateInfo` uses exactly 24 bytes: a 16-byte header followed by two
little-endian `f32` dimensions. The header contains `CHSV`, `u32` major/minor
versions 0/1, and a `u32` format identifier 0; integers are little-endian.
Loading requires the exact supported header and length and positive, finite dimensions.
Missing state uses defaults; invalid/unreadable state logs an error and falls back.
Legacy JSON is rejected, not migrated. The next successful save writes binary state.

Automatic exit on close requests is disabled. The app queries window size,
attempts to save it, then closes even if saving fails. Creating the state directory
during boot, unlike saving on exit, is fatal on failure. Writes truncate directly;
they are not atomic. Remove `recreate` while the app is closed to reset dimensions.
Window position, chess position, console history, and source settings are not restored.

## Developer console

F12 toggles the read-only console. `RUST_LOG` filters capture, not just display;
excluded events cannot later be recovered with a source toggle. The fallback
filter is `off,chessvault=info,platform_dirs=info`. Terminal output requires debug
assertions; in-app capture also works in release.

The shared `Arc<Mutex<LogStore>>` retains at most 2,000 complete events, not lines.
Publish events atomically through `Logs::store_event` so concurrent formatting
cannot interleave fragments. Normalize bridged `log` records before extracting
the first `::`-separated target component as their source.

Application events are initially enabled; `platform_dirs` is enabled when
discovered. Other new sources start hidden. Source toggles filter snapshots,
not capture. “Copy all” copies the filtered snapshot. “Clear” removes all retained
events, including hidden ones, without resetting source settings.

The console keeps the last displayed string and `text_editor::Content` with
cursor/selection state. Reject editing actions but allow navigation and selection.
Pause periodic display refresh while text is selected; capture continues.
The 250 ms subscription runs only while the console is open.

## File logging and retention

`Logs::init()` installs the global subscriber before GUI startup. Installation
failure aborts startup; tests instead use scoped subscribers with
`tracing::subscriber::with_default`.

Debug and release builds attempt a per-launch JSON log beneath `user_log_dir`.
The capture filter also applies to files and terminal output. Directory/file
initialization failure is nonfatal and warns through remaining outputs, subject
to `RUST_LOG`. Console truncation notices must not promise a full event on disk.

After opening the current log and installing the subscriber, a background thread
performs retention once per launch without delaying the GUI. It keeps at most
20 logs including the current file and deletes eligible logs older than seven
days using filename timestamps. The current file is always preserved. Only
regular files matching the exact application filename format are eligible;
unrelated files, directories, and symlinks are untouched. Cleanup failures log
errors and do not abort startup or stop other deletions where possible. There
is no cross-instance locking: other instances' active logs are not protected.
