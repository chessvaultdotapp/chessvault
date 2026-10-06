use chess_core::{GameStatus, Side};

pub fn label(status: GameStatus) -> &'static str {
    match status {
        GameStatus::Ongoing => "",
        GameStatus::Draw => "Draw",
        GameStatus::Checkmate(Side::White) => "Black wins - Checkmate",
        GameStatus::Checkmate(Side::Black) => "White wins - Checkmate",
        GameStatus::Checkmate(Side::Empty) => "",
        GameStatus::Resigned(Side::White) => "Black wins - White resigned",
        GameStatus::Resigned(Side::Black) => "White wins - Black resigned",
        GameStatus::Resigned(Side::Empty) => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_labels() {
        for (status, expected) in [
            (GameStatus::Ongoing, ""),
            (GameStatus::Draw, "Draw"),
            (GameStatus::Checkmate(Side::White), "Black wins - Checkmate"),
            (GameStatus::Checkmate(Side::Black), "White wins - Checkmate"),
            (GameStatus::Checkmate(Side::Empty), ""),
            (GameStatus::Resigned(Side::White), "Black wins - White resigned"),
            (GameStatus::Resigned(Side::Black), "White wins - Black resigned"),
        ] {
            assert_eq!(label(status), expected);
        }
    }
}
