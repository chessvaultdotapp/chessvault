use chess_core::{DrawReason, GameStatus, Side};

pub fn label(status: GameStatus) -> &'static str {
    match status {
        GameStatus::Ongoing => "",
        GameStatus::Draw(DrawReason::SeventyFiveMoveRule) => "Draw - Seventy-five-move rule",
        GameStatus::Draw(DrawReason::FiftyMoveRule(Side::White)) => "White claimed a draw - Fifty-move rule",
        GameStatus::Draw(DrawReason::FiftyMoveRule(Side::Black)) => "Black claimed a draw - Fifty-move rule",
        GameStatus::Draw(DrawReason::FiftyMoveRule(Side::Empty)) => "",
        GameStatus::Draw(DrawReason::Stalemate) => "Draw - Stalemate",
        GameStatus::Draw(DrawReason::FivefoldRepetition) => "Draw - Fivefold repetition",
        GameStatus::Draw(DrawReason::ThreefoldRepetition(Side::White)) => "White claimed a draw - Threefold repetition",
        GameStatus::Draw(DrawReason::ThreefoldRepetition(Side::Black)) => "Black claimed a draw - Threefold repetition",
        GameStatus::Draw(DrawReason::ThreefoldRepetition(Side::Empty)) => "",
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
            (GameStatus::Draw(DrawReason::Stalemate), "Draw - Stalemate"),
            (
                GameStatus::Draw(DrawReason::SeventyFiveMoveRule),
                "Draw - Seventy-five-move rule",
            ),
            (
                GameStatus::Draw(DrawReason::FiftyMoveRule(Side::White)),
                "White claimed a draw - Fifty-move rule",
            ),
            (
                GameStatus::Draw(DrawReason::FiftyMoveRule(Side::Black)),
                "Black claimed a draw - Fifty-move rule",
            ),
            (GameStatus::Draw(DrawReason::FiftyMoveRule(Side::Empty)), ""),
            (
                GameStatus::Draw(DrawReason::FivefoldRepetition),
                "Draw - Fivefold repetition",
            ),
            (
                GameStatus::Draw(DrawReason::ThreefoldRepetition(Side::White)),
                "White claimed a draw - Threefold repetition",
            ),
            (
                GameStatus::Draw(DrawReason::ThreefoldRepetition(Side::Black)),
                "Black claimed a draw - Threefold repetition",
            ),
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
