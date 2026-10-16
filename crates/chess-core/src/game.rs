use crate::{Move, Position, PositionStatus, Side, position::repetition::RepetitionKey};

/// The rule that ended a game in a draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawReason {
    Agreement,
    Stalemate,
    /// Neither player can checkmate by any legal sequence.
    /// Currently detected only for the material cases in [`PositionStatus::DeadPosition`].
    DeadPosition,
    /// Threefold repetition claimed by the associated player.
    ThreefoldRepetition(Side),
    FivefoldRepetition,
    /// Fifty-move rule claimed by the associated player.
    FiftyMoveRule(Side),
    SeventyFiveMoveRule,
}

/// Game outcome, including results that cannot be inferred from the board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    Draw(DrawReason),
    /// The associated side is the checkmated player, not the winner.
    Checkmate(Side),
    Resigned(Side),
}

/// A current position, played moves, repetition history, and recorded results.
pub struct Game {
    initial_position: Position,
    position: Position,
    moves: Vec<Move>,
    // Includes the initial position; always one entry longer than `moves`.
    position_keys: Vec<RepetitionKey>,
    resigned: Option<Side>,
    claimed_draw: Option<DrawReason>,
    draw_offer: Option<Side>,
}

impl Game {
    /// Starts a game from a position with no recorded resignation or draw claim.
    pub fn new(position: Position) -> Self {
        Self {
            position_keys: vec![position.repetition_key()],
            initial_position: position.clone(),
            position,
            moves: Vec::new(),
            resigned: None,
            claimed_draw: None,
            draw_offer: None,
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

    /// The player whose draw offer is awaiting the opponent's response.
    pub fn draw_offer(&self) -> Option<Side> {
        self.draw_offer
    }

    /// Offers a draw on the player's turn. An outstanding offer cannot be withdrawn
    /// or replaced. It survives the offering player's move.
    pub fn offer_draw(&mut self, side: Side) -> bool {
        if side == Side::Empty
            || side != self.position.side_to_move()
            || self.status() != GameStatus::Ongoing
            || self.draw_offer.is_some()
        {
            return false;
        }
        self.draw_offer = Some(side);
        true
    }

    fn can_respond_to_draw(&self, side: Side) -> bool {
        side != Side::Empty
            && self.status() == GameStatus::Ongoing
            && self.draw_offer.is_some_and(|offering| offering != side)
    }

    /// Accepts the opponent's offer without changing the board or move history.
    /// Acceptance is allowed before or after the offering player's move.
    pub fn accept_draw(&mut self, side: Side) -> bool {
        if !self.can_respond_to_draw(side) {
            return false;
        }
        self.claimed_draw = Some(DrawReason::Agreement);
        self.draw_offer = None;
        true
    }

    /// Declines the opponent's offer. Playing a legal move also declines it.
    pub fn decline_draw(&mut self, side: Side) -> bool {
        if !self.can_respond_to_draw(side) {
            return false;
        }
        self.draw_offer = None;
        true
    }

    /// Occurrences of the current position, including the supplied starting position.
    /// Only history recorded by this game is counted; this does not adjudicate a draw.
    pub fn repetition_count(&self) -> usize {
        let current = self.position_keys.last().expect("initial position key");
        self.position_keys.iter().filter(|key| *key == current).count()
    }

    /// Whether the side to move can claim repetition or the fifty-move rule now.
    pub fn can_claim_draw(&self) -> bool {
        self.status() == GameStatus::Ongoing && (self.repetition_count() >= 3 || self.position.halfmove_clock() >= 100)
    }

    /// Claims a draw for the side to move, without playing a move.
    pub fn claim_draw(&mut self) -> bool {
        if !self.can_claim_draw() {
            return false;
        }
        self.claimed_draw = Some(self.claim_reason(self.position.halfmove_clock()));
        self.draw_offer = None;
        true
    }

    fn claim_reason(&self, clock: u8) -> DrawReason {
        if clock >= 100 {
            DrawReason::FiftyMoveRule(self.position.side_to_move())
        } else {
            DrawReason::ThreefoldRepetition(self.position.side_to_move())
        }
    }

    /// Whether an intended legal move permits repetition or a fifty-move claim.
    /// Neither the move nor its resulting position is recorded.
    pub fn can_claim_draw_with_move(&self, mv: Move) -> bool {
        if self.status() != GameStatus::Ongoing {
            return false;
        }
        let mut next = self.position.clone();
        if !next.play(mv) {
            return false;
        }
        let key = next.repetition_key();
        next.halfmove_clock() >= 100 || self.position_keys.iter().filter(|&&previous| previous == key).count() >= 2
    }

    /// Claims based on an intended move, leaving the board and history unchanged.
    pub fn claim_draw_with_move(&mut self, mv: Move) -> bool {
        if !self.can_claim_draw_with_move(mv) {
            return false;
        }
        let mut next = self.position.clone();
        assert!(next.play(mv), "validated intended move");
        self.claimed_draw = Some(self.claim_reason(next.halfmove_clock()));
        self.draw_offer = None;
        true
    }

    /// Plays and records a legal move. Finished games reject further moves.
    pub fn play(&mut self, mv: Move) -> bool {
        if self.status() != GameStatus::Ongoing || !self.position.play(mv) {
            return false;
        }
        self.moves.push(mv);
        self.position_keys.push(self.position.repetition_key());
        // After a move the side to move is the opponent of the player who moved.
        if self.draw_offer == Some(self.position.side_to_move()) || self.status() != GameStatus::Ongoing {
            self.draw_offer = None;
        }
        true
    }

    /// Records resignation by a player. Rejects the empty-side sentinel.
    pub fn resign(&mut self, side: Side) -> bool {
        if side == Side::Empty || self.status() != GameStatus::Ongoing {
            return false;
        }
        self.resigned = Some(side);
        self.draw_offer = None;
        true
    }

    /// Derives the outcome from the position and repetition history, or a recorded result.
    pub fn status(&self) -> GameStatus {
        if let Some(side) = self.resigned {
            return GameStatus::Resigned(side);
        }
        if let Some(reason) = self.claimed_draw {
            return GameStatus::Draw(reason);
        }
        match self.position.status() {
            PositionStatus::Checkmate => GameStatus::Checkmate(self.position.side_to_move()),
            PositionStatus::Stalemate => GameStatus::Draw(DrawReason::Stalemate),
            PositionStatus::DeadPosition => GameStatus::Draw(DrawReason::DeadPosition),
            PositionStatus::Ongoing if self.position.halfmove_clock() >= 150 => {
                GameStatus::Draw(DrawReason::SeventyFiveMoveRule)
            }
            PositionStatus::Ongoing if self.repetition_count() >= 5 => GameStatus::Draw(DrawReason::FivefoldRepetition),
            PositionStatus::Ongoing => GameStatus::Ongoing,
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

    fn play_knight(game: &mut Game, side: Side) {
        use crate::Square::{F3, F6, G1, G8};
        let (from, to) = if side == Side::White { (G1, F3) } else { (G8, F6) };
        assert!(game.play(Move {
            from,
            to,
            promotion: None
        }));
    }

    #[test]
    fn agreement_preserves_board_and_history_and_finishes_game() {
        for offering in [Side::White, Side::Black] {
            for move_first in [false, true] {
                let mut game = Game::new(Position::standard());
                if offering == Side::Black {
                    play_knight(&mut game, Side::White);
                }
                let responding = if offering == Side::White {
                    Side::Black
                } else {
                    Side::White
                };
                assert!(game.offer_draw(offering));
                if move_first {
                    play_knight(&mut game, offering);
                }
                assert_eq!(game.draw_offer(), Some(offering));
                let position = game.position().clone();
                let moves = game.moves().to_vec();
                assert!(!game.accept_draw(offering));
                assert!(!game.accept_draw(Side::Empty));
                assert!(game.accept_draw(responding));
                assert_eq!(game.status(), GameStatus::Draw(DrawReason::Agreement));
                assert_eq!(game.draw_offer(), None);
                assert_eq!(game.position(), &position);
                assert_eq!(game.moves(), moves);
                assert!(!game.play(game.position().legal_moves()[0]));
                for side in [Side::White, Side::Black, Side::Empty] {
                    assert!(!game.offer_draw(side));
                    assert!(!game.accept_draw(side));
                    assert!(!game.decline_draw(side));
                    assert!(!game.resign(side));
                }
                assert!(!game.claim_draw());
            }
        }
    }

    #[test]
    fn offers_require_the_turn_and_opponents_response() {
        let mut game = Game::new(Position::standard());
        for side in [Side::White, Side::Black, Side::Empty] {
            assert!(!game.accept_draw(side));
            assert!(!game.decline_draw(side));
        }
        assert!(!game.offer_draw(Side::Empty));
        assert!(!game.offer_draw(Side::Black));
        assert!(game.offer_draw(Side::White));
        assert!(!game.offer_draw(Side::White));
        assert!(!game.decline_draw(Side::White));
        assert!(!game.decline_draw(Side::Empty));
        play_knight(&mut game, Side::White);
        assert!(!game.offer_draw(Side::Black));
        assert!(game.decline_draw(Side::Black));
        assert_eq!(game.draw_offer(), None);
        assert_eq!(game.status(), GameStatus::Ongoing);
        assert!(game.offer_draw(Side::Black));
        assert!(game.decline_draw(Side::White));
    }

    #[test]
    fn only_successful_opponent_moves_implicitly_decline_offers() {
        let mut game = Game::new(Position::standard());
        assert!(game.offer_draw(Side::White));
        play_knight(&mut game, Side::White);
        assert!(!game.play(Move {
            from: crate::Square::G8,
            to: crate::Square::G5,
            promotion: None
        }));
        assert_eq!(game.draw_offer(), Some(Side::White));
        play_knight(&mut game, Side::Black);
        assert_eq!(game.draw_offer(), None);
        assert!(!game.accept_draw(Side::Black));
    }

    #[test]
    fn other_results_clear_offers() {
        let mut game = Game::new(Position::standard());
        assert!(game.offer_draw(Side::White));
        assert!(game.resign(Side::Black));
        assert_eq!(game.draw_offer(), None);
        assert!(!game.accept_draw(Side::Black));
        assert!(!game.offer_draw(Side::White));

        let mut game = Game::new(Position::standard());
        shuffle(&mut game);
        shuffle(&mut game);
        assert!(game.offer_draw(Side::White));
        assert!(game.claim_draw());
        assert_eq!(game.draw_offer(), None);

        let mut game = Game::new(Position::standard());
        for (from, to) in [(F2, F3), (E7, E5), (G2, G4)] {
            assert!(game.play(Move {
                from,
                to,
                promotion: None
            }));
        }
        assert!(game.offer_draw(Side::Black));
        assert!(game.play(Move {
            from: D8,
            to: H4,
            promotion: None
        }));
        assert_eq!(game.status(), GameStatus::Checkmate(Side::White));
        assert_eq!(game.draw_offer(), None);
        assert!(!game.accept_draw(Side::White));
    }

    fn shuffle(game: &mut Game) {
        use crate::Square::{F3, F6, G1, G8};
        for (from, to) in [(G1, F3), (G8, F6), (F3, G1), (F6, G8)] {
            assert!(game.play(Move {
                from,
                to,
                promotion: None
            }));
        }
    }

    #[test]
    fn threefold_and_fourfold_require_a_claim() {
        for cycles in [2, 3] {
            for black_to_move in [false, true] {
                let mut game = Game::new(Position::standard());
                if black_to_move {
                    assert!(game.play(Move {
                        from: crate::Square::A2,
                        to: crate::Square::A3,
                        promotion: None
                    }));
                }
                assert!(!game.can_claim_draw());
                assert!(!game.claim_draw());
                // Use the same cycle shifted by one ply for Black's turn.
                use crate::Square::{F3, F6, G1, G8};
                let cycle = if black_to_move {
                    [(G8, F6), (G1, F3), (F6, G8), (F3, G1)]
                } else {
                    [(G1, F3), (G8, F6), (F3, G1), (F6, G8)]
                };
                for _ in 0..cycles {
                    for (from, to) in cycle {
                        assert!(game.play(Move {
                            from,
                            to,
                            promotion: None
                        }));
                    }
                }
                assert_eq!(game.status(), GameStatus::Ongoing);
                assert!(game.can_claim_draw());
                let side = game.position().side_to_move();
                let before = game.position().clone();
                let moves = game.moves().to_vec();
                assert!(game.claim_draw());
                assert_eq!(game.status(), GameStatus::Draw(DrawReason::ThreefoldRepetition(side)));
                assert_eq!(game.position(), &before);
                assert_eq!(game.moves(), moves);
                assert!(!game.can_claim_draw());
                assert!(!game.claim_draw());
                assert!(!game.resign(side));
                assert!(!game.play(Move {
                    from: cycle[0].0,
                    to: cycle[0].1,
                    promotion: None
                }));
            }
        }
    }

    #[test]
    fn fifth_occurrence_is_automatic() {
        let mut game = Game::new(Position::standard());
        for _ in 0..4 {
            shuffle(&mut game);
        }
        assert_eq!(game.repetition_count(), 5);
        assert_eq!(game.status(), GameStatus::Draw(DrawReason::FivefoldRepetition));
        assert!(!game.claim_draw());
        assert!(!game.resign(Side::White));
        assert!(!game.play(Move {
            from: crate::Square::E2,
            to: crate::Square::E4,
            promotion: None
        }));
    }

    #[test]
    fn intended_move_claim_does_not_play_the_move() {
        use crate::Square::{F3, F6, G1, G8};
        let mut game = Game::new(Position::standard());
        let intended = Move {
            from: F6,
            to: G8,
            promotion: None,
        };
        assert!(!game.can_claim_draw_with_move(intended));
        assert!(!game.claim_draw_with_move(intended));
        shuffle(&mut game);
        for (from, to) in [(G1, F3), (G8, F6), (F3, G1)] {
            assert!(game.play(Move {
                from,
                to,
                promotion: None
            }));
        }
        assert!(!game.can_claim_draw());
        let before = game.position().clone();
        let keys = game.position_keys.clone();
        let moves = game.moves().to_vec();
        assert!(game.can_claim_draw_with_move(intended));
        assert!(game.claim_draw_with_move(intended));
        assert_eq!(
            game.status(),
            GameStatus::Draw(DrawReason::ThreefoldRepetition(Side::Black))
        );
        assert_eq!(game.position(), &before);
        assert_eq!(game.position_keys, keys);
        assert_eq!(game.moves(), moves);
        assert!(!game.claim_draw_with_move(intended));
    }

    #[test]
    fn repetition_tracks_only_played_positions() {
        use crate::Square::{E2, E4, E5, E7, F3, F6, G1, G8};
        let mut game = Game::new(Position::standard());
        assert_eq!(game.repetition_count(), 1);
        for expected in 2..=3 {
            for (from, to) in [(G1, F3), (G8, F6), (F3, G1), (F6, G8)] {
                assert!(game.play(Move {
                    from,
                    to,
                    promotion: None
                }));
                assert_eq!(game.position_keys.len(), game.moves.len() + 1);
            }
            assert_eq!(game.repetition_count(), expected);
        }
        let keys = game.position_keys.clone();
        assert!(!game.play(Move {
            from: E7,
            to: E5,
            promotion: None
        }));
        assert!(game.position_at(0).is_some());
        assert!(game.resign(Side::White));
        assert!(!game.play(Move {
            from: E2,
            to: E4,
            promotion: None
        }));
        assert_eq!(game.position_keys, keys);
        assert_eq!(game.repetition_count(), 3);
        game = Game::new(Position::standard());
        assert_eq!(game.repetition_count(), 1);
    }

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
