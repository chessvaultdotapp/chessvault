use iced::widget::text_editor;

use crate::logs::Logs;

mod message;
mod subscription;
mod update;
mod view;

#[cfg(test)]
mod tests;

pub(crate) use message::Message;

pub(crate) struct State {
    pub(crate) logs: Logs,
    pub(crate) open: bool,
    pub(crate) text: String,
    pub(crate) content: text_editor::Content,
    pub(crate) sources_open: bool,
}

impl State {
    pub(crate) fn new(logs: Logs) -> Self {
        Self {
            logs,
            open: false,
            text: String::new(),
            content: text_editor::Content::new(),
            sources_open: false,
        }
    }
}
