use chess_core::{DrawReason, Game, GameStatus, Position, PositionStatus, Side};
use iced::{
    Element, Fill, Task,
    keyboard::key::Named,
    widget::{button, column, row, scrollable, text},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Navigation {
    First,
    Previous,
    Next,
    Latest,
    Ply(usize),
}

impl Navigation {
    pub fn from_key(key: Named) -> Option<Self> {
        match key {
            Named::ArrowLeft => Some(Self::Previous),
            Named::ArrowRight => Some(Self::Next),
            Named::ArrowUp | Named::Home => Some(Self::First),
            Named::ArrowDown | Named::End => Some(Self::Latest),
            _ => None,
        }
    }
}

/// The live game is never rewound. A review holds an independent reconstructed position.
#[derive(Default)]
pub struct History {
    review: Option<(usize, Position)>,
    labels: Vec<String>,
}

impl History {
    pub fn is_reviewing(&self) -> bool {
        self.review.is_some()
    }

    pub fn ply(&self, game: &Game) -> usize {
        self.review.as_ref().map_or(game.moves().len(), |(ply, _)| *ply)
    }

    pub fn position<'a>(&'a self, game: &'a Game) -> &'a Position {
        self.review.as_ref().map_or(game.position(), |(_, position)| position)
    }

    pub fn status(&self, game: &Game) -> GameStatus {
        match &self.review {
            None => game.status(),
            Some((_, position)) => match position.status() {
                PositionStatus::Ongoing => GameStatus::Ongoing,
                PositionStatus::Checkmate => GameStatus::Checkmate(position.side_to_move()),
                PositionStatus::Stalemate => GameStatus::Draw(DrawReason::Stalemate),
                PositionStatus::DeadPosition => GameStatus::Draw(DrawReason::DeadPosition),
            },
        }
    }

    /// Cache notation only when new moves arrive, not on redraws or console refreshes.
    pub fn sync(&mut self, game: &Game) {
        if self.labels.len() == game.moves().len() {
            return;
        }
        let mut position = game
            .position_at(self.labels.len())
            .expect("history belongs to this game");
        for &mv in &game.moves()[self.labels.len()..] {
            let san = position.san(mv).expect("recorded move is legal");
            let label = if position.side_to_move() == Side::White {
                format!("{}. {san}", position.fullmove_number())
            } else {
                san
            };
            self.labels.push(label);
            assert!(position.play(mv));
        }
    }

    pub fn navigate(&mut self, game: &Game, navigation: Navigation) {
        let latest = game.moves().len();
        let current = self.ply(game);
        let target = match navigation {
            Navigation::First => 0,
            Navigation::Previous => current.saturating_sub(1),
            Navigation::Next => current.saturating_add(1).min(latest),
            Navigation::Latest => latest,
            Navigation::Ply(ply) => ply.min(latest),
        };
        if target == latest {
            self.review = None;
        } else if target != current {
            self.review = Some((target, game.position_at(target).expect("bounded history index")));
        }
    }

    pub fn scroll_to_current<Message>(&self, game: &Game) -> Task<Message> {
        let ply = self.ply(game);
        // Two moves per 34-pixel row; the starting position has no list entry.
        let row = ply.saturating_sub(1) / 2;
        iced::widget::operation::scroll_to(
            "move-history",
            scrollable::AbsoluteOffset {
                x: 0.0,
                y: row.saturating_sub(2) as f32 * 34.0,
            },
        )
    }

    pub fn controls<'a, Message: Clone + 'a>(
        &self,
        game: &Game,
        enabled: bool,
        on_navigate: impl Fn(Navigation) -> Message,
    ) -> Element<'a, Message> {
        let ply = self.ply(game);
        let latest = game.moves().len();
        let control = |label, action, available| {
            button(text(label).size(14))
                .padding(6)
                .on_press_maybe((enabled && available).then_some(on_navigate(action)))
        };
        let buttons = row![
            control("First", Navigation::First, ply > 0),
            control("Previous", Navigation::Previous, ply > 0),
            control("Next", Navigation::Next, ply < latest),
            control("Latest", Navigation::Latest, ply < latest),
        ]
        .spacing(6);
        let caption = if self.is_reviewing() {
            format!("Reviewing {ply}/{latest} — read-only")
        } else {
            String::new()
        };
        // Reserve the return-action row so the move list stays still during review.
        let return_action: Element<'_, Message> = if self.is_reviewing() {
            control("Return to latest", Navigation::Latest, true).into()
        } else {
            iced::widget::Space::new().into()
        };
        column![
            buttons,
            // Keep the status row stable between live and review modes.
            iced::widget::container(text(caption).size(14)).height(20),
            iced::widget::container(return_action).height(32),
        ]
        .spacing(6)
        .width(Fill)
        .into()
    }

    pub fn move_list<'a, Message: Clone + 'a>(
        &self,
        game: &Game,
        enabled: bool,
        on_navigate: impl Fn(Navigation) -> Message,
    ) -> Element<'a, Message> {
        let current = self.ply(game);
        let entry = |label: String, ply| {
            button(text(label).size(16))
                .height(32)
                .width(Fill)
                .style(if current == ply { button::primary } else { button::text })
                .on_press_maybe(enabled.then_some(on_navigate(Navigation::Ply(ply))))
        };
        let mut moves = column![].spacing(2);
        for (index, pair) in self.labels.chunks(2).enumerate() {
            let mut line = row![entry(pair[0].clone(), index * 2 + 1)].spacing(2);
            if let Some(black) = pair.get(1) {
                line = line.push(entry(black.clone(), index * 2 + 2));
            } else {
                line = line.push(iced::widget::Space::new().width(Fill));
            }
            moves = moves.push(line);
        }
        scrollable(moves).id("move-history").height(Fill).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::{Move, Square::*};

    #[test]
    fn views_and_scroll_tasks_accept_component_consumer_messages() {
        let game = Game::new(Position::standard());
        let history = History::default();
        let _: Element<'_, Navigation> = history.controls(&game, true, std::convert::identity);
        let _: Element<'_, Navigation> = history.move_list(&game, true, std::convert::identity);
        let _: Task<Navigation> = history.scroll_to_current(&game);
    }

    #[test]
    fn keys_match_the_navigation_convention() {
        for (key, action) in [
            (Named::ArrowLeft, Navigation::Previous),
            (Named::ArrowRight, Navigation::Next),
            (Named::ArrowUp, Navigation::First),
            (Named::Home, Navigation::First),
            (Named::ArrowDown, Navigation::Latest),
            (Named::End, Navigation::Latest),
        ] {
            assert_eq!(Navigation::from_key(key), Some(action));
        }
        assert_eq!(Navigation::from_key(Named::Escape), Option::None);
    }

    #[test]
    fn navigation_is_bounded_and_preserves_the_live_game_and_result() {
        let mut game = Game::new(Position::standard());
        let mut history = History::default();
        for action in [
            Navigation::First,
            Navigation::Previous,
            Navigation::Next,
            Navigation::Latest,
        ] {
            history.navigate(&game, action);
            assert!(!history.is_reviewing());
            assert_eq!(history.ply(&game), 0);
        }
        for (from, to) in [(E2, E4), (E7, E5), (G1, F3)] {
            assert!(game.play(Move {
                from,
                to,
                promotion: Option::None
            }));
        }
        assert!(game.resign(Side::Black));
        history.sync(&game);
        history.sync(&game);
        assert_eq!(history.labels, ["1. e4", "e5", "2. Nf3"]);
        let live = game.position().clone();
        for (action, expected) in [
            (Navigation::Previous, 2),
            (Navigation::First, 0),
            (Navigation::Previous, 0),
            (Navigation::Next, 1),
            (Navigation::Ply(2), 2),
            (Navigation::Latest, 3),
            (Navigation::Next, 3),
            (Navigation::Ply(usize::MAX), 3),
        ] {
            history.navigate(&game, action);
            assert_eq!(history.ply(&game), expected);
            assert_eq!(history.position(&game), &game.position_at(expected).unwrap());
            assert_eq!(
                history.status(&game),
                if expected == 3 {
                    GameStatus::Resigned(Side::Black)
                } else {
                    GameStatus::Ongoing
                }
            );
            assert_eq!(game.position(), &live);
            assert_eq!(game.moves().len(), 3);
        }
    }
}
