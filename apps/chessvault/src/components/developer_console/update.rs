use iced::{Task, widget::text_editor};
use tracing::debug;

use super::{Message, State};

impl State {
    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Toggle => {
                self.open = !self.open;
                debug!(open = self.open, "Developer console toggled");
                self.refresh();
            }
            Message::Clear => {
                self.logs.clear();
                self.text.clear();
                self.content = text_editor::Content::new();
            }
            Message::Refresh => {
                // Preserve selection while copying. Capture continues in the buffer.
                if self.content.selection().is_none() {
                    self.refresh();
                }
            }
            Message::Action(action) => {
                if !action.is_edit() {
                    self.content.perform(action);
                }
            }
            Message::Copy => return iced::clipboard::write(self.logs.snapshot()),
            Message::ToggleSources => self.sources_open = !self.sources_open,
            Message::SetSourceEnabled(source, enabled) => {
                self.logs.set_source_enabled(source, enabled);
                self.refresh();
            }
        }
        Task::none()
    }

    fn refresh(&mut self) {
        let logs = self.logs.snapshot();
        if logs != self.text {
            self.content = text_editor::Content::with_text(&logs);
            self.content
                .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
            self.text = logs;
        }
    }
}
