use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};

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
    /// Creates a zero-valued bitboard with no squares in its set.
    ///
    /// This is also the value returned by [`Default::default`].
    pub(crate) fn empty() -> Self {
        Self(0)
    }
}

/// Defaults to a zero-valued bitboard via [`Bitboard::empty`].
impl Default for Bitboard {
    fn default() -> Self {
        Self::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn bitboard_initializes_with_value_zero() {
        let bitboard = Bitboard::default();
        assert_eq!(bitboard, Bitboard(0));
        assert_eq!(bitboard, Bitboard::empty());
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
}
