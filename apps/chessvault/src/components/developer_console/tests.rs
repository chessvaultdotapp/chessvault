use iced::widget::text_editor;
use tracing_subscriber::layer::SubscriberExt;

use super::{Message, State};
use crate::logs::Logs;

fn record(logs: &Logs, message: &str) {
    let subscriber = tracing_subscriber::registry().with(logs.capture_layer());
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("{message}");
    });
}

#[test]
fn refresh_preserves_selection_until_the_user_moves_the_cursor() {
    let mut console = State::new(Logs::default());
    record(&console.logs, "first event");
    let _ = console.update(Message::Toggle);
    assert!(console.open);
    assert!(console.text.contains("first event"));
    let _ = console.update(Message::Action(text_editor::Action::SelectAll));
    let selection = console.content.selection();
    assert!(selection.is_some());
    let before = console.text.clone();

    record(&console.logs, "second event");
    let _ = console.update(Message::Refresh);
    assert_eq!(console.text, before);
    assert_eq!(console.content.selection(), selection);
    assert!(console.logs.snapshot().contains("second event"));

    let _ = console.update(Message::Action(text_editor::Action::Move(
        text_editor::Motion::DocumentEnd,
    )));
    let _ = console.update(Message::Refresh);
    assert!(console.text.contains("second event"));
}

#[test]
fn editor_is_read_only_and_clear_preserves_source_settings() {
    let mut console = State::new(Logs::default());
    record(&console.logs, "captured event");
    let _ = console.update(Message::Toggle);
    let before = console.content.text();
    let _ = console.update(Message::Action(text_editor::Action::Edit(text_editor::Edit::Insert(
        'x',
    ))));
    assert_eq!(console.content.text(), before);

    let _ = console.update(Message::ToggleSources);
    let _ = console.update(Message::SetSourceEnabled("chessvault".into(), false));
    let sources = console.logs.sources();
    let _ = console.update(Message::Clear);
    assert!(console.sources_open);
    assert!(console.text.is_empty());
    assert!(console.content.text().is_empty());
    assert!(console.logs.snapshot().is_empty());
    assert_eq!(console.logs.sources(), sources);
}
