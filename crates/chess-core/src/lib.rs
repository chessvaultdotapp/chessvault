#[derive(Debug, PartialEq)]
struct Bitboard(u64);

impl Default for Bitboard {
    fn default() -> Self {
        return Self(u64::default());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitboard_initializes_with_value_zero() {
        let bitboard = Bitboard::default();
        assert_eq!(bitboard, Bitboard(0))
    }
}
