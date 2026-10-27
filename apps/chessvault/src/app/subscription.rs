use std::time::Duration;

use iced::{
    Subscription,
    keyboard::{self, Key, key::Named},
    window,
};

use super::{Message, State};
use crate::history::Navigation;

impl State {
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
}
