use std::{io::Write, time::Duration};

use anyhow::{Context, Result};
use iced::{
    Element, Fill, Font, Size, Subscription, Task,
    keyboard::{self, Key, key::Named},
    widget::{Space, button, checkbox, column, container, row, scrollable, text, text_editor},
    window,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use chess_core::{Game, GameStatus, Position, Side, Square};

mod board;
mod fs;
mod game_status;
mod icon;
mod logs;

fn main() -> iced::Result {
    let logs = logs::Logs::init();
    info!("ChessVault started");

    let mut window_settings = window::Settings {
        icon: match icon::load() {
            Ok(icon) => Some(icon),
            Err(err) => {
                error!(error = ?err, "Failed to load window icon");
                None
            }
        },
        ..window::Settings::default()
    };
    #[cfg(target_os = "linux")]
    {
        window_settings.platform_specific.application_id = "chessvault".into();
    }
    match WindowRecreateInfo::load() {
        Ok(Some(info)) => window_settings.size = Size::new(info.width, info.height),
        Ok(None) => {
            info!("No saved window size found; using default size");
        }
        Err(err) => error!(error = ?err, "Failed to restore window size; using default size"),
    }

    iced::application(
        move || ChessVault::boot(logs.clone()),
        ChessVault::update,
        ChessVault::view,
    )
    .title("ChessVault")
    .window(window_settings)
    .exit_on_close_request(false)
    .subscription(ChessVault::subscription)
    .run()
}

struct ChessVault {
    logs: logs::Logs,
    console_open: bool,
    log_text: String,
    log_content: text_editor::Content,
    sources_open: bool,
    game: Game,
    selected_square: Option<Square>,
}

#[derive(Serialize, Deserialize, Debug)]
struct WindowRecreateInfo {
    width: f32,
    height: f32,
}

impl WindowRecreateInfo {
    fn load() -> Result<Option<Self>> {
        let path = fs::window_recreate_info_filepath().context("Failed to resolve window recreate info filepath")?;

        let json = match std::fs::read_to_string(&path) {
            Ok(json) => json,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(err).with_context(|| format!("Failed to read {}", path.display()));
            }
        };

        Self::from_json(&json)
            .with_context(|| format!("Failed to load window recreate info from {}", path.display()))
            .map(Some)
    }

    fn from_json(json: &str) -> Result<Self> {
        let info: Self = serde_json::from_str(json).context("Failed to deserialize window recreate info")?;
        anyhow::ensure!(
            info.width.is_finite() && info.height.is_finite() && info.width > 0.0 && info.height > 0.0,
            "Saved window dimensions must be finite and positive"
        );
        Ok(info)
    }

    fn save(&self) -> Result<()> {
        let path = fs::window_recreate_info_filepath().context("Failed to resolve window recreate info filepath")?;

        let json = serde_json::to_string(self).context("Failed to serialize window recreate info")?;

        let mut file =
            std::fs::File::create(&path).with_context(|| format!("Failed to create or truncate {}", path.display()))?;

        file.write_all(json.as_bytes())
            .with_context(|| format!("Failed to write window recreate info to {}", path.display()))?;

        Ok(())
    }
}

