/// A chess side, or a sentinel indicating that no side is present.
///
/// White and Black have discriminants 0 and 1 for indexing the side bitboard
/// array in [`crate::position::Position`].
/// [`Side::Empty`] is not a valid index into that array.
#[repr(u8)]
pub(crate) enum Side {
    White,
    Black,
    /// No side, including no side to move in an empty position.
    Empty,
}

/// Number of playing sides (two), excluding the [`Side::Empty`] sentinel.
pub(crate) const NUM_SIDES: usize = 2;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn side_uses_one_byte() {
        assert_eq!(size_of::<Side>(), size_of::<u8>());
    }

    #[rstest]
    #[case::white(Side::White, 0)]
    #[case::black(Side::Black, 1)]
    #[case::empty(Side::Empty, 2)]
    fn side_variants_have_expected_values(#[case] variant: Side, #[case] expected: u8) {
        assert_eq!(variant as u8, expected);
    }
}
