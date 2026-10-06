//! Core chess primitives and bitboard-based position storage.
//!
//! Positions store six piece-type bitboards and two side bitboards. The
//! [`Piece::Empty`](piece::Piece::Empty) and [`Side::Empty`](side::Side::Empty)
//! variants represent absence; they are sentinels, not valid indices into those
//! arrays. [`Square::None`](square::Square::None) represents the absence of a
//! square and is not a valid bit index in a [`Bitboard`](bitboard::Bitboard).

mod bitboard;
mod castling_rights;
mod game;
mod piece;
mod position;
mod side;
mod square;

pub use game::{Game, GameStatus};
pub use piece::Piece;
pub use position::{
    Position,
    movegen::{Move, PositionStatus},
};
pub use side::Side;
pub use square::{InvalidSquareIndex, Square};
