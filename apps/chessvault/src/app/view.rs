use iced::{
    Element, Fill, Font,
    widget::{Space, button, checkbox, column, container, row, scrollable, text, text_editor},
};

use super::{Message, State};
use crate::components::{board, game_controls, new_game_confirmation};

impl State {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        if self.confirm_new_game {
            return new_game_confirmation::view(Message::CancelNewGame, Message::ConfirmNewGame);
        }
        let navigation_enabled = !self.console_open && self.pending_promotion.is_none();
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
