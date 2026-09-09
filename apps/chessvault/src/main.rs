use iced::keyboard::{self, Key, key::Named};
use iced::widget::{Space, button, column, container, row, text, text_editor};
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
}

#[derive(Debug, Clone)]
enum Message {
    ToggleConsole,
    ClearLogs,
    RefreshLogs,
    LogAction(text_editor::Action),
    CopyLogs,
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
            button("Copy all").on_press(Message::CopyLogs),
            button("Clear").on_press(Message::ClearLogs),
            button("Close (F12)").on_press(Message::ToggleConsole),
        ]
        .spacing(12)
        .align_y(iced::Center);

        let console = container(
            column![
                header,
                text_editor(&self.log_content)
                    .on_action(Message::LogAction)
                    .placeholder("No logs yet.")
                    .font(Font::MONOSPACE)
                    .size(13)
                    .height(Fill),
            ]
            .spacing(12),
        )
        .padding(16)
        .width(Fill)
        .height(300)
        .style(container::rounded_box);

        content.push(console).into()
    }
}
mod logs;

use std::time::Duration;
