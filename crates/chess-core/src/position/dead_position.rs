use super::*;

impl Position {
    /// Proves deadness for a conservative subset of material configurations.
    ///
    /// This is not a general reachability solver: blocked positions with pawns,
    /// for example, may be dead without satisfying this test. Two knights are
    /// deliberately not accepted: inability to force mate is not deadness.
    pub(super) fn has_dead_material(&self) -> bool {
        let mut kings = [0; 2];
        let mut minors = 0;
        let mut knights = 0;
        let mut bishop_colour = None;
        let mut mixed_bishops = false;
        for index in 0..64 {
            let square = Square::try_from(index as u8).expect("board square");
            let Some((piece, side)) = self.piece_at(square) else {
                continue;
            };
            match piece {
                Piece::King => kings[side as usize] += 1,
                Piece::Knight => {
                    minors += 1;
                    knights += 1;
                }
                Piece::Bishop => {
                    minors += 1;
                    let colour = (index / 8 + index % 8) % 2;
                    mixed_bishops |= bishop_colour.is_some_and(|previous| previous != colour);
                    bishop_colour = Some(colour);
                }
                _ => return false,
            }
        }
        kings == [1, 1] && (minors <= 1 || (knights == 0 && !mixed_bishops))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DrawReason, Game, GameStatus, Move, PositionStatus};

    fn setup(extra: &[(Square, Piece, Side)], turn: Side) -> Position {
        let mut position = Position::empty();
        position.side_to_move = turn;
        for &(square, piece, side) in [
            (Square::A1, Piece::King, Side::White),
            (Square::H8, Piece::King, Side::Black),
        ]
        .iter()
        .chain(extra)
        {
            position.pieces[piece as usize].set(square).unwrap();
            position.sides[side as usize].set(square).unwrap();
        }
        position
    }

    #[test]
    fn bare_kings_and_single_minor_are_dead_for_either_player() {
        use {Piece::*, Side::*, Square::*};
        for turn in [White, Black] {
            for owner in [White, Black] {
                for extra in [vec![], vec![(C1, Bishop, owner)], vec![(B1, Knight, owner)]] {
                    let position = setup(&extra, turn);
                    assert!(position.has_dead_material());
                    assert_eq!(position.status(), PositionStatus::DeadPosition);
                    let game = Game::new(position);
                    assert_eq!(game.status(), GameStatus::Draw(DrawReason::DeadPosition));
                }
            }
        }
    }

    #[test]
    fn bishops_must_all_share_a_square_colour_regardless_of_owner_or_count() {
        use {Piece::*, Side::*, Square::*};
        for turn in [White, Black] {
            for owner in [White, Black] {
                for squares in [[C1, E3, A3], [B1, D3, F3]] {
                    let position = setup(
                        &[
                            (squares[0], Bishop, owner),
                            (squares[1], Bishop, White),
                            (squares[2], Bishop, Black),
                        ],
                        turn,
                    );
                    assert!(position.has_dead_material());
                    assert_eq!(position.status(), PositionStatus::DeadPosition);
                }
            }
        }
    }

    #[test]
    fn material_with_possible_mate_is_not_declared_dead() {
        use {Piece::*, Side::*, Square::*};
        for turn in [White, Black] {
            for owner in [White, Black] {
                for other in [White, Black] {
                    for extra in [
                        vec![(B2, Pawn, owner)],
                        vec![(C1, Rook, owner)],
                        vec![(C1, Queen, owner)],
                        vec![(B1, Knight, owner), (D2, Knight, other)],
                        vec![(C1, Bishop, owner), (B1, Knight, other)],
                        vec![(C1, Bishop, owner), (B1, Bishop, other)],
                    ] {
                        let position = setup(&extra, turn);
                        assert!(!position.has_dead_material(), "{extra:?}");
                        assert_ne!(position.status(), PositionStatus::DeadPosition);
                    }
                }
            }
        }
        assert!(!Position::standard().has_dead_material());
        assert!(!Position::empty().has_dead_material());
    }

    #[test]
    fn promotion_can_create_dead_material_including_after_a_capture() {
        use {Piece::*, Side::*, Square::*};
        for capture in [false, true] {
            for piece in [Bishop, Knight, Rook, Queen] {
                let mut extra = vec![(B7, Pawn, White)];
                if capture {
                    extra.push((C8, Rook, Black));
                }
                let mut game = Game::new(setup(&extra, White));
                assert_eq!(game.status(), GameStatus::Ongoing);
                assert!(game.play(Move {
                    from: B7,
                    to: if capture { C8 } else { B8 },
                    promotion: Some(piece),
                }));
                assert_eq!(
                    game.status(),
                    if matches!(piece, Bishop | Knight) {
                        GameStatus::Draw(DrawReason::DeadPosition)
                    } else {
                        GameStatus::Ongoing
                    }
                );
            }
        }
    }

    #[test]
    fn check_without_mate_does_not_prevent_a_dead_position_draw() {
        let position = setup(&[(Square::B2, Piece::Bishop, Side::Black)], Side::White);
        assert!(position.is_in_check());
        assert!(!position.legal_moves().is_empty());
        assert_eq!(position.status(), PositionStatus::DeadPosition);
    }

    #[test]
    fn capture_ends_game_without_requiring_a_claim() {
        use {Piece::*, Side::*, Square::*};
        let mut game = Game::new(setup(&[(C1, Bishop, White), (D2, Knight, Black)], White));
        assert_eq!(game.status(), GameStatus::Ongoing);
        assert!(game.play(Move {
            from: C1,
            to: D2,
            promotion: Option::None
        }));
        assert_eq!(game.status(), GameStatus::Draw(DrawReason::DeadPosition));
        assert!(!game.position().legal_moves().is_empty());
        let mv = game.position().legal_moves()[0];
        assert!(!game.play(mv));
        assert!(!game.can_claim_draw());
        assert!(!game.claim_draw_with_move(mv));
        assert!(!game.resign(Black));
        assert_eq!(game.moves().len(), 1);
        assert_eq!(game.position_at(0).unwrap().status(), PositionStatus::Ongoing);
        assert_eq!(game.position_at(1).unwrap().status(), PositionStatus::DeadPosition);
    }
}
