use chess_core::{Game, GameStatus, Side};

/// Minimal, headerless PGN for a completed game.
pub fn export(game: &Game) -> Option<String> {
    let result = match game.status() {
        GameStatus::Ongoing => return None,
        GameStatus::Checkmate(Side::White) | GameStatus::Resigned(Side::White) => "0-1",
        GameStatus::Checkmate(Side::Black) | GameStatus::Resigned(Side::Black) => "1-0",
        GameStatus::Draw(_) => "1/2-1/2",
        _ => unreachable!("a finished game has a valid losing side"),
    };
    let mut position = game.position_at(0).expect("game has an initial position");
    let mut tokens = Vec::new();
    for (index, &mv) in game.moves().iter().enumerate() {
        if position.side_to_move() == Side::White {
            tokens.push(format!("{}.", position.fullmove_number()));
        } else if index == 0 {
            tokens.push(format!("{}...", position.fullmove_number()));
        }
        tokens.push(position.san(mv).expect("recorded move is legal"));
        assert!(position.play(mv));
    }
    tokens.push(result.to_owned());
    // Keep movetext lines short without splitting SAN tokens.
    let mut output = String::new();
    let mut column = 0;
    for token in tokens {
        if column > 0 {
            if column + 1 + token.len() > 79 {
                output.push('\n');
                column = 0;
            } else {
                output.push(' ');
                column += 1;
            }
        }
        output.push_str(&token);
        column += token.len();
    }
    output.push('\n');
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess_core::{Move, Position, Square};

    #[test]
    fn ongoing_games_are_not_exported() {
        assert_eq!(export(&Game::new(Position::standard())), None);
    }

    #[test]
    fn exports_checkmate_with_san_and_result() {
        let mut game = Game::new(Position::standard());
        for (from, to) in [
            (Square::F2, Square::F3),
            (Square::E7, Square::E5),
            (Square::G2, Square::G4),
            (Square::D8, Square::H4),
        ] {
            assert!(game.play(Move {
                from,
                to,
                promotion: None
            }));
        }
        assert_eq!(export(&game).as_deref(), Some("1. f3 e5 2. g4 Qh4# 0-1\n"));
    }

    #[test]
    fn resignation_without_moves() {
        let mut game = Game::new(Position::standard());
        assert!(game.resign(Side::Black));
        assert_eq!(export(&game).as_deref(), Some("1-0\n"));
    }

    #[test]
    fn agreed_draw_without_moves() {
        let mut game = Game::new(Position::standard());
        assert!(game.offer_draw(Side::White));
        assert!(game.accept_draw(Side::Black));
        assert_eq!(export(&game).as_deref(), Some("1/2-1/2\n"));
    }
}
