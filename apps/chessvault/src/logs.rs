use std::collections::{BTreeMap, VecDeque};
use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use chrono::Local;
use tracing_appender::rolling;
use tracing_log::NormalizeEvent;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, format, layer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, registry};

const MAX_ENTRIES: usize = 2_000;
// Dependency events can include entire pixel buffers. Keep console layout bounded;
// the independent JSON file layer still writes the complete event.
const MAX_ENTRY_BYTES: usize = 4_096;
const TRUNCATED_ENTRY: &str = "\n[console entry truncated]\n";
const DEFAULT_LOG_FILTER: &str = "off,chessvault=info";

#[derive(Clone, Default)]
pub struct Logs(Arc<Mutex<LogStore>>);

struct Entry {
    source: String,
    text: String,
}

struct LogStore {
    entries: VecDeque<Entry>,
    sources: BTreeMap<String, bool>,
}

impl Default for LogStore {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            sources: [("chessvault".into(), true)].into(),
        }
    }
}

impl Logs {
    pub fn init() -> Result<Self> {
        let logs = Self::default();

        let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_FILTER));
        #[cfg(debug_assertions)]
        let terminal_layer = layer().with_ansi(true).with_writer(io::stderr);

        let (file_appender, file_error) = prepare_file_logging(
            platform_dirs::user_log_dir(Some("chessvault"), None).context("Failed to resolve application log dir"),
        );
        let file_layer = file_appender.map(|appender| {
            layer()
                .json()
                .flatten_event(true)
                .with_ansi(false)
                .with_writer(appender)
        });

        let subscriber = registry().with(env_filter).with(logs.capture_layer()).with(file_layer);
        #[cfg(debug_assertions)]
        let subscriber = subscriber.with(terminal_layer);
        subscriber
            .try_init()
            .context("Failed to initialize logging subscriber")?;

        report_file_logging_failure(file_error);
        Ok(logs)
    }

    // A dedicated formatting layer preserves tracing's field/span formatting while
    // capturing complete events independently of terminal and file output.
    pub(super) fn capture_layer<S>(&self) -> impl Layer<S>
    where
        S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    {
        layer()
            .with_ansi(false)
            .event_format(LogFormatter(self.clone()))
            .with_writer(io::sink)
    }

    fn store_event(&self, target: &str, mut text: String) {
        if text.is_empty() {
            return;
        }
        if text.len() > MAX_ENTRY_BYTES {
            let mut end = MAX_ENTRY_BYTES;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            text.truncate(end);
            text.push_str(TRUNCATED_ENTRY);
        }

        let source = target.split("::").next().unwrap_or(target).to_owned();
        // Publish complete events atomically so concurrent formatting cannot interleave them.
        let mut store = self.0.lock().unwrap();
        store.sources.entry(source.clone()).or_insert(false);
        if store.entries.len() == MAX_ENTRIES {
            store.entries.pop_front();
        }
        store.entries.push_back(Entry { source, text });
    }

    pub fn snapshot(&self) -> String {
        let store = self.0.lock().unwrap();
        store
            .entries
            .iter()
            .filter(|entry| store.sources.get(&entry.source).copied().unwrap_or(false))
            .map(|entry| entry.text.as_str())
            .collect()
    }

    pub fn sources(&self) -> Vec<(String, bool)> {
        self.0
            .lock()
            .unwrap()
            .sources
            .iter()
            .map(|(source, enabled)| (source.clone(), *enabled))
            .collect()
    }

    pub fn set_source_enabled(&self, source: String, enabled: bool) {
        self.0.lock().unwrap().sources.insert(source, enabled);
    }

    pub fn clear(&self) {
        self.0.lock().unwrap().entries.clear();
    }
}

fn prepare_file_logging(log_dir: Result<PathBuf>) -> (Option<rolling::RollingFileAppender>, Option<anyhow::Error>) {
    let result = log_dir.and_then(|log_dir| {
        let local_time = Local::now().format("%Y-%m-%dT%H-%M-%S-%f%z").to_string();
        rolling::RollingFileAppender::builder()
            .rotation(rolling::Rotation::NEVER)
            .filename_prefix(format!("{local_time}.log"))
            .build(&log_dir)
            .with_context(|| format!("Failed to initialize log file in {}", log_dir.display()))
    });
    match result {
        Ok(appender) => (Some(appender), None),
        Err(error) => (None, Some(error)),
    }
}

