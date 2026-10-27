use iced::widget::text_editor;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    Toggle,
    Clear,
    Refresh,
    Action(text_editor::Action),
    Copy,
    ToggleSources,
    SetSourceEnabled(String, bool),
}
