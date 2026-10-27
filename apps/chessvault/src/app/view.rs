use iced::{
    Element, Fill,
    widget::{column, container, row, text},
};

use super::{Message, State};
use crate::components::{board, game_controls, new_game_confirmation};

impl State {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        if self.confirm_new_game {
            return new_game_confirmation::view(Message::CancelNewGame, Message::ConfirmNewGame);
        }
        let navigation_enabled = !self.console.open && self.pending_promotion.is_none();
        let controls = game_controls::view(game_controls::Props {
            game: &self.game,
            status: self.history.status(&self.game),
            reviewing: self.history.is_reviewing(),
            navigation_enabled,
            draw_actions_enabled: self.draw_actions_enabled(),
            saving_pgn: self.saving_pgn,
            pgn_error: self.pgn_error.as_deref(),
        });
        let controls = iced::widget::Column::from_vec(vec![
            iced::Element::from(controls).map(Message::from),
            text("Move history").size(20).into(),
            self.history.controls(&self.game, navigation_enabled, Message::Navigate),
            self.history
                .move_list(&self.game, navigation_enabled, Message::Navigate),
        ])
        .spacing(12)
        .height(Fill);

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

        if !self.console.open {
            return content.into();
        }

        content.push(self.console.view().map(Message::Console)).into()
    }
}
