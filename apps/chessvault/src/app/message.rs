use chess_core::{Move, Piece, Side, Square};
use iced::{widget::text_editor, window};

use crate::history::Navigation;

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
