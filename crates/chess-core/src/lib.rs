#[repr(u8)]

enum Side {
    White,
    Black,
    Empty,
}

const NUM_SIDES: usize = 2;

#[repr(u8)]
enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    Empty,
}

const NUM_PIECES: usize = 6;

#[derive(Debug, PartialEq)]
struct Bitboard(u64);

impl Default for Bitboard {
    fn default() -> Self {
        return Self(u64::default());
    }
}

struct Position {
    pieces: [Bitboard; NUM_PIECES],
    sides: [Bitboard; NUM_SIDES],
    side_to_move: Side,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            pieces: Default::default(),
            sides: Default::default(),
            side_to_move: Side::Empty,
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
    fn bitboard_initializes_with_value_zero() {
        let bitboard = Bitboard::default();
        assert_eq!(bitboard, Bitboard(0))
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
    fn position_has_one_bitboard_per_piece_type() {
        let position = Position::default();
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
    fn position_has_one_bitboard_per_side() {
        let position = Position::default();
        let sides = [Side::White, Side::Black];

        assert_eq!(position.sides.len(), sides.len());
        for side in sides {
            assert_eq!(position.sides[side as usize], Bitboard(0));
        }
    }

    #[test]
    fn position_defaults_to_no_side_to_move() {
        let position = Position::default();
        assert!(matches!(position.side_to_move, Side::Empty));
    }
}
