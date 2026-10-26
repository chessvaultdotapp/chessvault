//! Chess positions, legal moves, standard algebraic notation, and game results.
//!
//! Construct a [`Position`] with [`Position::standard`]. Positions generate legal
//! moves, apply them without history, and classify board-based outcomes through
//! [`PositionStatus`]. Use [`Game`] for played games: it records moves and repetition
//! history, rejects moves after a result, and handles resignation, draw offers,
//! claims, and automatic draws through [`GameStatus`] and [`DrawReason`].
//!
//! ```
//! use chess_core::{Game, Move, Position, Square};
//!
//! let mut game = Game::new(Position::standard());
//! assert!(game.play(Move {
//!     from: Square::E2,
//!     to: Square::E4,
//!     promotion: None,
//! }));
//! assert_eq!(game.position().side_to_move(), chess_core::Side::Black);
//! ```
//!

//! Positions store six piece-type bitboards and two side bitboards. The
//! [`Piece::Empty`] and [`Side::Empty`]
//! variants represent absence; they are sentinels, not valid indices into those
//! arrays. [`Square::None`] represents the absence of a square and is not a valid
//! bit index in an internal bitboard.

mod bitboard;
mod castling_rights;
mod game;
mod piece;
mod position;
mod side;
mod square;

pub use game::{DrawReason, Game, GameStatus};
pub use piece::Piece;
pub use position::{
    Position,
    movegen::{Move, PositionStatus},
};
pub use side::Side;
pub use square::{InvalidSquareIndex, Square};
