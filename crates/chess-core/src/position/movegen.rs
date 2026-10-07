use super::*;

/// A move identified by its origin, destination, and optional promotion piece.
/// Castling uses the king's squares; en passant uses the pawn's landing square.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    /// Queen, rook, bishop, or knight for a promotion; otherwise `None`.
    pub promotion: Option<Piece>,
}

/// Position-based status only; repetition, move-count and dead-position draws are not evaluated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionStatus {
    Ongoing,
    Checkmate,
    Stalemate,
}

type Board = [Option<(Piece, Side)>; 64];

fn square(index: usize) -> Square {
    Square::try_from(index as u8).expect("board index")
}

fn opponent(side: Side) -> Side {
    match side {
        Side::White => Side::Black,
        Side::Black => Side::White,
        Side::Empty => Side::Empty,
    }
}

// Geometric attacks deliberately include pinned pieces and exclude castling.
fn attacks(board: &Board, from: usize, to: usize) -> bool {
    let Some((piece, side)) = board[from] else { return false };
    let dx = (to % 8) as i32 - (from % 8) as i32;
    let dy = (to / 8) as i32 - (from / 8) as i32;
    if from == to {
        return false;
    }
    match piece {
        Piece::Pawn => return dx.abs() == 1 && dy == if side == Side::White { 1 } else { -1 },
        Piece::Knight => return matches!((dx.abs(), dy.abs()), (1, 2) | (2, 1)),
        Piece::King => return dx.abs().max(dy.abs()) == 1,
        Piece::Bishop if dx.abs() == dy.abs() => {}
        Piece::Rook if dx == 0 || dy == 0 => {}
        Piece::Queen if dx == 0 || dy == 0 || dx.abs() == dy.abs() => {}
        _ => return false,
    }
    let step = dx.signum() + 8 * dy.signum();
    let mut at = from as i32 + step;
    while at != to as i32 {
        if board[at as usize].is_some() {
            return false;
        }
        at += step;
    }
    true
}

fn attacked(board: &Board, to: usize, by: Side) -> bool {
    (0..64).any(|from| board[from].is_some_and(|(_, side)| side == by) && attacks(board, from, to))
}

fn safe(board: &Board, side: Side) -> bool {
    board
        .iter()
        .position(|entry| *entry == Some((Piece::King, side)))
        .is_some_and(|king| !attacked(board, king, opponent(side)))
}

impl Position {
    fn board(&self) -> Board {
        std::array::from_fn(|index| self.piece_at(square(index)))
    }

    /// Returns the side whose legal moves are generated.
    pub fn side_to_move(&self) -> Side {
        self.side_to_move
    }

    /// Returns whether the side-to-move king is in check.
    pub fn is_in_check(&self) -> bool {
        !safe(&self.board(), self.side_to_move)
    }

    /// Classifies a legal position by available moves and king safety.
    /// Does not adjudicate other draw rules or game-level results such as resignation.
    pub fn status(&self) -> PositionStatus {
        if !self.legal_moves().is_empty() {
            PositionStatus::Ongoing
        } else if !self.is_in_check() {
            PositionStatus::Stalemate
        } else {
            PositionStatus::Checkmate
        }
    }

    /// Generates all legal moves for the side to move, including all four promotions.
    /// Moves leaving the moving king in check are excluded. Ordering is unspecified.
    pub fn legal_moves(&self) -> Vec<Move> {
        let board = self.board();
        let mut moves = Vec::new();
        if self.side_to_move == Side::Empty {
            return moves;
        }
        for from in 0..64 {
            let Some((piece, side)) = board[from] else { continue };
            if side != self.side_to_move {
                continue;
            }
            for to in 0..64 {
                if from == to || board[to].is_some_and(|(p, s)| s == side || p == Piece::King) {
                    continue;
                }
                let dx = (to % 8) as i32 - (from % 8) as i32;
                let dy = (to / 8) as i32 - (from / 8) as i32;
                let possible = if piece == Piece::Pawn {
                    let direction = if side == Side::White { 1 } else { -1 };
                    let start = if side == Side::White { 1 } else { 6 };
                    if dx == 0 {
                        board[to].is_none()
                            && (dy == direction
                                || (dy == 2 * direction
                                    && from / 8 == start
                                    && board[(from as i32 + 8 * direction) as usize].is_none()))
                    } else {
                        dx.abs() == 1
                            && dy == direction
                            && (board[to].is_some()
                                || (self.en_passant == Some(square(to))
                                    && board[to].is_none()
                                    && board[(to as i32 - 8 * direction) as usize]
                                        == Some((Piece::Pawn, opponent(side)))))
                    }
                } else {
                    attacks(&board, from, to) || (piece == Piece::King && self.can_castle(&board, from, to, side))
                };
                if !possible {
                    continue;
                }
                let promotions: &[Option<Piece>] = if piece == Piece::Pawn && (to / 8 == 0 || to / 8 == 7) {
                    &[
                        Some(Piece::Queen),
                        Some(Piece::Rook),
                        Some(Piece::Bishop),
                        Some(Piece::Knight),
                    ]
                } else {
                    &[None]
                };
                for &promotion in promotions {
                    let mv = Move {
                        from: square(from),
                        to: square(to),
                        promotion,
                    };
                    if safe(&self.moved_board(board, mv), side) {
                        moves.push(mv);
                    }
                }
            }
        }
        moves
    }

