//! Core chess primitives and bitboard-based position storage.
//!
//! Positions store six piece-type bitboards and two side bitboards. The
//! `Piece::Empty` and `Side::Empty` variants represent absence; they are
//! sentinels, not valid indices into those arrays.

/// A chess side, or a sentinel indicating that no side is present.
///
/// White and Black have discriminants 0 and 1 for indexing [`Position::sides`].
/// [`Side::Empty`] is not a valid index into that array.
#[repr(u8)]
enum Side {
    White,
    Black,
    /// No side, including no side to move in an empty position.
    Empty,
}

/// Number of playing sides (two), excluding the [`Side::Empty`] sentinel.
const NUM_SIDES: usize = 2;

/// A chess piece type, or a sentinel indicating that no piece is present.
///
/// Pawn through King have discriminants 0 through 5 for indexing
/// [`Position::pieces`]. [`Piece::Empty`] is not a valid index into that array.
#[repr(u8)]
enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    /// No piece; excluded from the piece-type bitboard array.
    Empty,
}

/// Number of piece types (six), excluding the [`Piece::Empty`] sentinel.
const NUM_PIECES: usize = 6;

/// Named castling-rights masks, combinable with bitwise OR after casting to `u8`.
#[repr(u8)]
enum CastlingRights {
    NoCastling = 0,
    WhiteKingside = 1,
    WhiteQueenside = 2,
    BlackKingside = 4,
    BlackQueenside = 8,
    All = 1 | 2 | 4 | 8,
}

/// A set of chessboard squares stored as 64 bits in a `u64`.
///
/// Each set bit marks a square in the set. The default is an empty set,
/// equivalent to [`Bitboard::empty`].
#[derive(Debug, PartialEq)]
struct Bitboard(u64);

impl Bitboard {
    /// Creates a zero-valued bitboard with no squares in its set.
    ///
    /// This is also the value returned by [`Default::default`].
    fn empty() -> Self {
        Self(0)
    }
}

/// Defaults to a zero-valued bitboard via [`Bitboard::empty`].
impl Default for Bitboard {
    fn default() -> Self {
        Self::empty()
    }
}

/// Occupancy bitboards, side to move, castling rights, and move counts.
struct Position {
    /// Six piece-type bitboards, indexed by Pawn through King.
    /// The [`Piece::Empty`] sentinel has no entry and is not a valid index.
    pieces: [Bitboard; NUM_PIECES],
    /// Two side bitboards, indexed by White and Black.
    /// The [`Side::Empty`] sentinel has no entry and is not a valid index.
    sides: [Bitboard; NUM_SIDES],
    /// The side whose turn it is, or [`Side::Empty`] when no side is to move.
    side_to_move: Side,
    /// Bitwise OR of [`CastlingRights`] values cast to `u8`.
    castling_rights: u8,
    /// Number of halfmoves since the last pawn move or capture.
    halfmove_count: u8,
    /// Full move count, initialized to zero in an empty position.
    fullmove_count: u32,
}

