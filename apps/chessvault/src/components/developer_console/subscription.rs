use std::time::Duration;

use iced::Subscription;

use super::{Message, State};

impl State {
    pub(crate) fn subscription(&self) -> Subscription<Message> {
        if self.open {
            iced::time::every(Duration::from_millis(250)).map(|_| Message::Refresh)
        } else {
            Subscription::none()
        }
    }
}
