use std::time::Duration;

use anyhow::{Context, Result};
use chess_core::{Game, GameStatus, Move, Position, Side, Square};
use iced::{
    Element, Fill, Font, Subscription, Task,
    keyboard::{self, Key, key::Named},
    widget::{Space, button, checkbox, column, container, row, scrollable, text, text_editor},
    window,
};
use tracing::{debug, error};

use crate::{
    Message, WindowRecreateInfo, board, game_status,
    history::{History, Navigation},
    logs::{self, Logs},
    pgn,
};

pub(crate) struct State {
    pub(crate) logs: Logs,
    pub(crate) console_open: bool,
    pub(crate) log_text: String,
    pub(crate) log_content: text_editor::Content,
    pub(crate) sources_open: bool,
    pub(crate) game: Game,
    pub(crate) history: History,
    pub(crate) selected_square: Option<Square>,
    pub(crate) pending_promotion: Option<Move>,
    pub(crate) confirm_new_game: bool,
    pub(crate) saving_pgn: bool,
    pub(crate) pgn_error: Option<String>,
}

impl State {
    pub(crate) fn boot(logs: logs::Logs) -> Self {
        let mut app = Self {
            history: History::default(),
            logs,
            console_open: false,
            log_text: String::new(),
            log_content: text_editor::Content::new(),
            sources_open: false,
            game: Game::new(Position::standard()),
            selected_square: None,
            pending_promotion: None,
            confirm_new_game: false,
            saving_pgn: false,
            pgn_error: None,
        };

        if let Err(err) = app.initialize() {
            error!(error = ?err, "Application initialization failed");
            std::process::exit(1);
        }

        app
    }