    fn can_castle(&self, board: &Board, from: usize, to: usize, side: Side) -> bool {
        let base = if side == Side::White { 0 } else { 56 };
        if from != base + 4 || ![base + 2, base + 6].contains(&to) {
            return false;
        }
        let kingside = to == base + 6;
        let mask = match (side, kingside) {
            (Side::White, true) => CastlingRights::WhiteKingside,
            (Side::White, false) => CastlingRights::WhiteQueenside,
            (_, true) => CastlingRights::BlackKingside,
            (_, false) => CastlingRights::BlackQueenside,
        } as u8;
        let rook = base + if kingside { 7 } else { 0 };
        let path = if kingside {
            base + 5..base + 7
        } else {
            base + 1..base + 4
        };
        if self.castling_rights & mask == 0
            || board[rook] != Some((Piece::Rook, side))
            || path.into_iter().any(|i| board[i].is_some())
            || !safe(board, side)
        {
            return false;
        }
        let mut transit = *board;
        transit[from] = None;
        let middle = (from + to) / 2;
        transit[middle] = Some((Piece::King, side));
        safe(&transit, side)
    }

    fn moved_board(&self, mut board: Board, mv: Move) -> Board {
        let from = mv.from as usize;
        let to = mv.to as usize;
        let (piece, side) = board[from].expect("generated move has a piece");
        if piece == Piece::Pawn && from % 8 != to % 8 && board[to].is_none() {
            board[from / 8 * 8 + to % 8] = None;
        }
        if piece == Piece::King && from.abs_diff(to) == 2 {
            let rook = from / 8 * 8 + if to > from { 7 } else { 0 };
            board[(from + to) / 2] = board[rook];
            board[rook] = None;
        }
        board[from] = None;
        board[to] = Some((mv.promotion.unwrap_or(piece), side));
        board
    }

    /// Plays a legal move, returning whether it was accepted.
    /// An illegal move leaves the position unchanged.
    pub fn play(&mut self, mv: Move) -> bool {
        if !self.legal_moves().contains(&mv) {
            return false;
        }
        *self = self.after_move(mv);
        true
    }

