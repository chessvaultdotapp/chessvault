//! Core chess primitives and bitboard-based position storage.
//!
//! Positions store six piece-type bitboards and two side bitboards. The
//! [`Piece::Empty`] and [`Side::Empty`] variants represent absence; they are
//! sentinels, not valid indices into those arrays.
//! [`Square::None`] represents the absence of a square and is not a valid
//! bit index in a [`Bitboard`].

/// A chessboard square, named by file (A–H) and rank (1–8), or a no-square sentinel.
///
/// Stored as one byte. The 64 board squares have discriminants 0 through 63:
/// A1 through H1 come first (0–7), followed by ranks 2 through 8 of each
/// file in turn, from A2 through A8 (8–14) to H2 through H8 (57–63).
/// [`Square::None`] has discriminant 64 and is not a valid bit index in a
/// [`Bitboard`].
#[repr(u8)]
enum Square {
    A1,
    B1,
    C1,
    D1,
    E1,
    F1,
    G1,
    H1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
    A8,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    E2,
    E3,
    E4,
    E5,
    E6,
    E7,
    E8,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    G2,
    G3,
    G4,
    G5,
    G6,
    G7,
    G8,
    H2,
    H3,
    H4,
    H5,
    H6,
    H7,
    H8,
    /// No square; a sentinel outside the 64 board squares.
    None,
}

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
    use rstest::{fixture, rstest};

    #[rstest]
    #[case::square(size_of::<Square>())]
    #[case::side(size_of::<Side>())]
    #[case::piece(size_of::<Piece>())]
    #[case::castling_rights(size_of::<CastlingRights>())]
    fn enums_use_one_byte(#[case] size: usize) {
        assert_eq!(size, size_of::<u8>());
    }

    #[rstest]
    #[case::a1(Square::A1, 0)]
    #[case::b1(Square::B1, 1)]
    #[case::c1(Square::C1, 2)]
    #[case::d1(Square::D1, 3)]
    #[case::e1(Square::E1, 4)]
    #[case::f1(Square::F1, 5)]
    #[case::g1(Square::G1, 6)]
    #[case::h1(Square::H1, 7)]
    #[case::a2(Square::A2, 8)]
    #[case::a3(Square::A3, 9)]
    #[case::a4(Square::A4, 10)]
    #[case::a5(Square::A5, 11)]
    #[case::a6(Square::A6, 12)]
    #[case::a7(Square::A7, 13)]
    #[case::a8(Square::A8, 14)]
    #[case::b2(Square::B2, 15)]
    #[case::b3(Square::B3, 16)]
    #[case::b4(Square::B4, 17)]
    #[case::b5(Square::B5, 18)]
    #[case::b6(Square::B6, 19)]
    #[case::b7(Square::B7, 20)]
    #[case::b8(Square::B8, 21)]
    #[case::c2(Square::C2, 22)]
    #[case::c3(Square::C3, 23)]
    #[case::c4(Square::C4, 24)]
    #[case::c5(Square::C5, 25)]
    #[case::c6(Square::C6, 26)]
    #[case::c7(Square::C7, 27)]
    #[case::c8(Square::C8, 28)]
    #[case::d2(Square::D2, 29)]
    #[case::d3(Square::D3, 30)]
    #[case::d4(Square::D4, 31)]
    #[case::d5(Square::D5, 32)]
    #[case::d6(Square::D6, 33)]
    #[case::d7(Square::D7, 34)]
    #[case::d8(Square::D8, 35)]
    #[case::e2(Square::E2, 36)]
    #[case::e3(Square::E3, 37)]
    #[case::e4(Square::E4, 38)]
    #[case::e5(Square::E5, 39)]
    #[case::e6(Square::E6, 40)]
    #[case::e7(Square::E7, 41)]
    #[case::e8(Square::E8, 42)]
    #[case::f2(Square::F2, 43)]
    #[case::f3(Square::F3, 44)]
    #[case::f4(Square::F4, 45)]
    #[case::f5(Square::F5, 46)]
    #[case::f6(Square::F6, 47)]
    #[case::f7(Square::F7, 48)]
    #[case::f8(Square::F8, 49)]
    #[case::g2(Square::G2, 50)]
    #[case::g3(Square::G3, 51)]
    #[case::g4(Square::G4, 52)]
    #[case::g5(Square::G5, 53)]
    #[case::g6(Square::G6, 54)]
    #[case::g7(Square::G7, 55)]
    #[case::g8(Square::G8, 56)]
    #[case::h2(Square::H2, 57)]
    #[case::h3(Square::H3, 58)]
    #[case::h4(Square::H4, 59)]
    #[case::h5(Square::H5, 60)]
    #[case::h6(Square::H6, 61)]
    #[case::h7(Square::H7, 62)]
    #[case::h8(Square::H8, 63)]
    #[case::none(Square::None, 64)]
    fn square_variants_have_expected_values(#[case] variant: Square, #[case] expected: u8) {
        assert_eq!(variant as u8, expected);
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

    #[rstest]
    #[case::empty(0)]
    #[case::full(u64::MAX)]
    #[case::odd_squares(0xAAAA_AAAA_AAAA_AAAA)]
    #[case::even_squares(0x5555_5555_5555_5555)]
    fn bitboard_preserves_full_and_alternating_bit_patterns(#[case] value: u64) {
        assert_eq!(Bitboard(value).0, value);
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

    #[rstest]
    #[case::pawn(Piece::Pawn)]
    #[case::knight(Piece::Knight)]
    #[case::bishop(Piece::Bishop)]
    #[case::rook(Piece::Rook)]
    #[case::queen(Piece::Queen)]
    #[case::king(Piece::King)]
    fn empty_position_has_one_zero_bitboard_per_piece_type(#[case] piece: Piece) {
        let position = Position::empty();

        assert_eq!(position.pieces.len(), 6);
        assert_eq!(position.pieces[piece as usize], Bitboard(0));
    }

    #[rstest]
    #[case::white(Side::White)]
    #[case::black(Side::Black)]
    fn empty_position_has_one_zero_bitboard_per_side(#[case] side: Side) {
        let position = Position::empty();

        assert_eq!(position.sides.len(), 2);
        assert_eq!(position.sides[side as usize], Bitboard(0));
    }

    #[fixture]
    fn empty_position() -> Position {
        Position::empty()
    }

    #[rstest]
    #[case::no_side_to_move(|position: &Position| assert!(matches!(position.side_to_move, Side::Empty)))]
    #[case::no_castling_rights(|position: &Position| {
        assert_eq!(position.castling_rights, CastlingRights::NoCastling as u8);
    })]
    #[case::zero_halfmove_count(|position: &Position| assert_eq!(position.halfmove_count, 0_u8))]
    #[case::zero_fullmove_count(|position: &Position| assert_eq!(position.fullmove_count, 0_u32))]
    fn empty_position_has_expected_fields(empty_position: Position, #[case] check: fn(&Position)) {
        check(&empty_position);
    }
}
