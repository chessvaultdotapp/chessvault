use chess_core::{GameStatus, Side};
use iced::{
    Element, Fill, Font,
    widget::{Space, button, checkbox, column, container, row, scrollable, text, text_editor},
};

use super::state::State;
use crate::{Message, board, game_status};

impl State {
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
