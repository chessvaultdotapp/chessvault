use iced::keyboard::{self, Key, key::Named};
use iced::widget::{Space, button, column, container, row, scrollable, text};
use iced::{Element, Fill, Font, Subscription};

fn main() -> iced::Result {
    let logs = logs::Logs::init();
    tracing::info!("ChessVault started");

    iced::application(
        move || ChessVault {
            logs: logs.clone(),
            console_open: false,
            log_text: String::new(),
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
}

#[derive(Debug, Clone)]
enum Message {
    ToggleConsole,
    ClearLogs,
    RefreshLogs,
}

impl ChessVault {
    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleConsole => {
                self.console_open = !self.console_open;
                tracing::debug!(open = self.console_open, "Developer console toggled");
                self.log_text = self.logs.snapshot();
            }
            Message::ClearLogs => {
                self.logs.clear();
                self.log_text.clear();
            }
            Message::RefreshLogs => self.log_text = self.logs.snapshot(),
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
            button("Clear").on_press(Message::ClearLogs),
            button("Close (F12)").on_press(Message::ToggleConsole),
        ]
        .spacing(12)
        .align_y(iced::Center);

        let logs = if self.log_text.is_empty() {
            "No logs yet."
        } else {
            &self.log_text
        };

        let console = container(
            column![
                header,
                scrollable(text(logs).font(Font::MONOSPACE).size(13))
                    .height(Fill)
                    .anchor_bottom(),
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
