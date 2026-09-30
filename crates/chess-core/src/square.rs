use std::fmt::Display;

/// A chessboard square, named by file (A–H) and rank (1–8), or a no-square sentinel.
///
/// Stored as one byte. The 64 board squares have discriminants 0 through 63:
/// Squares are ordered by rank, from A1 through H1 (0–7) to A8 through H8
/// (56–63). Each square's index is `8 * (rank - 1) + file`, with files A–H
/// numbered 0–7.
/// [`Square::None`] has discriminant 64 and is not a valid bit index in a
/// [`crate::bitboard::Bitboard`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
#[rustfmt::skip]
pub enum Square {
    A1,B1,C1,D1,E1,F1,G1,H1,
    A2,B2,C2,D2,E2,F2,G2,H2,
    A3,B3,C3,D3,E3,F3,G3,H3,
    A4,B4,C4,D4,E4,F4,G4,H4,
    A5,B5,C5,D5,E5,F5,G5,H5,
    A6,B6,C6,D6,E6,F6,G6,H6,
    A7,B7,C7,D7,E7,F7,G7,H7,
    A8,B8,C8,D8,E8,F8,G8,H8,

    /// No square; a sentinel outside the 64 board squares.
    None,
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// An index outside the 64 board squares, including the no-square sentinel.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidSquareIndex;

impl TryFrom<u8> for Square {
    type Error = InvalidSquareIndex;

    /// Converts a board index in `0..=63`; sentinel index 64 is rejected.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use crate::square::Square::*;

        Ok(match value {
            0 => A1,
            1 => B1,
            2 => C1,
            3 => D1,
            4 => E1,
            5 => F1,
            6 => G1,
            7 => H1,
            8 => A2,
            9 => B2,
            10 => C2,
            11 => D2,
            12 => E2,
            13 => F2,
            14 => G2,
            15 => H2,
            16 => A3,
            17 => B3,
            18 => C3,
            19 => D3,
            20 => E3,
            21 => F3,
            22 => G3,
            23 => H3,
            24 => A4,
            25 => B4,
            26 => C4,
            27 => D4,
            28 => E4,
            29 => F4,
            30 => G4,
            31 => H4,
            32 => A5,
            33 => B5,
            34 => C5,
            35 => D5,
            36 => E5,
            37 => F5,
            38 => G5,
            39 => H5,
            40 => A6,
            41 => B6,
            42 => C6,
            43 => D6,
            44 => E6,
            45 => F6,
            46 => G6,
            47 => H6,
            48 => A7,
            49 => B7,
            50 => C7,
            51 => D7,
            52 => E7,
            53 => F7,
            54 => G7,
            55 => H7,
            56 => A8,
            57 => B8,
            58 => C8,
            59 => D8,
            60 => E8,
            61 => F8,
            62 => G8,
            63 => H8,
            _ => return Err(InvalidSquareIndex),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn square_uses_one_byte() {
        assert_eq!(size_of::<Square>(), size_of::<u8>());
    }

    #[test]
    fn square_converts_every_valid_index() {
        for index in 0_u8..64 {
            assert_eq!(Square::try_from(index).map(|square| square as u8), Ok(index));
        }
    }

    #[test]
    fn square_rejects_every_invalid_index() {
        for index in 64..=u8::MAX {
            assert_eq!(
                Square::try_from(index).map(|square| square as u8),
                Err(InvalidSquareIndex),
                "index {index} should be rejected"
            );
        }
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
    #[case::b2(Square::B2, 9)]
    #[case::c2(Square::C2, 10)]
    #[case::d2(Square::D2, 11)]
    #[case::e2(Square::E2, 12)]
    #[case::f2(Square::F2, 13)]
    #[case::g2(Square::G2, 14)]
    #[case::h2(Square::H2, 15)]
    #[case::a3(Square::A3, 16)]
    #[case::b3(Square::B3, 17)]
    #[case::c3(Square::C3, 18)]
    #[case::d3(Square::D3, 19)]
    #[case::e3(Square::E3, 20)]
    #[case::f3(Square::F3, 21)]
    #[case::g3(Square::G3, 22)]
    #[case::h3(Square::H3, 23)]
    #[case::a4(Square::A4, 24)]
    #[case::b4(Square::B4, 25)]
    #[case::c4(Square::C4, 26)]
    #[case::d4(Square::D4, 27)]
    #[case::e4(Square::E4, 28)]
    #[case::f4(Square::F4, 29)]
    #[case::g4(Square::G4, 30)]
    #[case::h4(Square::H4, 31)]
    #[case::a5(Square::A5, 32)]
    #[case::b5(Square::B5, 33)]
    #[case::c5(Square::C5, 34)]
    #[case::d5(Square::D5, 35)]
    #[case::e5(Square::E5, 36)]
    #[case::f5(Square::F5, 37)]
    #[case::g5(Square::G5, 38)]
    #[case::h5(Square::H5, 39)]
    #[case::a6(Square::A6, 40)]
    #[case::b6(Square::B6, 41)]
    #[case::c6(Square::C6, 42)]
    #[case::d6(Square::D6, 43)]
    #[case::e6(Square::E6, 44)]
    #[case::f6(Square::F6, 45)]
    #[case::g6(Square::G6, 46)]
    #[case::h6(Square::H6, 47)]
    #[case::a7(Square::A7, 48)]
    #[case::b7(Square::B7, 49)]
    #[case::c7(Square::C7, 50)]
    #[case::d7(Square::D7, 51)]
    #[case::e7(Square::E7, 52)]
    #[case::f7(Square::F7, 53)]
    #[case::g7(Square::G7, 54)]
    #[case::h7(Square::H7, 55)]
    #[case::a8(Square::A8, 56)]
    #[case::b8(Square::B8, 57)]
    #[case::c8(Square::C8, 58)]
    #[case::d8(Square::D8, 59)]
    #[case::e8(Square::E8, 60)]
    #[case::f8(Square::F8, 61)]
    #[case::g8(Square::G8, 62)]
    #[case::h8(Square::H8, 63)]
    #[case::none(Square::None, 64)]
    fn square_variants_have_expected_values(#[case] variant: Square, #[case] expected: u8) {
        assert_eq!(variant as u8, expected);
    }
}
