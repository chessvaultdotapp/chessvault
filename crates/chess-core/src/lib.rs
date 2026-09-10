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
    use rstest::rstest;

    #[rstest]
    #[case::side(size_of::<Side>())]
    #[case::piece(size_of::<Piece>())]
    #[case::castling_rights(size_of::<CastlingRights>())]
    fn enums_use_one_byte(#[case] size: usize) {
        assert_eq!(size, size_of::<u8>());
    }

    #[rstest]
    #[case::white(Side::White, 0)]
    #[case::black(Side::Black, 1)]
    #[case::empty(Side::Empty, 2)]
    fn side_variants_have_expected_values(#[case] variant: Side, #[case] expected: u8) {
        assert_eq!(variant as u8, expected);
    }

    #[rstest]
    #[case::pawn(Piece::Pawn, 0)]
    #[case::knight(Piece::Knight, 1)]
    #[case::bishop(Piece::Bishop, 2)]
    #[case::rook(Piece::Rook, 3)]
    #[case::queen(Piece::Queen, 4)]
    #[case::king(Piece::King, 5)]
    #[case::empty(Piece::Empty, 6)]
    fn piece_variants_have_expected_values(#[case] variant: Piece, #[case] expected: u8) {
        assert_eq!(variant as u8, expected);
    }

    #[rstest]
    #[case::none(CastlingRights::NoCastling, 0)]
    #[case::white_kingside(CastlingRights::WhiteKingside, 1)]
    #[case::white_queenside(CastlingRights::WhiteQueenside, 2)]
    #[case::black_kingside(CastlingRights::BlackKingside, 4)]
    #[case::black_queenside(CastlingRights::BlackQueenside, 8)]
    #[case::all(CastlingRights::All, 15)]
    fn castling_rights_variants_have_expected_values(
        #[case] variant: CastlingRights,
        #[case] expected: u8,
    ) {
        assert_eq!(variant as u8, expected);
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

    #[rstest]
    fn bitboard_preserves_each_square_bit(
        #[values(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63
        )]
        square: u32,
    ) {
        let value = 1_u64 << square;
        assert_eq!(Bitboard(value).0, value);
    }

    #[test]
    fn bitboard_preserves_full_and_alternating_bit_patterns() {
        for value in [0, u64::MAX, 0xAAAA_AAAA_AAAA_AAAA, 0x5555_5555_5555_5555] {
            assert_eq!(Bitboard(value).0, value);
        }
    }

    #[rstest]
    fn bitboard_equality_compares_all_square_bits(
        #[values(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63
        )]
        square: u32,
    ) {
        let value = 1_u64 << square;
        assert_eq!(Bitboard(value), Bitboard(value));
        assert_ne!(Bitboard(value), Bitboard(0));
        assert_ne!(Bitboard(u64::MAX ^ value), Bitboard(u64::MAX));
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