fn report_file_logging_failure(error: Option<anyhow::Error>) {
    if let Some(error) = error {
        tracing::warn!(error = %format!("{error:#}"), "File logging unavailable; continuing without a log file");
    }
}

struct LogFormatter(Logs);

impl<S, N> FormatEvent<S, N> for LogFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        _writer: format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let mut text = String::new();
        format::format()
            .with_ansi(false)
            .format_event(ctx, format::Writer::new(&mut text), event)?;

        // Bridged records store the original target in event fields; their
        // static tracing metadata only reports "log".
        let normalized = event.normalized_metadata();
        let metadata = normalized.as_ref().unwrap_or_else(|| event.metadata());
        self.0.store_event(metadata.target(), text);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[derive(Clone, Default)]
    struct TestWriter(Arc<Mutex<Vec<u8>>>);

    impl io::Write for TestWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[rstest]
    #[case::resolution_failure(true)]
    #[case::creation_failure(false)]
    fn file_logging_failure_preserves_console_capture(#[case] resolution_failure: bool) {
        let path = if resolution_failure {
            Err(anyhow::anyhow!("test path resolution failure"))
        } else {
            // An existing executable is a file, so it cannot contain a log directory.
            Ok(std::env::current_exe().unwrap().join("logs"))
        };
        let (appender, error) = prepare_file_logging(path);
        assert!(appender.is_none());
        assert!(error.is_some());

        let logs = Logs::default();
        let subscriber = registry()
            .with(EnvFilter::new(DEFAULT_LOG_FILTER))
            .with(logs.capture_layer())
            .with(appender.map(|appender| layer().json().with_writer(appender)));
        tracing::subscriber::with_default(subscriber, || {
            report_file_logging_failure(error);
            tracing::info!("application continues");
        });

        let snapshot = logs.snapshot();
        assert!(snapshot.contains("WARN"));
        assert!(snapshot.contains("File logging unavailable"));
        assert!(snapshot.contains("application continues"));
        assert!(snapshot.contains(if resolution_failure {
            "test path resolution failure"
        } else {
            "Failed to initialize log file"
        }));
    }

    #[test]
    fn oversized_dependency_events_are_bounded_before_being_enabled() {
        let logs = Logs::default();
        logs.store_event("iced_winit", "x".repeat(335_791));

        assert!(logs.snapshot().is_empty());
        logs.set_source_enabled("iced_winit".into(), true);
        let snapshot = logs.snapshot();
        assert_eq!(snapshot.len(), MAX_ENTRY_BYTES + TRUNCATED_ENTRY.len());
        assert!(snapshot.ends_with(TRUNCATED_ENTRY));
    }

    #[test]
    fn truncation_preserves_utf8_boundaries() {
        let logs = Logs::default();
        let prefix = "x".repeat(MAX_ENTRY_BYTES - 1);
        logs.store_event("chessvault", format!("{prefix}♟more"));
        assert_eq!(logs.snapshot(), format!("{prefix}{TRUNCATED_ENTRY}"));
    }

    #[test]
    fn entry_at_byte_limit_is_not_truncated() {
        let logs = Logs::default();
        let text = "x".repeat(MAX_ENTRY_BYTES);
        logs.store_event("chessvault", text.clone());
        assert_eq!(logs.snapshot(), text);
    }

    #[test]
    fn empty_events_do_not_discover_sources() {
        let logs = Logs::default();
        logs.store_event("dependency", String::new());
        assert!(logs.snapshot().is_empty());
        assert_eq!(logs.sources(), vec![("chessvault".into(), true)]);
    }

    #[test]
    fn sources_start_with_only_chessvault() {
        assert_eq!(Logs::default().sources(), vec![("chessvault".into(), true)]);
    }

    #[test]
    fn captures_formatted_events_and_clears_history() {
        let logs = Logs::default();
        let subscriber = registry().with(logs.capture_layer());

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(answer = 42, "Console capture test");
        });

        let snapshot = logs.snapshot();
        assert!(snapshot.contains("INFO"));
        assert!(snapshot.contains("Console capture test"));
        assert!(snapshot.contains("answer=42"));
        assert!(!snapshot.contains('\u{1b}'));

        logs.clear();
        assert!(logs.snapshot().is_empty());
    }

    #[test]
    fn capture_and_json_layers_share_the_filter_without_duplicate_capture() {
        let logs = Logs::default();
        let subscriber = registry()
            .with(EnvFilter::new("info"))
            .with(logs.capture_layer())
            .with(layer().json().with_writer(io::sink));

        tracing::subscriber::with_default(subscriber, || {
            tracing::debug!("filtered event");
            tracing::info!("captured once");
        });

        let snapshot = logs.snapshot();
        assert_eq!(snapshot.lines().count(), 1);
        assert!(snapshot.contains("captured once"));
        assert!(!snapshot.contains("filtered event"));
    }

    #[test]
    fn default_filter_excludes_dependencies_from_all_layers() {
        let logs = Logs::default();
        let output = TestWriter::default();
        let file_output = output.clone();
        let subscriber = registry()
            .with(EnvFilter::new(DEFAULT_LOG_FILTER))
            .with(logs.capture_layer())
            .with(layer().json().with_writer(move || file_output.clone()));

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: "chessvault::game", "application event");
            tracing::debug!(target: "chessvault::game", "debug event");
            tracing::error!(target: "new_dependency::module", "dependency event");
        });

        let file_output = String::from_utf8(output.0.lock().unwrap().clone()).unwrap();
        for snapshot in [logs.snapshot(), file_output] {
            assert!(snapshot.contains("application event"));
            assert!(!snapshot.contains("debug event"));
            assert!(!snapshot.contains("dependency event"));
        }
        assert!(!logs.sources().iter().any(|(source, _)| source == "new_dependency"));
    }

    #[test]
    fn retains_only_the_latest_entries() {
        let logs = Logs::default();
        for index in 0..=MAX_ENTRIES {
            logs.store_event("chessvault", format!("event {index}\n"));
        }

        let snapshot = logs.snapshot();
        assert_eq!(snapshot.lines().count(), MAX_ENTRIES);
        assert!(snapshot.starts_with("event 1\n"));
        assert!(snapshot.ends_with(&format!("event {MAX_ENTRIES}\n")));
    }

    #[rstest]
    #[case::wgpu_hal("wgpu_hal::vulkan", "wgpu_hal")]
    #[case::iced_winit("iced_winit::window", "iced_winit")]
    #[case::iced_wgpu("iced_wgpu", "iced_wgpu")]
    #[case::calloop("calloop", "calloop")]
    fn bridged_logs_follow_the_original_source_when_log_is_disabled(#[case] target: &str, #[case] source: &str) {
        let logs = Logs::default();
        let subscriber = registry().with(logs.capture_layer());

        tracing::subscriber::with_default(subscriber, || {
            log::Log::log(
                &tracing_log::LogTracer::new(),
                &log::Record::builder()
                    .args(format_args!("bridged dependency event"))
                    .level(log::Level::Info)
                    .target(target)
                    .build(),
            );
        });

        assert!(logs.snapshot().is_empty());
        logs.set_source_enabled(source.into(), true);
        let snapshot = logs.snapshot();
        assert_eq!(snapshot.lines().count(), 1);
        assert!(snapshot.contains(source));
        assert!(snapshot.contains("bridged dependency event"));
        logs.set_source_enabled(source.into(), false);
        assert!(!logs.sources().iter().any(|(source, _)| source == "log"));
        logs.set_source_enabled("log".into(), true);
        assert!(logs.snapshot().is_empty());
    }

    #[test]
    fn filters_sources_and_can_reveal_previously_hidden_events() {
        let logs = Logs::default();
        let subscriber = registry().with(logs.capture_layer());
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: "chessvault::game", "visible event");
            tracing::info!(target: "iced_winit::window", "hidden window event");
            tracing::info!(target: "wgpu_hal::vulkan", "hidden backend event");
            tracing::info!(target: "iced_wgpu", "hidden renderer event");
            tracing::info!(target: "new_source::module", "discovered event");
        });

        assert!(logs.snapshot().contains("visible event"));
        assert!(!logs.snapshot().contains("hidden"));
        assert!(!logs.snapshot().contains("discovered event"));
        assert!(logs.sources().contains(&("new_source".into(), false)));
        logs.set_source_enabled("new_source".into(), true);
        assert!(logs.snapshot().contains("discovered event"));

        logs.set_source_enabled("iced_winit".into(), true);
        assert!(logs.snapshot().contains("hidden window event"));
        logs.set_source_enabled("chessvault".into(), false);
        assert!(!logs.snapshot().contains("visible event"));

        logs.clear();
        assert!(logs.snapshot().is_empty());
        assert!(logs.sources().contains(&("chessvault".into(), false)));
    }
}
