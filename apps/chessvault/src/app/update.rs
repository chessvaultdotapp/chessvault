use chess_core::{Game, GameStatus, Move, Position};
use iced::{Task, widget::text_editor, window};
use tracing::{debug, error};

use super::{Message, State};
use crate::{WindowRecreateInfo, components::board, history::History, pgn};

impl State {
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

    pub(super) fn draw_actions_enabled(&self) -> bool {
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
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
