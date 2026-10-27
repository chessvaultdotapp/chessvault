use iced::{
    Element, Fill, Font,
    widget::{Space, button, checkbox, column, container, row, scrollable, text, text_editor},
};

use super::{Message, State};

impl State {
    pub(crate) fn view(&self) -> Element<'_, Message> {
        let available_sources = self.logs.sources();
        let has_other_sources = available_sources.iter().any(|(source, _)| source != "chessvault");
        let header = row![
            text("Developer console").size(18),
            Space::new().width(Fill),
            button(if self.sources_open { "Hide sources" } else { "Sources" })
                .on_press_maybe(has_other_sources.then_some(Message::ToggleSources)),
            button("Copy all").on_press(Message::Copy),
            button("Clear").on_press(Message::Clear),
            button("Close (F12)").on_press(Message::Toggle),
        ]
        .spacing(12)
        .align_y(iced::Center);

        let editor = text_editor(&self.content)
            .on_action(Message::Action)
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

        console.into()
    }
}