impl From<Size<f32>> for WindowRecreateInfo {
    fn from(value: Size<f32>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

#[cfg(test)]
mod window_recreate_tests {
    use super::*;

    #[test]
    fn saved_dimensions_round_trip() {
        let json = serde_json::to_string(&WindowRecreateInfo::from(Size::new(820.5, 620.0))).unwrap();
        let info = WindowRecreateInfo::from_json(&json).unwrap();
        assert_eq!(info.width, 820.5);
        assert_eq!(info.height, 620.0);
    }

    #[test]
    fn invalid_json_retains_deserialization_error() {
        let err = WindowRecreateInfo::from_json("{invalid}").unwrap_err();
        assert!(err.downcast_ref::<serde_json::Error>().is_some());
    }

    #[test]
    fn invalid_dimensions_are_rejected() {
        for json in [
            r#"{"width":0,"height":620}"#,
            r#"{"width":820,"height":-1}"#,
            r#"{"width":1e100,"height":620}"#,
            r#"{"width":820,"height":1e100}"#,
            r#"{"width":null,"height":620}"#,
            r#"{"width":820}"#,
        ] {
            assert!(WindowRecreateInfo::from_json(json).is_err(), "{json}");
        }
    }
}

#[cfg(test)]
mod resignation_tests {
    use super::*;

    #[test]
    fn resigning_ends_play_and_clears_selection() {
        for side in [Side::White, Side::Black] {
            let mut app = ChessVault {
                logs: logs::Logs::default(),
                console_open: false,
                log_text: String::new(),
                log_content: text_editor::Content::new(),
                sources_open: false,
                game: Game::new(Position::standard()),
                selected_square: Some(Square::E2),
            };
            let _ = app.update(Message::Resign(side));
            assert_eq!(app.game.status(), GameStatus::Resigned(side));
            assert_eq!(app.selected_square, None);
            let _ = app.update(Message::SelectSquare(Square::E2));
            let _ = app.update(Message::SelectSquare(Square::E4));
            assert_eq!(app.selected_square, None);
            assert!(app.game.position.piece_at(Square::E2).is_some());
            assert!(app.game.position.piece_at(Square::E4).is_none());
            let _ = app.update(Message::Resign(Side::White));
            let _ = app.update(Message::Resign(Side::Black));
            assert_eq!(app.game.status(), GameStatus::Resigned(side));
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    SelectSquare(Square),
    Resign(Side),
    ToggleConsole,
    ClearLogs,
    RefreshLogs,
    LogAction(text_editor::Action),
    CopyLogs,
    ToggleSources,
    SetSourceEnabled(String, bool),
    WindowCloseRequested(window::Id),
}

impl ChessVault {
    fn boot(logs: logs::Logs) -> Self {
        let mut app = Self {
            logs,
            console_open: false,
            log_text: String::new(),
            log_content: text_editor::Content::new(),
            sources_open: false,
            game: Game::new(Position::standard()),
            selected_square: None,
        };

        if let Err(err) = app.initialize() {
            error!(error = ?err, "Application initialization failed");
            std::process::exit(1);
        }

        app
    }

    fn initialize(&mut self) -> Result<()> {
        match application_runtime::fs::application_state_dir().context("Failed to resolve application state directory")
        {
            Ok(path) => std::fs::create_dir_all(&path)
                .with_context(|| format!("Failed to create application state directory: {}", path.display())),
            Err(err) => Err(err),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectSquare(square) => {
                if self.game.status() == GameStatus::Ongoing {
                    self.selected_square = board::click(&mut self.game.position, self.selected_square, square);
                }
            }
            Message::Resign(side) => {
                if self.game.resign(side) {
                    self.selected_square = None;
                }
            }
            Message::ToggleConsole => {
                self.console_open = !self.console_open;
                debug!(open = self.console_open, "Developer console toggled");
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
            Message::WindowCloseRequested(id) => {
                return window::size(id).then(move |size| {
                    if let Err(err) = WindowRecreateInfo::from(size).save() {
                        error!(error = ?err, "Failed to save window recreate info");
                    }

                    window::close(id)
                });
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

        let close_requests = window::close_requests().map(Message::WindowCloseRequested);

        if self.console_open {
            Subscription::batch([
                keyboard,
                close_requests,
                iced::time::every(Duration::from_millis(250)).map(|_| Message::RefreshLogs),
            ])
        } else {
            Subscription::batch([keyboard, close_requests])
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let game_status = self.game.status();
        let ongoing = game_status == GameStatus::Ongoing;
        let label = game_status::label(game_status);
        let status = container(text(label).size(24)).padding(16).width(Fill);
        let status = if label.is_empty() {
            status
        } else {
            status.style(container::rounded_box)
        };

        let controls = column![
            button("Black resigns")
                .on_press_maybe(ongoing.then_some(Message::Resign(Side::Black)))
                .width(Fill),
            status,
            button("White resigns")
                .on_press_maybe(ongoing.then_some(Message::Resign(Side::White)))
                .width(Fill),
        ]
        .spacing(12);

        let content = column![
            container(
                row![
                    board::view(&self.game.position, self.selected_square, Message::SelectSquare),
                    container(controls).width(220).center_y(Fill),
                ]
                .spacing(24)
            )
            .padding(24)
            .width(Fill)
            .height(Fill)
        ];

        if !self.console_open {
            return content.into();
        }

        let header = row![
            text("Developer console").size(18),
            Space::new().width(Fill),
            button(if self.sources_open { "Hide sources" } else { "Sources" }).on_press(Message::ToggleSources),
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
                sources = sources.push(
                    checkbox(enabled)
                        .label(source.clone())
                        .on_toggle(move |enabled| Message::SetSourceEnabled(source.clone(), enabled)),
                );
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
