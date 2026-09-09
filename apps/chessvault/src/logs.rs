use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use tracing_subscriber::EnvFilter;

const MAX_ENTRIES: usize = 2_000;

#[derive(Clone, Default)]
pub struct Logs(Arc<Mutex<VecDeque<String>>>);

impl Logs {
    pub fn init() -> Self {
        let logs = Self::default();
        let writer_logs = logs.clone();

        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info"))
                    .add_directive("iced_winit=off".parse().unwrap())
                    .add_directive("wgpu_hal=off".parse().unwrap())
                    .add_directive("iced_wgpu=off".parse().unwrap()),
            )
            .with_ansi(false)
            .with_writer(move || writer_logs.writer())
            .init();

        logs
    }

    fn writer(&self) -> LogWriter {
        LogWriter {
            logs: self.clone(),
            buffer: Vec::new(),
        }
    }

    pub fn snapshot(&self) -> String {
        self.0.lock().unwrap().iter().cloned().collect()
    }

    pub fn clear(&self) {
        self.0.lock().unwrap().clear();
    }
}

// Collect each formatted event before publishing it so concurrent writers cannot
// interleave fragments of different events in the console.
struct LogWriter {
    logs: Logs,
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

        let mut entries = self.logs.0.lock().unwrap();
        if entries.len() == MAX_ENTRIES {
            entries.pop_front();
        }
        entries.push_back(String::from_utf8_lossy(&self.buffer).into_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_formatted_events_and_clears_history() {
        let logs = Logs::default();
        let writer_logs = logs.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_writer(move || writer_logs.writer())
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
            writeln!(logs.writer(), "event {index}").unwrap();
        }

        let snapshot = logs.snapshot();
        assert_eq!(snapshot.lines().count(), MAX_ENTRIES);
        assert!(snapshot.starts_with("event 1\n"));
        assert!(snapshot.ends_with(&format!("event {MAX_ENTRIES}\n")));
    }
}
