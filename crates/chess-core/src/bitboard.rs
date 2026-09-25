use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

use anyhow::{Context, Result};

use crate::square::Square;

/// A set of chessboard squares stored as 64 bits in a `u64`.
///
/// Each set bit marks a square in the set. The default is an empty set,
/// equivalent to [`Bitboard::empty`].
#[derive(Debug, PartialEq)]
pub(crate) struct Bitboard(u64);

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Bitboard {
    /// Whether a square belongs to this set; the no-square sentinel never does.
    pub(crate) fn contains(&self, square: Square) -> bool {
        (square as u8) < 64 && self.0 & (1_u64 << square as u8) != 0
    }

    /// Creates a zero-valued bitboard with no squares in its set.
    ///
    /// This is also the value returned by [`Default::default`].
    pub(crate) fn empty() -> Self {
        Self(0)
    }

    /// Sets a square's bit, preserving all other bits; rejects `Square::None`.
    pub(crate) fn set(&mut self, square: Square) -> Result<()> {
        *self |=
            Bitboard::try_from(square).with_context(|| format!("Failed to convert square {square} to bitboard"))?;
        Ok(())
    }
}

/// Defaults to a zero-valued bitboard via [`Bitboard::empty`].
impl Default for Bitboard {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct InvalidSquare;

impl std::fmt::Display for InvalidSquare {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the no-square sentinel cannot be converted to a bitboard")
    }
}

impl std::error::Error for InvalidSquare {}

impl TryFrom<Square> for Bitboard {
    type Error = InvalidSquare;

    fn try_from(value: Square) -> Result<Self, Self::Error> {
        match value {
            Square::None => Err(InvalidSquare),
            square => Ok(Self(1_u64 << square as u8)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn bitboard_contains_only_the_set_square() {
        for set_index in 0_u8..64 {
            let board = Bitboard(1_u64 << set_index);
            for query_index in 0_u8..64 {
                let square = Square::try_from(query_index).unwrap();
                assert_eq!(
                    board.contains(square),
                    query_index == set_index,
                    "set bit {set_index}, queried {square}"
                );
            }
        }
    }

    #[rstest]
    #[case::empty(0, false)]
    #[case::full(u64::MAX, true)]
    fn bitboard_contains_handles_empty_and_full_boards(#[case] mask: u64, #[case] expected: bool) {
        let board = Bitboard(mask);
        for index in 0_u8..64 {
            let square = Square::try_from(index).unwrap();
            assert_eq!(board.contains(square), expected, "queried {square}");
        }
    }

    #[rstest]
    #[case::empty(0)]
    #[case::full(u64::MAX)]
    #[case::highest_bit(1_u64 << 63)]
    fn bitboard_never_contains_no_square(#[case] mask: u64) {
        assert!(!Bitboard(mask).contains(Square::None));
    }

    #[test]
    fn bitboard_initializes_with_value_zero() {
        let bitboard = Bitboard::default();
        assert_eq!(bitboard, Bitboard(0));
        assert_eq!(bitboard, Bitboard::empty());
    }

    #[rstest]
    fn bitboard_converts_every_square_to_its_bit(
        #[values(
            #[rustfmt::skip]
            Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1,
            Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
            Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
            Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
            Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
            Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
            Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
            Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
        )]
        square: Square,
    ) {
        let bit = square as u8;
        assert_eq!(Bitboard::try_from(square), Ok(Bitboard(1_u64 << bit)));
    }

    #[test]
    fn bitboard_rejects_no_square() {
        assert_eq!(Bitboard::try_from(Square::None), Err(InvalidSquare));
    }

    #[test]
    fn bitboard_set_preserves_existing_bits_and_is_idempotent() {
        let mut board = Bitboard::empty();
        board.set(Square::A1).unwrap();
        board.set(Square::H8).unwrap();
        board.set(Square::A1).unwrap();
        assert_eq!(board, Bitboard(1 | (1_u64 << 63)));
    }

    #[test]
    fn bitboard_set_rejects_no_square_without_changing_bits() {
        let mut board = Bitboard(0b1010);
        let error = board.set(Square::None).unwrap_err();
        assert_eq!(board, Bitboard(0b1010));
        assert_eq!(error.downcast_ref::<InvalidSquare>(), Some(&InvalidSquare));
        assert_eq!(error.to_string(), "Failed to convert square None to bitboard");
    }

    #[test]
    fn bitboard_bitand_keeps_shared_squares() {
        assert_eq!(Bitboard(0b1100) & Bitboard(0b1010), Bitboard(0b1000));

        let mut board = Bitboard(0b1100);
        board &= Bitboard(0b1010);
        assert_eq!(board, Bitboard(0b1000));
    }

    #[test]
    fn bitboard_bitor_combines_squares() {
        assert_eq!(Bitboard(0b1100) | Bitboard(0b1010), Bitboard(0b1110));

        let mut board = Bitboard(0b1100);
        board |= Bitboard(0b1010);
        assert_eq!(board, Bitboard(0b1110));
    }

    #[test]
    fn bitboard_bitxor_keeps_unshared_squares() {
        assert_eq!(Bitboard(0b1100) ^ Bitboard(0b1010), Bitboard(0b0110));

        let mut board = Bitboard(0b1100);
        board ^= Bitboard(0b1010);
        assert_eq!(board, Bitboard(0b0110));
    }

    #[rstest]
    fn bitboard_preserves_each_square_bit(
        #[values(
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
            29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
            56, 57, 58, 59, 60, 61, 62, 63
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
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
            29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55,
            56, 57, 58, 59, 60, 61, 62, 63
        )]
        square: u32,
    ) {
        let value = 1_u64 << square;
        assert_eq!(Bitboard(value), Bitboard(value));
        assert_ne!(Bitboard(value), Bitboard(0));
        assert_ne!(Bitboard(u64::MAX ^ value), Bitboard(u64::MAX));
    }
}
