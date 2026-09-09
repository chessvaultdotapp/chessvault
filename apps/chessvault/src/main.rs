use iced::keyboard::{self, Key, key::Named};
use iced::widget::{
    Space, button, checkbox, column, container, row, scrollable, text, text_editor,
};
use iced::{Element, Fill, Font, Subscription, Task};

fn main() -> iced::Result {
    let logs = logs::Logs::init();
    tracing::info!("ChessVault started");

    iced::application(
        move || ChessVault {
            logs: logs.clone(),
            console_open: false,
            log_text: String::new(),
            log_content: text_editor::Content::new(),
            sources_open: false,
        },
        ChessVault::update,
        ChessVault::view,
    )
    .title("ChessVault")
    .subscription(ChessVault::subscription)
    .run()
}

struct ChessVault {
    logs: logs::Logs,
    console_open: bool,
    log_text: String,
    log_content: text_editor::Content,
    sources_open: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ToggleConsole,
    ClearLogs,
    RefreshLogs,
    LogAction(text_editor::Action),
    CopyLogs,
    ToggleSources,
    SetSourceEnabled(String, bool),
}

impl ChessVault {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleConsole => {
                self.console_open = !self.console_open;
                tracing::debug!(open = self.console_open, "Developer console toggled");
                self.refresh_logs();
            }
            Message::ClearLogs => {
                self.logs.clear();
                self.log_text.clear();
                self.log_content = text_editor::Content::new();
            }
            Message::RefreshLogs => {
                // Keep a selection stable while the user is copying, even if
                // new events arrive. Capture continues in the log buffer.
                if self.log_content.selection().is_none() {
                    self.refresh_logs();
                }
            }
            Message::LogAction(action) => {
                if !action.is_edit() {
                    self.log_content.perform(action);
                }
            }
            Message::CopyLogs => return iced::clipboard::write(self.logs.snapshot()),
            Message::ToggleSources => self.sources_open = !self.sources_open,
            Message::SetSourceEnabled(source, enabled) => {
                self.logs.set_source_enabled(source, enabled);
                self.refresh_logs();
            }
        }
        Task::none()
    }

    fn refresh_logs(&mut self) {
        let logs = self.logs.snapshot();
        if logs != self.log_text {
            self.log_content = text_editor::Content::with_text(&logs);
            self.log_content
                .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
            self.log_text = logs;
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let keyboard = iced::event::listen_with(|event, _status, _window| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::F12),
                repeat: false,
                ..
            }) => Some(Message::ToggleConsole),
            _ => None,
        });

        if self.console_open {
            Subscription::batch([
                keyboard,
                iced::time::every(Duration::from_millis(250)).map(|_| Message::RefreshLogs),
            ])
        } else {
            keyboard
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = column![Space::new().height(Fill)];

        if !self.console_open {
            return content.into();
        }

        let header = row![
            text("Developer console").size(18),
            Space::new().width(Fill),
            button(if self.sources_open {
                "Hide sources"
            } else {
                "Sources"
            })
            .on_press(Message::ToggleSources),
            button("Copy all").on_press(Message::CopyLogs),
            button("Clear").on_press(Message::ClearLogs),
            button("Close (F12)").on_press(Message::ToggleConsole),
        ]
        .spacing(12)
        .align_y(iced::Center);

        let editor = text_editor(&self.log_content)
            .on_action(Message::LogAction)
            .placeholder("No logs match the selected sources.")
            .font(Font::MONOSPACE)
            .size(13)
            .height(Fill);

        let mut body = row![editor].spacing(12).height(Fill);
        if self.sources_open {
            let mut sources = column![text("Log sources").size(16)].spacing(8);
            for (source, enabled) in self.logs.sources() {
                sources =
                    sources.push(checkbox(enabled).label(source.clone()).on_toggle(
                        move |enabled| Message::SetSourceEnabled(source.clone(), enabled),
                    ));
            }
            body = body.push(scrollable(sources).width(200).height(Fill));
        }

        let console = container(column![header, body].spacing(12))
            .padding(16)
            .width(Fill)
            .height(300)
            .style(container::rounded_box);

        content.push(console).into()
    }
}
mod logs;

use std::time::Duration;
