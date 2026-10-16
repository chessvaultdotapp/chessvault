use super::*;

/// Exact repetition identity: two four-bit square codes per byte, then metadata.
/// Move clocks and en passant targets without a legal capture are excluded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RepetitionKey([u8; 34]);

impl Position {
    pub(crate) fn repetition_key(&self) -> RepetitionKey {
        let mut bytes = [0; 34];
        for index in 0..64 {
            let square = Square::try_from(index as u8).expect("board index");
            if let Some((piece, side)) = self.piece_at(square) {
                let code = 1 + piece as u8 + if side == Side::Black { 6 } else { 0 };
                bytes[index / 2] |= code << (4 * (index % 2));
            }
        }

        // Bits 0..=4: side and castling rights. Bits 5..=8: no EP (0), or file + 1.
        let en_passant = self.en_passant.filter(|&target| {
            self.legal_moves().iter().any(|mv| {
                mv.to == target
                    && self.piece_at(mv.from) == Some((Piece::Pawn, self.side_to_move))
                    && mv.from as usize % 8 != target as usize % 8
                    && self.piece_at(target).is_none()
            })
        });
        let ep_file = en_passant.map_or(0, |target| target as u16 % 8 + 1);
        let state =
            u16::from(self.side_to_move == Side::Black) | (u16::from(self.castling_rights) << 1) | (ep_file << 5);
        bytes[32..].copy_from_slice(&state.to_le_bytes());
        RepetitionKey(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Move;

    #[test]
    fn key_is_exactly_34_bytes() {
        assert_eq!(size_of::<RepetitionKey>(), 34);
    }

    #[test]
    fn clocks_are_ignored_but_turn_rights_and_placement_are_not() {
        let position = Position::standard();
        let key = position.repetition_key();
        let mut changed = position.clone();
        changed.halfmove_count = 42;
        changed.fullmove_count = 99;
        assert_eq!(key, changed.repetition_key());
        changed.side_to_move = Side::Black;
        assert_ne!(key, changed.repetition_key());
        changed = position.clone();
        changed.castling_rights = 0;
        assert_ne!(key, changed.repetition_key());
        changed = position.clone();
        assert!(changed.play(Move {
            from: Square::E2,
            to: Square::E4,
            promotion: None
        }));
        changed.side_to_move = Side::White;
        assert_ne!(key, changed.repetition_key());
    }

    fn ep_position(file: usize, pinned: bool) -> Position {
        use {Piece::*, Side::*, Square::*};
        let mut position = Position::empty();
        let pawn = Square::try_from((32 + file) as u8).unwrap();
        let adjacent = Square::try_from((32 + if file == 7 { 6 } else { file + 1 }) as u8).unwrap();
        for (square, piece, side) in [
            (E1, King, White),
            (A8, King, Black),
            (pawn, Pawn, Black),
            (adjacent, Pawn, White),
        ] {
            position.pieces[piece as usize].set(square).unwrap();
            position.sides[side as usize].set(square).unwrap();
        }
        if pinned {
            position.pieces[Rook as usize].set(E8).unwrap();
            position.sides[Black as usize].set(E8).unwrap();
        }
        position.side_to_move = White;
        position.en_passant = Some(Square::try_from((40 + file) as u8).unwrap());
        position
    }

    #[test]
    fn legal_en_passant_on_every_file_changes_identity() {
        for file in 0..8 {
            let mut position = ep_position(file, false);
            let key = position.repetition_key();
            position.en_passant = None;
            assert_ne!(key, position.repetition_key(), "file {file}");
        }
    }

    #[test]
    fn pinned_or_unavailable_en_passant_is_ignored() {
        let mut pinned = ep_position(3, true);
        let key = pinned.repetition_key();
        pinned.en_passant = None;
        assert_eq!(key, pinned.repetition_key());

        let mut position = Position::standard();
        assert!(position.play(Move {
            from: Square::E2,
            to: Square::E4,
            promotion: None
        }));
        let key = position.repetition_key();
        position.en_passant = None;
        assert_eq!(key, position.repetition_key());
    }
}
