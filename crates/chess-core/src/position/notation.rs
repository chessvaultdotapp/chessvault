use crate::{Move, Piece, Position, PositionStatus};

fn letter(piece: Piece) -> &'static str {
    match piece {
        Piece::Knight => "N",
        Piece::Bishop => "B",
        Piece::Rook => "R",
        Piece::Queen => "Q",
        Piece::King => "K",
        Piece::Pawn | Piece::Empty => "",
    }
}

impl Position {
    /// Formats a legal move in standard algebraic notation, without changing the position.
    /// Illegal moves return `None`.
    pub fn san(&self, mv: Move) -> Option<String> {
        let legal = self.legal_moves();
        if !legal.contains(&mv) {
            return None;
        }
        let (piece, side) = self.piece_at(mv.from)?;
        let file = |square| char::from(b'a' + square as u8 % 8);
        let rank = |square| char::from(b'1' + square as u8 / 8);
        let mut san = String::new();
        if piece == Piece::King && (mv.from as u8).abs_diff(mv.to as u8) == 2 {
            san.push_str(if mv.to as u8 > mv.from as u8 { "O-O" } else { "O-O-O" });
        } else {
            let capture = self.piece_at(mv.to).is_some() || (piece == Piece::Pawn && file(mv.from) != file(mv.to));
            san.push_str(letter(piece));
            if piece == Piece::Pawn {
                if capture {
                    san.push(file(mv.from));
                }
            } else {
                let alternatives: Vec<_> = legal
                    .iter()
                    .filter(|other| {
                        other.from != mv.from && other.to == mv.to && self.piece_at(other.from) == Some((piece, side))
                    })
                    .collect();
                if !alternatives.is_empty() {
                    let same_file = alternatives.iter().any(|other| file(other.from) == file(mv.from));
                    let same_rank = alternatives.iter().any(|other| rank(other.from) == rank(mv.from));
                    if !same_file || same_rank {
                        san.push(file(mv.from));
                    }
                    if same_file {
                        san.push(rank(mv.from));
                    }
                }
            }
            if capture {
                san.push('x');
            }
            san.push(file(mv.to));
            san.push(rank(mv.to));
            if let Some(promotion) = mv.promotion {
                san.push('=');
                san.push_str(letter(promotion));
            }
        }
        let mut after = self.clone();
        assert!(after.play(mv));
        if after.is_in_check() {
            san.push(if after.status() == PositionStatus::Checkmate {
                '#'
            } else {
                '+'
            });
        }
        Some(san)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Side, Square, castling_rights::CastlingRights};

    fn mv(from: Square, to: Square) -> Move {
        Move {
            from,
            to,
            promotion: None,
        }
    }

    fn setup(pieces: &[(Square, Piece, Side)]) -> Position {
        let mut position = Position::empty();
        position.side_to_move = Side::White;
        for &(square, piece, side) in pieces {
            position.pieces[piece as usize].set(square).unwrap();
            position.sides[side as usize].set(square).unwrap();
        }
        position
    }

    #[test]
    fn normal_moves_and_mate() {
        use Square::*;
        let mut position = Position::standard();
        assert_eq!(position.san(mv(E2, E5)), Option::None);
        for (from, to, san) in [(F2, F3, "f3"), (E7, E5, "e5"), (G2, G4, "g4"), (D8, H4, "Qh4#")] {
            let before = position.clone();
            assert_eq!(position.san(mv(from, to)).as_deref(), Some(san));
            assert_eq!(position, before);
            assert!(position.play(mv(from, to)));
        }
    }

    #[test]
    fn castling_en_passant_and_promotions() {
        use Piece::*;
        use Side::*;
        use Square::*;
        let mut position = setup(&[
            (E1, King, White),
            (A1, Rook, White),
            (H1, Rook, White),
            (E8, King, Black),
        ]);
        position.castling_rights = CastlingRights::All as u8;
        assert_eq!(position.san(mv(E1, G1)).as_deref(), Some("O-O"));
        assert_eq!(position.san(mv(E1, C1)).as_deref(), Some("O-O-O"));
        let mut position = Position::standard();
        for (from, to) in [(E2, E4), (A7, A6), (E4, E5), (D7, D5)] {
            assert!(position.play(mv(from, to)));
        }
        assert_eq!(position.san(mv(E5, D6)).as_deref(), Some("exd6"));
        let position = setup(&[
            (E1, King, White),
            (H7, King, Black),
            (B7, Pawn, White),
            (A8, Rook, Black),
        ]);
        for (piece, letter) in [(Queen, "Q"), (Rook, "R"), (Bishop, "B"), (Knight, "N")] {
            for (to, prefix) in [(B8, "b8="), (A8, "bxa8=")] {
                assert_eq!(
                    position.san(Move {
                        from: B7,
                        to,
                        promotion: Some(piece)
                    }),
                    Some(format!("{prefix}{letter}"))
                );
            }
        }
    }

    #[test]
    fn captures_and_checks_including_castling() {
        use Piece::*;
        use Side::*;
        use Square::*;
        let position = setup(&[(E1, King, White), (H8, King, Black), (D1, Queen, White)]);
        assert_eq!(position.san(mv(D1, H5)).as_deref(), Some("Qh5+"));
        let position = setup(&[
            (E1, King, White),
            (H8, King, Black),
            (B1, Knight, White),
            (D2, Pawn, Black),
        ]);
        assert_eq!(position.san(mv(B1, D2)).as_deref(), Some("Nxd2"));
        let mut position = setup(&[(E1, King, White), (F8, King, Black), (H1, Rook, White)]);
        position.castling_rights = CastlingRights::All as u8;
        assert_eq!(position.san(mv(E1, G1)).as_deref(), Some("O-O+"));
    }

    #[test]
    fn disambiguates_by_file_rank_or_both_and_ignores_pinned_pieces() {
        use Piece::*;
        use Side::*;
        use Square::*;
        for (pieces, from, to, san) in [
            (vec![(B1, Knight, White), (F1, Knight, White)], B1, D2, "Nbd2"),
            (vec![(B1, Knight, White), (B3, Knight, White)], B1, D2, "N1d2"),
            (
                vec![(B1, Knight, White), (B3, Knight, White), (F1, Knight, White)],
                B1,
                D2,
                "Nb1d2",
            ),
            (
                vec![(E2, Knight, White), (G2, Knight, White), (E8, Rook, Black)],
                G2,
                F4,
                "Nf4",
            ),
        ] {
            let mut pieces = pieces;
            pieces.extend([(E1, King, White), (H8, King, Black)]);
            assert_eq!(setup(&pieces).san(mv(from, to)).as_deref(), Some(san));
        }
    }
}
