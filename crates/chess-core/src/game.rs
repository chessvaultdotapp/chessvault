use crate::{Position, PositionStatus, Side};

/// Game outcome, including results that cannot be inferred from the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    /// Currently detected only for stalemate; other draw rules are not adjudicated.
    Draw,
    /// The associated side is the checkmated player, not the winner.
    Checkmate(Side),
    Resigned(Side),
}

/// A current position and its game-level resignation state.
pub struct Game {
    pub position: Position,
    resigned: Option<Side>,
}

impl Game {
    /// Starts a game from a position without a resignation result.
    pub fn new(position: Position) -> Self {
        Self {
            position,
            resigned: None,
        }
    }

    /// Records resignation by a player. Rejects the empty-side sentinel.
    pub fn resign(&mut self, side: Side) -> bool {
        if side == Side::Empty || self.status() != GameStatus::Ongoing {
            return false;
        }
        self.resigned = Some(side);
        true
    }

    /// Derives the outcome from the current position unless a player resigned.
    pub fn status(&self) -> GameStatus {
        if let Some(side) = self.resigned {
            return GameStatus::Resigned(side);
        }
        match self.position.status() {
            PositionStatus::Ongoing => GameStatus::Ongoing,
            PositionStatus::Checkmate => GameStatus::Checkmate(self.position.side_to_move()),
            PositionStatus::Stalemate => GameStatus::Draw,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Move,
        Square::{D8, E5, E7, F2, F3, G2, G4, H4},
    };

    #[test]
    fn status_follows_moves_and_replacement() {
        let mut game = Game::new(Position::standard());
        assert_eq!(game.status(), GameStatus::Ongoing);
        for (from, to) in [(F2, F3), (E7, E5), (G2, G4), (D8, H4)] {
            assert!(game.position.play(Move {
                from,
                to,
                promotion: None
            }));
        }
        assert_eq!(game.status(), GameStatus::Checkmate(Side::White));
        assert!(!game.resign(Side::White));
        game = Game::new(Position::standard());
        assert_eq!(game.status(), GameStatus::Ongoing);
    }

    #[test]
    fn checkmate_identifies_black() {
        use crate::Square::*;
        let mut game = Game::new(Position::standard());
        for (from, to) in [(E2, E4), (E7, E5), (F1, C4), (B8, C6), (D1, H5), (G8, F6), (H5, F7)] {
            assert!(game.position.play(Move {
                from,
                to,
                promotion: Option::None
            }));
        }
        assert_eq!(game.status(), GameStatus::Checkmate(Side::Black));
    }

    #[test]
    fn resignation_is_game_state() {
        for side in [Side::White, Side::Black] {
            let mut game = Game::new(Position::standard());
            assert!(!game.resign(Side::Empty));
            assert_eq!(game.status(), GameStatus::Ongoing);
            assert!(game.resign(side));
            assert_eq!(game.status(), GameStatus::Resigned(side));
            assert_eq!(game.position.status(), PositionStatus::Ongoing);
            assert!(!game.resign(side));
        }
    }
}
