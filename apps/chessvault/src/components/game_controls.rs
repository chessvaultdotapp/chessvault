use chess_core::{Game, GameStatus, Move, Side};
use iced::{
    Fill,
    widget::{Column, button, column, container, row, scrollable, text},
};

use crate::game_status;

#[derive(Debug, Clone)]
pub(crate) enum Message {
    NewGame,
    SavePgn,
    Resign(Side),
    ClaimDraw,
    ClaimDrawWithMove(Move),
    OfferDraw(Side),
    AcceptDraw(Side),
    DeclineDraw(Side),
}

pub(crate) struct Props<'a> {
    pub(crate) game: &'a Game,
    pub(crate) status: GameStatus,
    pub(crate) reviewing: bool,
    pub(crate) navigation_enabled: bool,
    pub(crate) draw_actions_enabled: bool,
    pub(crate) saving_pgn: bool,
    pub(crate) pgn_error: Option<&'a str>,
}

pub(crate) fn view(props: Props<'_>) -> Column<'_, Message> {
    let Props {
        game,
        status: game_status,
        reviewing,
        navigation_enabled,
        draw_actions_enabled,
        saving_pgn,
        pgn_error,
    } = props;
    let ongoing = !reviewing && game_status == GameStatus::Ongoing;
    let label = game_status::label(game_status);
    let status = container(text(label).size(24)).padding(16).width(Fill);
    let status = if label.is_empty() {
        status
    } else {
        status.style(container::rounded_box)
    };

    let controls = column![
        button("New game")
            .on_press_maybe(
                (!game.moves().is_empty() || game.status() != GameStatus::Ongoing).then_some(Message::NewGame),
            )
            .width(Fill)
    ]
    .spacing(12);
    let controls = if !reviewing && game.can_claim_draw() {
        controls.push(
            button("Claim draw")
                .on_press_maybe(navigation_enabled.then_some(Message::ClaimDraw))
                .width(Fill),
        )
    } else {
        controls
    };
    let mut controls = controls;
    if ongoing {
        let enabled = draw_actions_enabled;
        if let Some(offering) = game.draw_offer() {
            let (label, responding) = if offering == Side::White {
                ("White offers a draw", Side::Black)
            } else {
                ("Black offers a draw", Side::White)
            };
            let player = if responding == Side::White { "White" } else { "Black" };
            controls = controls.push(
                column![
                    text(label),
                    row![
                        button(text(format!("{player} accepts")))
                            .on_press_maybe(enabled.then_some(Message::AcceptDraw(responding))),
                        button("Decline").on_press_maybe(enabled.then_some(Message::DeclineDraw(responding))),
                    ]
                    .spacing(8),
                ]
                .spacing(8),
            );
        } else {
            let side = game.position().side_to_move();
            let label = if side == Side::White {
                "White offers draw"
            } else {
                "Black offers draw"
            };
            controls = controls.push(
                button(label)
                    .on_press_maybe(enabled.then_some(Message::OfferDraw(side)))
                    .width(Fill),
            );
        }
    }
    if ongoing && !game.can_claim_draw() {
        let mut intended_claims = column![].spacing(8);
        let mut has_intended_claim = false;
        for mv in game.position().legal_moves() {
            if game.can_claim_draw_with_move(mv) {
                has_intended_claim = true;
                intended_claims = intended_claims.push(
                    button(text(format!("Claim draw with {}–{}", mv.from, mv.to)))
                        .on_press_maybe(navigation_enabled.then_some(Message::ClaimDrawWithMove(mv)))
                        .width(Fill),
                );
            }
        }
        if has_intended_claim {
            controls = controls.push(container(scrollable(intended_claims)).max_height(140));
        }
    }
    if game.status() != GameStatus::Ongoing && !reviewing {
        controls = controls.push(
            button("Save PGN")
                .on_press_maybe((!saving_pgn).then_some(Message::SavePgn))
                .width(Fill),
        );
        if let Some(error) = pgn_error {
            controls = controls.push(text(error));
        }
    }
    controls.extend([
        button("Black resigns")
            .on_press_maybe(ongoing.then_some(Message::Resign(Side::Black)))
            .width(Fill)
            .into(),
        status.into(),
        button("White resigns")
            .on_press_maybe(ongoing.then_some(Message::Resign(Side::White)))
            .width(Fill)
            .into(),
    ])
}
