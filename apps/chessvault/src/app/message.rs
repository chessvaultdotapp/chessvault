use chess_core::{Move, Piece, Side, Square};
use iced::{widget::text_editor, window};

use crate::components::{game_controls, move_history::Navigation};

#[derive(Debug, Clone)]
pub(crate) enum Message {
    NewGame,
    SavePgn,
    PgnSaved(Result<(), String>),
    ConfirmNewGame,
    CancelNewGame,
    Navigate(Navigation),
    SelectSquare(Square),
    Promote(Piece),
    CancelPromotion,
    Resign(Side),
    ClaimDraw,
    ClaimDrawWithMove(Move),
    OfferDraw(Side),
    AcceptDraw(Side),
    DeclineDraw(Side),
    ToggleConsole,
    ClearLogs,
    RefreshLogs,
    LogAction(text_editor::Action),
    CopyLogs,
    ToggleSources,
    SetSourceEnabled(String, bool),
    WindowCloseRequested(window::Id),
}

impl From<game_controls::Message> for Message {
    fn from(message: game_controls::Message) -> Self {
        match message {
            game_controls::Message::NewGame => Self::NewGame,
            game_controls::Message::SavePgn => Self::SavePgn,
            game_controls::Message::Resign(side) => Self::Resign(side),
            game_controls::Message::ClaimDraw => Self::ClaimDraw,
            game_controls::Message::ClaimDrawWithMove(mv) => Self::ClaimDrawWithMove(mv),
            game_controls::Message::OfferDraw(side) => Self::OfferDraw(side),
            game_controls::Message::AcceptDraw(side) => Self::AcceptDraw(side),
            game_controls::Message::DeclineDraw(side) => Self::DeclineDraw(side),
        }
    }
}
