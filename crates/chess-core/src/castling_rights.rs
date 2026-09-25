/// Named castling-rights masks, combinable with bitwise OR after casting to `u8`.
#[repr(u8)]
pub(crate) enum CastlingRights {
    NoCastling = 0,
    WhiteKingside = 1,
    WhiteQueenside = 2,
    BlackKingside = 4,
    BlackQueenside = 8,
    All = 1 | 2 | 4 | 8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn castling_rights_uses_one_byte() {
        assert_eq!(size_of::<CastlingRights>(), size_of::<u8>());
    }

    #[rstest]
    #[case::none(CastlingRights::NoCastling, 0)]
    #[case::white_kingside(CastlingRights::WhiteKingside, 1)]
    #[case::white_queenside(CastlingRights::WhiteQueenside, 2)]
    #[case::black_kingside(CastlingRights::BlackKingside, 4)]
    #[case::black_queenside(CastlingRights::BlackQueenside, 8)]
    #[case::all(CastlingRights::All, 15)]
    fn castling_rights_variants_have_expected_values(#[case] variant: CastlingRights, #[case] expected: u8) {
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
}