impl Position {
    /// Creates an empty position with no pieces, side to move, or castling rights.
    ///
    /// All six piece-type bitboards and both side bitboards are initialized
    /// with [`Bitboard::empty`], and the side to move is [`Side::Empty`].
    /// Neither array includes an entry for its `Empty` sentinel.
    /// Both move counts are initialized to zero.
    fn empty() -> Self {
        Self {
            pieces: std::array::from_fn(|_| Bitboard::empty()),
            sides: std::array::from_fn(|_| Bitboard::empty()),
            side_to_move: Side::Empty,
            castling_rights: CastlingRights::NoCastling as u8,
            halfmove_count: 0,
            fullmove_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_uses_one_byte() {
        assert_eq!(size_of::<Side>(), size_of::<u8>());
    }

    #[test]
    fn side_variants_have_expected_values() {
        assert_eq!(Side::White as u8, 0);
        assert_eq!(Side::Black as u8, 1);
        assert_eq!(Side::Empty as u8, 2);
    }

    #[test]
    fn piece_uses_one_byte() {
        assert_eq!(size_of::<Piece>(), size_of::<u8>());
    }

    #[test]
    fn piece_variants_have_expected_values() {
        assert_eq!(Piece::Pawn as u8, 0);
        assert_eq!(Piece::Knight as u8, 1);
        assert_eq!(Piece::Bishop as u8, 2);
        assert_eq!(Piece::Rook as u8, 3);
        assert_eq!(Piece::Queen as u8, 4);
        assert_eq!(Piece::King as u8, 5);
        assert_eq!(Piece::Empty as u8, 6);
    }

    #[test]
    fn castling_rights_uses_one_byte() {
        assert_eq!(size_of::<CastlingRights>(), size_of::<u8>());
    }

    #[test]
    fn castling_rights_variants_have_expected_values() {
        assert_eq!(CastlingRights::NoCastling as u8, 0);
        assert_eq!(CastlingRights::WhiteKingside as u8, 1);
        assert_eq!(CastlingRights::WhiteQueenside as u8, 2);
        assert_eq!(CastlingRights::BlackKingside as u8, 4);
        assert_eq!(CastlingRights::BlackQueenside as u8, 8);
        assert_eq!(CastlingRights::All as u8, 15);
    }

    #[test]
    fn all_castling_rights_combines_the_four_individual_rights() {
        let combined = CastlingRights::WhiteKingside as u8
            | CastlingRights::WhiteQueenside as u8
            | CastlingRights::BlackKingside as u8
            | CastlingRights::BlackQueenside as u8;

        assert_eq!(CastlingRights::All as u8, combined);
    }

    #[test]
    fn bitboard_initializes_with_value_zero() {
        let bitboard = Bitboard::default();
        assert_eq!(bitboard, Bitboard(0));
        assert_eq!(bitboard, Bitboard::empty());
    }

    #[test]
    fn bitboard_preserves_each_square_bit() {
        for square in 0..64 {
            let value = 1_u64 << square;
            assert_eq!(Bitboard(value).0, value);
        }
    }

    #[test]
    fn bitboard_preserves_full_and_alternating_bit_patterns() {
        for value in [0, u64::MAX, 0xAAAA_AAAA_AAAA_AAAA, 0x5555_5555_5555_5555] {
            assert_eq!(Bitboard(value).0, value);
        }
    }

    #[test]
    fn bitboard_equality_compares_all_square_bits() {
        for square in 0..64 {
            let value = 1_u64 << square;
            assert_eq!(Bitboard(value), Bitboard(value));
            assert_ne!(Bitboard(value), Bitboard(0));
            assert_ne!(Bitboard(u64::MAX ^ value), Bitboard(u64::MAX));
        }
    }

    #[test]
    fn empty_position_has_one_zero_bitboard_per_piece_type() {
        let position = Position::empty();
        let pieces = [
            Piece::Pawn,
            Piece::Knight,
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
        ];

        assert_eq!(position.pieces.len(), pieces.len());
        for piece in pieces {
            assert_eq!(position.pieces[piece as usize], Bitboard(0));
        }
    }

    #[test]
    fn empty_position_has_one_zero_bitboard_per_side() {
        let position = Position::empty();
        let sides = [Side::White, Side::Black];

        assert_eq!(position.sides.len(), sides.len());
        for side in sides {
            assert_eq!(position.sides[side as usize], Bitboard(0));
        }
    }

    #[test]
    fn empty_position_has_no_side_to_move() {
        let position = Position::empty();
        assert!(matches!(position.side_to_move, Side::Empty));
    }

    #[test]
    fn empty_position_has_no_castling_rights() {
        let position = Position::empty();
        assert_eq!(position.castling_rights, CastlingRights::NoCastling as u8);
    }

    #[test]
    fn empty_position_has_zero_halfmove_count() {
        let position = Position::empty();
        assert_eq!(position.halfmove_count, 0_u8);
    }

    #[test]
    fn empty_position_has_zero_fullmove_count() {
        let position = Position::empty();
        assert_eq!(position.fullmove_count, 0_u32);
    }
}
