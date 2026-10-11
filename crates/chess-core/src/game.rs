use crate::{Move, Position, PositionStatus, Side};

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

/// A current position, played moves, and game-level resignation state.
pub struct Game {
    initial_position: Position,
    position: Position,
    moves: Vec<Move>,
    resigned: Option<Side>,
}

impl Game {
    /// Starts a game from a position without a resignation result.
    pub fn new(position: Position) -> Self {
        Self {
            initial_position: position.clone(),
            position,
            moves: Vec::new(),
            resigned: None,
        }
    }

    /// Returns the current position without allowing history to be bypassed.
    pub fn position(&self) -> &Position {
        &self.position
    }

    /// Reconstructs the position after `ply` recorded moves without changing the game.
    /// Zero returns the supplied starting position; out-of-range indices return `None`.
    pub fn position_at(&self, ply: usize) -> Option<Position> {
        let moves = self.moves.get(..ply)?;
        let mut position = self.initial_position.clone();
        for &mv in moves {
            assert!(position.play(mv), "recorded moves must remain legal");
        }
        Some(position)
    }

    /// Moves successfully played since this game was created, in play order.
    pub fn moves(&self) -> &[Move] {
        &self.moves
    }

    /// Plays and records a legal move. Finished games reject further moves.
    pub fn play(&mut self, mv: Move) -> bool {
        if self.status() != GameStatus::Ongoing || !self.position.play(mv) {
            return false;
        }
        self.moves.push(mv);
        true
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
            assert!(game.play(Move {
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
            assert!(game.play(Move {
                from,
                to,
                promotion: Option::None
            }));
        }
        assert_eq!(game.status(), GameStatus::Checkmate(Side::Black));
    }

    #[test]
    fn history_records_only_successful_moves() {
        use crate::Square::{E2, E4};
        let mut game = Game::new(Position::standard());
        assert!(game.moves().is_empty());
        let white = Move {
            from: E2,
            to: E4,
            promotion: None,
        };
        let black = Move {
            from: E7,
            to: E5,
            promotion: None,
        };
        assert!(!game.play(black));
        assert!(game.moves().is_empty());
        assert!(game.play(white));
        assert!(!game.play(white));
        assert!(game.play(black));
        assert_eq!(game.moves(), &[white, black]);
        assert!(game.resign(Side::White));
        let before = game.position().legal_moves();
        assert!(!game.play(Move {
            from: G2,
            to: G4,
            promotion: None
        }));
        assert_eq!(game.position().legal_moves(), before);
        assert_eq!(game.moves(), &[white, black]);
        game = Game::new(Position::standard());
        assert!(game.moves().is_empty());
    }

    #[test]
    fn history_round_trips_special_moves_and_all_position_state() {
        use crate::{Piece, Square::*};
        let sequences = [
            vec![(E2, E4), (E7, E5), (G1, F3), (B8, C6), (F1, C4), (G8, F6), (E1, G1)],
            vec![(E2, E4), (A7, A6), (E4, E5), (D7, D5), (E5, D6)],
            vec![
                (A2, A4),
                (H7, H5),
                (A4, A5),
                (H5, H4),
                (A5, A6),
                (H4, H3),
                (A6, B7),
                (H3, G2),
            ],
        ];
        for sequence in sequences {
            for promotion in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
                let mut game = Game::new(Position::standard());
                let mut expected = vec![game.position().clone()];
                for &(from, to) in &sequence {
                    assert!(game.play(Move {
                        from,
                        to,
                        promotion: Option::None
                    }));
                    expected.push(game.position().clone());
                }
                if sequence.last() == Some(&(H3, G2)) {
                    assert!(game.play(Move {
                        from: B7,
                        to: A8,
                        promotion: Some(promotion)
                    }));
                    expected.push(game.position().clone());
                }
                for (ply, position) in expected.iter().enumerate().rev() {
                    assert_eq!(game.position_at(ply).as_ref(), Some(position));
                }
                assert_eq!(game.position_at(expected.len()), Option::None);
                assert_eq!(game.position(), expected.last().unwrap());
            }
        }
    }

    #[test]
    fn history_starts_at_the_supplied_position_not_the_standard_board() {
        use crate::Square::{E2, E4};
        let mut initial = Position::standard();
        assert!(initial.play(Move {
            from: E2,
            to: E4,
            promotion: None
        }));
        let mut game = Game::new(initial.clone());
        assert!(game.play(Move {
            from: E7,
            to: E5,
            promotion: None
        }));
        assert_eq!(game.position_at(0), Some(initial));
        assert_eq!(game.position_at(1).as_ref(), Some(game.position()));
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
