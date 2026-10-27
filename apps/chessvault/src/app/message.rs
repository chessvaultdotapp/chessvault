use chess_core::{Move, Piece, Side, Square};
use iced::window;

use crate::components::{developer_console, game_controls, move_history::Navigation};

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
    Console(developer_console::Message),
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
