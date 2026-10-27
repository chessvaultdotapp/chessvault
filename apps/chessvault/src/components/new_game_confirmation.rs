use iced::{
    Element, Fill,
    widget::{button, column, container, row, text},
};

pub(crate) fn view<'a, Message: Clone + 'a>(on_cancel: Message, on_confirm: Message) -> Element<'a, Message> {
    container(
        column![
            text("Start a new game?").size(24),
            text("The current game and its move history will be discarded."),
            row![
                button("Cancel").on_press(on_cancel),
                button("Start new game").on_press(on_confirm),
            ]
            .spacing(12),
        ]
        .spacing(16),
    )
    .padding(24)
    .center_x(Fill)
    .center_y(Fill)
    .into()
}