    fn initialize(&mut self) -> Result<()> {
        match platform_dirs::user_state_dir(Some("chessvault")).context("Failed to resolve application state directory")
        {
            Ok(path) => std::fs::create_dir_all(&path)
                .with_context(|| format!("Failed to create application state directory: {}", path.display())),
            Err(err) => Err(err),
        }
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        if self.confirm_new_game
            && matches!(
                message,
                Message::Navigate(_)
                    | Message::SelectSquare(_)
                    | Message::Promote(_)
                    | Message::Resign(_)
                    | Message::ClaimDraw
                    | Message::ClaimDrawWithMove(_)
                    | Message::OfferDraw(_)
                    | Message::AcceptDraw(_)
                    | Message::DeclineDraw(_)
            )
        {
            return Task::none();
        }
        let previous_move_count = self.game.moves().len();
        match message {
            Message::SavePgn => {
                if self.saving_pgn {
                    return Task::none();
                }
                let Some(pgn) = pgn::export(&self.game) else {
                    return Task::none();
                };
                self.saving_pgn = true;
                self.pgn_error = None;
                return Task::perform(
                    async move {
                        let Some(file) = rfd::AsyncFileDialog::new()
                            .add_filter("PGN", &["pgn"])
                            .set_file_name("game.pgn")
                            .save_file()
                            .await
                        else {
                            return Ok(());
                        };
                        file.write(pgn.as_bytes()).await.map_err(|err| err.to_string())
                    },
                    Message::PgnSaved,
                );
            }
            Message::PgnSaved(result) => {
                self.saving_pgn = false;
                if let Err(err) = result {
                    error!(error = %err, "Failed to save PGN");
                    self.pgn_error = Some(format!("Could not save PGN: {err}"));
                }
            }
            Message::NewGame => {
                if self.game.status() == GameStatus::Ongoing && !self.game.moves().is_empty() {
                    self.confirm_new_game = true;
                } else {
                    self.reset_game();
                }
            }
            Message::ConfirmNewGame => {
                if self.confirm_new_game {
                    self.reset_game();
                }
            }
            Message::CancelNewGame => self.confirm_new_game = false,
            Message::Navigate(navigation) => {
                if !self.console_open && self.pending_promotion.is_none() {
                    self.history.sync(&self.game);
                    self.history.navigate(&self.game, navigation);
                    self.selected_square = None;
                    return self.history.scroll_to_current(&self.game);
                }
            }
            Message::SelectSquare(square) => {
                if !self.history.is_reviewing()
                    && self.game.status() == GameStatus::Ongoing
                    && self.pending_promotion.is_none()
                {
                    self.pending_promotion =
                        self.game.position().legal_moves().into_iter().find(|mv| {
                            Some(mv.from) == self.selected_square && mv.to == square && mv.promotion.is_some()
                        });
                    if self.pending_promotion.is_none() {
                        self.selected_square = board::click(&mut self.game, self.selected_square, square);
                    }
                }
            }
            Message::Promote(piece) => {
                if let Some(mv) = self.pending_promotion
                    && !self.history.is_reviewing()
                    && self.game.status() == GameStatus::Ongoing
                    && self.game.play(Move {
                        promotion: Some(piece),
                        ..mv
                    })
                {
                    self.pending_promotion = None;
                    self.selected_square = None;
                }
            }
            Message::CancelPromotion => {
                if self.confirm_new_game {
                    self.confirm_new_game = false;
                } else {
                    // Keep the pawn selected so another destination can be chosen.
                    self.pending_promotion = None;
                }
            }
            Message::OfferDraw(side) => {
                if self.draw_actions_enabled() {
                    self.game.offer_draw(side);
                }
            }
            Message::AcceptDraw(side) => {
                if self.draw_actions_enabled() && self.game.accept_draw(side) {
                    self.selected_square = None;
                }
            }
            Message::DeclineDraw(side) => {
                if self.draw_actions_enabled() {
                    self.game.decline_draw(side);
                }
            }
            Message::ClaimDraw => {
                if !self.history.is_reviewing()
                    && !self.console_open
                    && self.pending_promotion.is_none()
                    && self.game.claim_draw()
                {
                    self.selected_square = None;
                }
            }
            Message::ClaimDrawWithMove(mv) => {
                if !self.history.is_reviewing()
                    && !self.console_open
                    && self.pending_promotion.is_none()
                    && self.game.claim_draw_with_move(mv)
                {
                    self.selected_square = None;
                }
            }
            Message::Resign(side) => {
                if !self.history.is_reviewing() && self.game.resign(side) {
                    self.selected_square = None;
                    self.pending_promotion = None;
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
        self.history.sync(&self.game);
        if self.game.moves().len() != previous_move_count {
            return self.history.scroll_to_current(&self.game);
        }
        Task::none()
    }

    fn draw_actions_enabled(&self) -> bool {
        !self.history.is_reviewing()
            && !self.console_open
            && self.pending_promotion.is_none()
            && !self.confirm_new_game
            && self.game.status() == GameStatus::Ongoing
    }

    fn reset_game(&mut self) {
        self.game = Game::new(Position::standard());
        self.history = History::default();
        self.selected_square = None;
        self.pending_promotion = None;
        self.confirm_new_game = false;
        self.pgn_error = None;
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

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        let keyboard = iced::event::listen_with(|event, status, _window| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::F12),
                repeat: false,
                ..
            }) => Some(Message::ToggleConsole),
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::Escape),
                repeat: false,
                ..
            }) => Some(Message::CancelPromotion),
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key),
                modifiers,
                ..
            }) if status == iced::event::Status::Ignored && modifiers.is_empty() => {
                Navigation::from_key(key).map(Message::Navigate)
            }
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

    pub(crate) fn view(&self) -> Element<'_, Message> {
        if self.confirm_new_game {
            return container(
                column![
                    text("Start a new game?").size(24),
                    text("The current game and its move history will be discarded."),
                    row![
                        button("Cancel").on_press(Message::CancelNewGame),
                        button("Start new game").on_press(Message::ConfirmNewGame),
                    ]
                    .spacing(12),
                ]
                .spacing(16),
            )
            .padding(24)
            .center_x(Fill)
            .center_y(Fill)
            .into();
        }
        let game_status = self.history.status(&self.game);
        let ongoing = !self.history.is_reviewing() && game_status == GameStatus::Ongoing;
        let navigation_enabled = !self.console_open && self.pending_promotion.is_none();
        let label = game_status::label(game_status);
        let status = container(text(label).size(24)).padding(16).width(Fill);
        let status = if label.is_empty() {
            status
        } else {
            status.style(container::rounded_box)
        };

        let controls = column![
            button("New game")
                .on_press_maybe(
                    (!self.game.moves().is_empty() || self.game.status() != GameStatus::Ongoing)
                        .then_some(Message::NewGame),
                )
                .width(Fill)
        ]
        .spacing(12)
        .height(Fill);
        let controls = if !self.history.is_reviewing() && self.game.can_claim_draw() {
            controls.push(
                button("Claim draw")
                    .on_press_maybe(navigation_enabled.then_some(Message::ClaimDraw))
                    .width(Fill),
            )
        } else {
            controls
        };
        let mut controls = controls;
        if ongoing {
            let enabled = self.draw_actions_enabled();
            if let Some(offering) = self.game.draw_offer() {
                let (label, responding) = if offering == Side::White {
                    ("White offers a draw", Side::Black)
                } else {
                    ("Black offers a draw", Side::White)
                };
                let player = if responding == Side::White { "White" } else { "Black" };
                controls = controls.push(
                    column![
                        text(label),
                        row![
                            button(text(format!("{player} accepts")))
                                .on_press_maybe(enabled.then_some(Message::AcceptDraw(responding))),
                            button("Decline").on_press_maybe(enabled.then_some(Message::DeclineDraw(responding))),
                        ]
                        .spacing(8),
                    ]
                    .spacing(8),
                );
            } else {
                let side = self.game.position().side_to_move();
                let label = if side == Side::White {
                    "White offers draw"
                } else {
                    "Black offers draw"
                };
                controls = controls.push(
                    button(label)
                        .on_press_maybe(enabled.then_some(Message::OfferDraw(side)))
                        .width(Fill),
                );
            }
        }
        if ongoing && !self.game.can_claim_draw() {
            let mut intended_claims = column![].spacing(8);
            let mut has_intended_claim = false;
            for mv in self.game.position().legal_moves() {
                if self.game.can_claim_draw_with_move(mv) {
                    has_intended_claim = true;
                    intended_claims = intended_claims.push(
                        button(text(format!("Claim draw with {}–{}", mv.from, mv.to)))
                            .on_press_maybe(navigation_enabled.then_some(Message::ClaimDrawWithMove(mv)))
                            .width(Fill),
                    );
                }
            }
            if has_intended_claim {
                controls = controls.push(container(scrollable(intended_claims)).max_height(140));
            }
        }
        if self.game.status() != GameStatus::Ongoing && !self.history.is_reviewing() {
            controls = controls.push(
                button("Save PGN")
                    .on_press_maybe((!self.saving_pgn).then_some(Message::SavePgn))
                    .width(Fill),
            );
            if let Some(error) = &self.pgn_error {
                controls = controls.push(text(error));
            }
        }
        let controls = controls.extend([
            button("Black resigns")
                .on_press_maybe(ongoing.then_some(Message::Resign(Side::Black)))
                .width(Fill)
                .into(),
            status.into(),
            button("White resigns")
                .on_press_maybe(ongoing.then_some(Message::Resign(Side::White)))
                .width(Fill)
                .into(),
            text("Move history").size(20).into(),
            self.history.controls(&self.game, navigation_enabled),
            self.history.move_list(&self.game, navigation_enabled),
        ]);

        let content = column![
            container(
                row![
                    column![
                        board::view(
                            self.history.position(&self.game),
                            self.selected_square,
                            Message::SelectSquare,
                            self.pending_promotion,
                            Message::Promote,
                            Message::CancelPromotion,
                        ),
                        container(text("← Previous · → Next · ↑/Home Start · ↓/End Latest").size(14)).center_x(Fill),
                    ]
                    .spacing(12)
                    .width(Fill)
                    .height(Fill),
                    container(controls).width(260).height(Fill),
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

        let available_sources = self.logs.sources();
        let has_other_sources = available_sources.iter().any(|(source, _)| source != "chessvault");
        let header = row![
            text("Developer console").size(18),
            Space::new().width(Fill),
            button(if self.sources_open { "Hide sources" } else { "Sources" })
                .on_press_maybe(has_other_sources.then_some(Message::ToggleSources)),
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
        if self.sources_open && has_other_sources {
            let mut sources = column![text("Log sources").size(16)].spacing(8);
            for (source, enabled) in available_sources {
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

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
