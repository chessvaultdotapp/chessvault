use std::collections::{BTreeMap, VecDeque};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use tracing_log::NormalizeEvent;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, format};
use tracing_subscriber::registry::LookupSpan;

const MAX_ENTRIES: usize = 2_000;

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
            sources: [
                ("chessvault".into(), true),
                ("log".into(), false),
                ("calloop".into(), false),
                ("iced_winit".into(), false),
                ("wgpu_hal".into(), false),
                ("iced_wgpu".into(), false),
            ]
            .into(),
        }
    }
}

impl Logs {
    pub fn init() -> Self {
        let logs = Self::default();

        #[cfg(debug_assertions)]
        let terminal = tracing_subscriber::fmt::writer::BoxMakeWriter::new(io::stderr);
        #[cfg(not(debug_assertions))]
        let terminal = tracing_subscriber::fmt::writer::BoxMakeWriter::new(io::sink);

        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
            )
            .with_ansi(false)
            .event_format(LogFormatter(logs.clone()))
            .with_writer(terminal)
            .init();

        logs
    }

    fn writer(&self, target: &str) -> LogWriter {
        LogWriter {
            logs: self.clone(),
            source: target.split("::").next().unwrap_or(target).to_owned(),
            buffer: Vec::new(),
        }
    }

    pub fn snapshot(&self) -> String {
        let store = self.0.lock().unwrap();
        store
            .entries
            .iter()
            .filter(|entry| store.sources.get(&entry.source).copied().unwrap_or(true))
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

struct LogFormatter(Logs);

impl<S, N> FormatEvent<S, N> for LogFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let mut text = String::new();
        format::format().with_ansi(false).format_event(
            ctx,
            format::Writer::new(&mut text),
            event,
        )?;

        // Bridged records store the original target in event fields; their
        // static tracing metadata only reports "log".
        let normalized = event.normalized_metadata();
        let metadata = normalized.as_ref().unwrap_or_else(|| event.metadata());
        self.0
            .writer(metadata.target())
            .write_all(text.as_bytes())
            .map_err(|_| std::fmt::Error)?;

        writer.write_str(&text)
    }
}

// Collect each formatted event before publishing it so concurrent writers cannot
// interleave fragments of different events in the console.
struct LogWriter {
    logs: Logs,
    source: String,
    buffer: Vec<u8>,
}

impl Write for LogWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for LogWriter {
    fn drop(&mut self) {
        if self.buffer.is_empty() {
            return;
        }

        let mut store = self.logs.0.lock().unwrap();
        store.sources.entry(self.source.clone()).or_insert(true);
        if store.entries.len() == MAX_ENTRIES {
            store.entries.pop_front();
        }
        store.entries.push_back(Entry {
            source: self.source.clone(),
            text: String::from_utf8_lossy(&self.buffer).into_owned(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn captures_formatted_events_and_clears_history() {
        let logs = Logs::default();
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .event_format(LogFormatter(logs.clone()))
            .with_writer(io::sink)
            .finish();

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
    fn retains_only_the_latest_entries() {
        let logs = Logs::default();
        for index in 0..=MAX_ENTRIES {
            writeln!(logs.writer("chessvault"), "event {index}").unwrap();
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
    fn bridged_logs_follow_the_original_source_when_log_is_disabled(
        #[case] target: &str,
        #[case] source: &str,
    ) {
        let logs = Logs::default();
        let subscriber = tracing_subscriber::fmt()
            .event_format(LogFormatter(logs.clone()))
            .with_writer(io::sink)
            .finish();

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
        assert!(logs.sources().contains(&("log".into(), false)));
        logs.set_source_enabled("log".into(), true);
        assert!(logs.snapshot().is_empty());
    }

    #[test]
    fn filters_sources_and_can_reveal_previously_hidden_events() {
        let logs = Logs::default();
        let subscriber = tracing_subscriber::fmt()
            .event_format(LogFormatter(logs.clone()))
            .with_writer(io::sink)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: "chessvault::game", "visible event");
            tracing::info!(target: "iced_winit::window", "hidden window event");
            tracing::info!(target: "wgpu_hal::vulkan", "hidden backend event");
            tracing::info!(target: "iced_wgpu", "hidden renderer event");
            tracing::info!(target: "new_source::module", "discovered event");
        });

        assert!(logs.snapshot().contains("visible event"));
        assert!(!logs.snapshot().contains("hidden"));
        assert!(logs.sources().contains(&("new_source".into(), true)));

        logs.set_source_enabled("iced_winit".into(), true);
        assert!(logs.snapshot().contains("hidden window event"));
        logs.set_source_enabled("chessvault".into(), false);
        assert!(!logs.snapshot().contains("visible event"));

        logs.clear();
        assert!(logs.snapshot().is_empty());
        assert!(logs.sources().contains(&("chessvault".into(), false)));
    }
}