    fn after_move(&self, mv: Move) -> Self {
        let board = self.board();
        let (piece, side) = board[mv.from as usize].expect("legal move");
        let capture =
            board[mv.to as usize].is_some() || (piece == Piece::Pawn && mv.from as usize % 8 != mv.to as usize % 8);
        let mut next = Self::empty();
        for (index, entry) in self.moved_board(board, mv).into_iter().enumerate() {
            if let Some((piece, side)) = entry {
                next.pieces[piece as usize].set(square(index)).expect("board square");
                next.sides[side as usize].set(square(index)).expect("board square");
            }
        }
        next.side_to_move = opponent(side);
        next.castling_rights = self.castling_rights;
        for (home, mask) in [(Square::A1, 2), (Square::H1, 1), (Square::A8, 8), (Square::H8, 4)] {
            if mv.from == home || mv.to == home {
                next.castling_rights &= !mask;
            }
        }
        if piece == Piece::King {
            next.castling_rights &= !(if side == Side::White { 3 } else { 12 });
        }
        if piece == Piece::Pawn && (mv.from as usize).abs_diff(mv.to as usize) == 16 {
            next.en_passant = Some(square((mv.from as usize + mv.to as usize) / 2));
        }
        next.halfmove_count = if piece == Piece::Pawn || capture {
            0
        } else {
            self.halfmove_count.saturating_add(1)
        };
        next.fullmove_count = self.fullmove_count.saturating_add(u32::from(side == Side::Black));
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perft(position: &Position, depth: u8) -> usize {
        if depth == 0 {
            return 1;
        }
        position
            .legal_moves()
            .into_iter()
            .map(|mv| perft(&position.after_move(mv), depth - 1))
            .sum()
    }

    #[test]
    fn starting_position_perft() {
        let position = Position::standard();
        for (depth, expected) in [(1, 20), (2, 400), (3, 8902)] {
            assert_eq!(perft(&position, depth), expected);
        }
    }

    fn setup(pieces: &[(Square, Piece, Side)]) -> Position {
        let mut position = Position::empty();
        position.side_to_move = Side::White;
        for &(square, piece, side) in pieces {
            position.pieces[piece as usize].set(square).unwrap();
            position.sides[side as usize].set(square).unwrap();
        }
        position
    }

    #[test]
    fn position_status() {
        use {Piece::*, Side::*, Square::*};
        assert_eq!(Position::standard().status(), PositionStatus::Ongoing);
        assert!(!Position::standard().is_in_check());
        for side in [White, Black] {
            let other = opponent(side);
            let check = setup(&[(A1, King, side), (H8, King, other), (A8, Rook, other)]);
            let mut check = check;
            check.side_to_move = side;
            assert!(check.is_in_check());
            assert_eq!(check.status(), PositionStatus::Ongoing);
            for (queen, expected) in [(B2, PositionStatus::Checkmate), (B3, PositionStatus::Stalemate)] {
                let mut position = setup(&[(A1, King, side), (C2, King, other), (queen, Queen, other)]);
                position.side_to_move = side;
                assert_eq!(position.status(), expected);
                assert_eq!(position.is_in_check(), expected == PositionStatus::Checkmate);
            }
        }
    }

    #[test]
    fn special_moves_and_pins() {
        use {Piece::*, Side::*, Square::*};
        let mut position = setup(&[
            (E1, King, White),
            (H1, Rook, White),
            (E8, King, Black),
            (A7, Pawn, White),
        ]);
        position.castling_rights = 1;
        let castle = Move {
            from: E1,
            to: G1,
            promotion: Option::None,
        };
        assert!(position.legal_moves().contains(&castle));
        assert_eq!(position.legal_moves().iter().filter(|m| m.from == A7).count(), 4);
        assert!(position.play(castle));
        assert_eq!(position.piece_at(F1), Some((Rook, White)));
        assert_eq!(position.castling_rights, 0);

        let mut pinned = setup(&[
            (E1, King, White),
            (E5, Pawn, White),
            (D5, Pawn, Black),
            (E8, Rook, Black),
            (A8, King, Black),
        ]);
        pinned.en_passant = Some(D6);
        assert!(!pinned.legal_moves().iter().any(|m| m.from == E5 && m.to == D6));
    }

    #[test]
    fn en_passant_expires_and_removes_captured_pawn() {
        use {Piece::*, Side::*, Square::*};
        let mut position = setup(&[
            (E1, King, White),
            (E8, King, Black),
            (E5, Pawn, White),
            (D7, Pawn, Black),
        ]);
        position.side_to_move = Black;
        assert!(position.play(Move {
            from: D7,
            to: D5,
            promotion: Option::None
        }));
        assert_eq!(position.en_passant, Some(D6));
        assert!(position.play(Move {
            from: E5,
            to: D6,
            promotion: Option::None
        }));
        assert_eq!(position.piece_at(D5), Option::None);
        assert_eq!(position.en_passant, Option::None);
    }

    #[test]
    fn fools_mate_and_invalid_moves() {
        use Square::*;
        let mut position = Position::standard();
        assert!(!position.play(Move {
            from: None,
            to: E4,
            promotion: Option::None
        }));
        for (from, to) in [(F2, F3), (E7, E5), (G2, G4), (D8, H4)] {
            assert!(position.play(Move {
                from,
                to,
                promotion: Option::None
            }));
        }
        assert!(position.legal_moves().is_empty());
        assert_eq!(position.status(), PositionStatus::Checkmate);
        assert!(Position::empty().legal_moves().is_empty());
    }
}
