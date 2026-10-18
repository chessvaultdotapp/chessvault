use std::{io::Write, time::Duration};

use anyhow::{Context, Result};
use iced::{
    Element, Fill, Font, Size, Subscription, Task,
    keyboard::{self, Key, key::Named},
    widget::{Space, button, checkbox, column, container, row, scrollable, text, text_editor},
    window,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use chess_core::{Game, GameStatus, Move, Piece, Position, Side, Square};

mod board;
mod fs;
mod game_status;
mod history;
mod icon;
mod pgn;

use history::{History, Navigation};
mod logs;

fn main() -> Result<()> {
    let logs = logs::Logs::init()?;
    info!("ChessVault started");

    let mut window_settings = window::Settings {
        icon: match icon::load() {
            Ok(icon) => Some(icon),
            Err(err) => {
                error!(error = ?err, "Failed to load window icon");
                None
            }
        },
        ..window::Settings::default()
    };
    #[cfg(target_os = "linux")]
    {
        window_settings.platform_specific.application_id = "chessvault".into();
    }
    match WindowRecreateInfo::load() {
        Ok(Some(info)) => window_settings.size = Size::new(info.width, info.height),
        Ok(None) => {
            info!("No saved window size found; using default size");
        }
        Err(err) => error!(error = ?err, "Failed to restore window size; using default size"),
    }

    iced::application(
        move || ChessVault::boot(logs.clone()),
        ChessVault::update,
        ChessVault::view,
    )
    .title("ChessVault")
    .window(window_settings)
    .exit_on_close_request(false)
    .subscription(ChessVault::subscription)
    .run()
    .context("Failed to run ChessVault")
    .inspect_err(|err| error!(error = ?err, "Application exited with an error"))
}

struct ChessVault {
    logs: logs::Logs,
    console_open: bool,
    log_text: String,
    log_content: text_editor::Content,
    sources_open: bool,
    game: Game,
    history: History,
    selected_square: Option<Square>,
    pending_promotion: Option<Move>,
    confirm_new_game: bool,
    saving_pgn: bool,
    pgn_error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct WindowRecreateInfo {
    width: f32,
    height: f32,
}

impl WindowRecreateInfo {
    fn load() -> Result<Option<Self>> {
        let path = fs::window_recreate_info_filepath().context("Failed to resolve window recreate info filepath")?;

        let json = match std::fs::read_to_string(&path) {
            Ok(json) => json,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(err).with_context(|| format!("Failed to read {}", path.display()));
            }
        };

        Self::from_json(&json)
            .with_context(|| format!("Failed to load window recreate info from {}", path.display()))
            .map(Some)
    }

    fn from_json(json: &str) -> Result<Self> {
        let info: Self = serde_json::from_str(json).context("Failed to deserialize window recreate info")?;
        anyhow::ensure!(
            info.width.is_finite() && info.height.is_finite() && info.width > 0.0 && info.height > 0.0,
            "Saved window dimensions must be finite and positive"
        );
        Ok(info)
    }

    fn save(&self) -> Result<()> {
        let path = fs::window_recreate_info_filepath().context("Failed to resolve window recreate info filepath")?;

        let json = serde_json::to_string(self).context("Failed to serialize window recreate info")?;

        let mut file =
            std::fs::File::create(&path).with_context(|| format!("Failed to create or truncate {}", path.display()))?;

        file.write_all(json.as_bytes())
            .with_context(|| format!("Failed to write window recreate info to {}", path.display()))?;

        Ok(())
    }
}

impl From<Size<f32>> for WindowRecreateInfo {
    fn from(value: Size<f32>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

#[cfg(test)]
mod window_recreate_tests {
    use super::*;

    #[test]
    fn saved_dimensions_round_trip() {
        let json = serde_json::to_string(&WindowRecreateInfo::from(Size::new(820.5, 620.0))).unwrap();
        let info = WindowRecreateInfo::from_json(&json).unwrap();
        assert_eq!(info.width, 820.5);
        assert_eq!(info.height, 620.0);
    }

    #[test]
    fn invalid_json_retains_deserialization_error() {
        let err = WindowRecreateInfo::from_json("{invalid}").unwrap_err();
        assert!(err.downcast_ref::<serde_json::Error>().is_some());
    }

    #[test]
    fn invalid_dimensions_are_rejected() {
        for json in [
            r#"{"width":0,"height":620}"#,
            r#"{"width":820,"height":-1}"#,
            r#"{"width":1e100,"height":620}"#,
            r#"{"width":820,"height":1e100}"#,
            r#"{"width":null,"height":620}"#,
            r#"{"width":820}"#,
        ] {
            assert!(WindowRecreateInfo::from_json(json).is_err(), "{json}");
        }
    }
}

#[cfg(test)]
mod draw_tests {
    use super::*;
    use chess_core::DrawReason;

    fn ready() -> ChessVault {
        let mut app = ChessVault {
            logs: logs::Logs::default(),
            console_open: false,
            log_text: String::new(),
            log_content: text_editor::Content::new(),
            sources_open: false,
            game: Game::new(Position::standard()),
            history: History::default(),
            selected_square: Some(Square::E2),
            pending_promotion: None,
            confirm_new_game: false,
            saving_pgn: false,
            pgn_error: None,
        };
        for _ in 0..2 {
            for (from, to) in [
                (Square::G1, Square::F3),
                (Square::G8, Square::F6),
                (Square::F3, Square::G1),
                (Square::F6, Square::G8),
            ] {
                assert!(app.game.play(Move {
                    from,
                    to,
                    promotion: None
                }));
            }
        }
        app.history.sync(&app.game);
        app
    }

    #[test]
    fn claim_ends_play_and_clears_selection() {
        let mut app = ready();
        let _ = app.update(Message::ClaimDraw);
        assert_eq!(
            app.game.status(),
            GameStatus::Draw(DrawReason::ThreefoldRepetition(Side::White))
        );
        assert_eq!(app.selected_square, None);
        let _ = app.update(Message::SelectSquare(Square::E2));
        let _ = app.update(Message::SelectSquare(Square::E4));
        assert_eq!(app.game.moves().len(), 8);
        let _ = app.update(Message::Navigate(history::Navigation::First));
        assert_eq!(app.history.status(&app.game), GameStatus::Ongoing);
    }

    #[test]
    fn offers_can_be_accepted_declined_and_reset() {
        let mut app = ready();
        app.reset_game();
        let _ = app.update(Message::OfferDraw(Side::White));
        assert_eq!(app.game.draw_offer(), Some(Side::White));
        let _ = app.update(Message::DeclineDraw(Side::Black));
        assert_eq!(app.game.draw_offer(), None);
        let _ = app.update(Message::OfferDraw(Side::White));
        let _ = app.update(Message::SelectSquare(Square::E2));
        let _ = app.update(Message::SelectSquare(Square::E4));
        assert_eq!(app.game.draw_offer(), Some(Side::White));
        let position = app.game.position().clone();
        let _ = app.update(Message::AcceptDraw(Side::Black));
        assert_eq!(app.game.status(), GameStatus::Draw(DrawReason::Agreement));
        assert_eq!(app.game.position(), &position);
        assert_eq!(app.game.moves().len(), 1);
        assert_eq!(app.selected_square, None);
        let _ = app.update(Message::Navigate(Navigation::First));
        assert_eq!(app.history.status(&app.game), GameStatus::Ongoing);
        let _ = app.update(Message::NewGame);
        assert_eq!(app.game.status(), GameStatus::Ongoing);
        assert_eq!(app.game.draw_offer(), None);
        assert!(app.game.moves().is_empty());

        let _ = app.update(Message::OfferDraw(Side::White));
        let _ = app.update(Message::AcceptDraw(Side::Black));
        assert_eq!(app.game.status(), GameStatus::Draw(DrawReason::Agreement));
        let _ = app.update(Message::NewGame);
        assert_eq!(app.game.status(), GameStatus::Ongoing);
    }

    #[test]
    fn offer_actions_are_blocked_during_review_and_modals() {
        for guard in 0..4 {
            let mut app = ready();
            assert!(app.game.offer_draw(Side::White));
            match guard {
                0 => {
                    let _ = app.update(Message::Navigate(Navigation::First));
                }
                1 => app.console_open = true,
                2 => {
                    app.pending_promotion = Some(Move {
                        from: Square::A7,
                        to: Square::A8,
                        promotion: Some(Piece::Queen),
                    })
                }
                _ => app.confirm_new_game = true,
            }
            let _ = app.update(Message::AcceptDraw(Side::Black));
            let _ = app.update(Message::DeclineDraw(Side::Black));
            assert_eq!(app.game.draw_offer(), Some(Side::White));
            assert_eq!(app.game.status(), GameStatus::Ongoing);
            assert!(app.game.decline_draw(Side::Black));
            let _ = app.update(Message::OfferDraw(Side::White));
            assert_eq!(app.game.draw_offer(), None);
        }
    }

    fn intended_claim_ready() -> (ChessVault, Move) {
        let mut app = ready();
        app.game = Game::new(Position::standard());
        for (from, to) in [
            (Square::G1, Square::F3),
            (Square::G8, Square::F6),
            (Square::F3, Square::G1),
            (Square::F6, Square::G8),
            (Square::G1, Square::F3),
            (Square::G8, Square::F6),
            (Square::F3, Square::G1),
        ] {
            assert!(app.game.play(Move {
                from,
                to,
                promotion: None
            }));
        }
        app.history = History::default();
        app.history.sync(&app.game);
        (
            app,
            Move {
                from: Square::F6,
                to: Square::G8,
                promotion: None,
            },
        )
    }

    #[test]
    fn intended_claim_ends_game_without_playing_move() {
        let (mut app, mv) = intended_claim_ready();
        assert!(!app.game.can_claim_draw());
        assert!(app.game.can_claim_draw_with_move(mv));
        let position = app.game.position().clone();
        let _ = app.update(Message::ClaimDrawWithMove(mv));
        assert_eq!(
            app.game.status(),
            GameStatus::Draw(DrawReason::ThreefoldRepetition(Side::Black))
        );
        assert_eq!(app.game.position(), &position);
        assert_eq!(app.game.moves().len(), 7);
        assert_eq!(app.selected_square, None);
        let _ = app.update(Message::SelectSquare(Square::G8));
        assert_eq!(app.game.moves().len(), 7);
    }

    #[test]
    fn intended_claim_respects_guards_and_rejects_invalid_moves() {
        for guard in 0..5 {
            let (mut app, mut mv) = intended_claim_ready();
            match guard {
                0 => {
                    let _ = app.update(Message::Navigate(Navigation::First));
                }
                1 => app.console_open = true,
                2 => app.pending_promotion = Some(mv),
                3 => app.confirm_new_game = true,
                _ => mv.to = Square::E4,
            }
            let _ = app.update(Message::ClaimDrawWithMove(mv));
            assert_eq!(app.game.status(), GameStatus::Ongoing);
            assert_eq!(app.game.moves().len(), 7);
        }
    }

    #[test]
    fn claims_are_blocked_during_review_and_modal_interactions() {
        let mut app = ready();
        let _ = app.update(Message::Navigate(history::Navigation::First));
        let _ = app.update(Message::ClaimDraw);
        assert_eq!(app.game.status(), GameStatus::Ongoing);
        let _ = app.update(Message::Navigate(history::Navigation::Latest));
        app.console_open = true;
        let _ = app.update(Message::ClaimDraw);
        assert_eq!(app.game.status(), GameStatus::Ongoing);
        app.console_open = false;
        app.pending_promotion = Some(Move {
            from: Square::A7,
            to: Square::A8,
            promotion: None,
        });
        let _ = app.update(Message::ClaimDraw);
        assert_eq!(app.game.status(), GameStatus::Ongoing);
        app.pending_promotion = None;
        app.confirm_new_game = true;
        let _ = app.update(Message::ClaimDraw);
        assert_eq!(app.game.status(), GameStatus::Ongoing);
    }
}

#[cfg(test)]
mod resignation_tests {
    use super::*;

    #[test]
    fn resigning_ends_play_and_clears_selection() {
        for side in [Side::White, Side::Black] {
            let mut app = ChessVault {
                logs: logs::Logs::default(),
                console_open: false,
                log_text: String::new(),
                log_content: text_editor::Content::new(),
                sources_open: false,
                game: Game::new(Position::standard()),
                history: History::default(),
                selected_square: Some(Square::E2),
                pending_promotion: None,
                confirm_new_game: false,
                saving_pgn: false,
                pgn_error: None,
            };
            let _ = app.update(Message::Resign(side));
            assert_eq!(app.game.status(), GameStatus::Resigned(side));
            assert_eq!(app.selected_square, None);
            let _ = app.update(Message::SelectSquare(Square::E2));
            let _ = app.update(Message::SelectSquare(Square::E4));
            assert_eq!(app.selected_square, None);
            assert!(app.game.position().piece_at(Square::E2).is_some());
            assert!(app.game.position().piece_at(Square::E4).is_none());
            let _ = app.update(Message::Resign(Side::White));
            let _ = app.update(Message::Resign(Side::Black));
            assert_eq!(app.game.status(), GameStatus::Resigned(side));
        }
    }
}

#[cfg(test)]
mod promotion_tests {
    use super::*;

    fn ready() -> ChessVault {
        use Square::{A2, A4, A5, A6, B7, B8, C6, F3, G1, G2, H3, H4, H5, H7};
        let mut app = ChessVault {
            history: History::default(),
            logs: logs::Logs::default(),
            console_open: false,
            log_text: String::new(),
            log_content: text_editor::Content::new(),
            sources_open: false,
            game: Game::new(Position::standard()),
            selected_square: None,
            pending_promotion: None,
            confirm_new_game: false,
            saving_pgn: false,
            pgn_error: None,
        };
        for (from, to) in [
            (G1, F3),
            (B8, C6),
            (A2, A4),
            (H7, H5),
            (A4, A5),
            (H5, H4),
            (A5, A6),
            (H4, H3),
            (A6, B7),
            (H3, G2),
        ] {
            assert!(app.game.play(Move {
                from,
                to,
                promotion: None
            }));
        }
        app
    }

    #[test]
    fn choices_map_to_legal_moves_for_both_sides_and_capture_types() {
        use Square::{A8, B1, B7, B8, C3, E2, E4, G1, G2, H1};
        for piece in [Piece::Queen, Piece::Rook, Piece::Bishop, Piece::Knight] {
            for capture in [false, true] {
                for side in [Side::White, Side::Black] {
                    let mut app = ready();
                    let (from, to) = if side == Side::White {
                        (B7, if capture { A8 } else { B8 })
                    } else {
                        assert!(app.game.play(Move {
                            from: B1,
                            to: C3,
                            promotion: None
                        }));
                        (G2, if capture { H1 } else { G1 })
                    };
                    let before = app.game.position().legal_moves();
                    let history_len = app.game.moves().len();
                    let _ = app.update(Message::SelectSquare(from));
                    let _ = app.update(Message::SelectSquare(to));
                    assert!(app.pending_promotion.is_some());
                    assert_eq!(app.game.position().legal_moves(), before);
                    assert_eq!(app.game.moves().len(), history_len);
                    let _ = app.update(Message::SelectSquare(E2));
                    let _ = app.update(Message::SelectSquare(E4));
                    assert_eq!(app.selected_square, Some(from));
                    assert_eq!(app.game.position().legal_moves(), before);
                    let _ = app.update(Message::Promote(piece));
                    assert_eq!(app.game.position().piece_at(to), Some((piece, side)));
                    assert_eq!(app.game.position().piece_at(from), None);
                    assert_eq!(app.game.moves().len(), history_len + 1);
                    assert_eq!(
                        app.game.moves().last(),
                        Some(&Move {
                            from,
                            to,
                            promotion: Some(piece)
                        })
                    );
                    assert_eq!(app.selected_square, None);
                    assert_eq!(app.pending_promotion, None);
                }
            }
        }
    }

    #[test]
    fn cancellation_and_stale_choices_do_not_mutate_position() {
        let mut app = ready();
        let before = app.game.position().legal_moves();
        let history = app.game.moves().to_vec();
        let _ = app.update(Message::SelectSquare(Square::B7));
        let _ = app.update(Message::SelectSquare(Square::A8));
        let _ = app.update(Message::Promote(Piece::King));
        assert!(app.pending_promotion.is_some());
        let _ = app.update(Message::CancelPromotion);
        let _ = app.update(Message::Promote(Piece::Queen));
        assert_eq!(app.pending_promotion, None);
        assert_eq!(app.selected_square, Some(Square::B7));
        assert_eq!(app.game.position().legal_moves(), before);
        assert_eq!(app.game.moves(), history);
        let _ = app.update(Message::SelectSquare(Square::A8));
        let _ = app.update(Message::Resign(Side::White));
        let _ = app.update(Message::Promote(Piece::Queen));
        assert_eq!(app.pending_promotion, None);
        assert_eq!(app.game.position().legal_moves(), before);
        assert_eq!(app.game.moves(), history);
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::*;

    fn app() -> ChessVault {
        ChessVault {
            logs: logs::Logs::default(),
            console_open: false,
            log_text: String::new(),
            log_content: text_editor::Content::new(),
            sources_open: false,
            game: Game::new(Position::standard()),
            history: History::default(),
            selected_square: None,
            pending_promotion: None,
            confirm_new_game: false,
            saving_pgn: false,
            pgn_error: None,
        }
    }

    #[test]
    fn new_game_confirmation_cancels_without_changes_and_resets_on_acceptance() {
        let mut app = app();
        let _ = app.update(Message::SelectSquare(Square::E2));
        let _ = app.update(Message::SelectSquare(Square::E4));
        let _ = app.update(Message::Navigate(Navigation::First));
        app.selected_square = Some(Square::B7);
        app.pending_promotion = Some(Move {
            from: Square::B7,
            to: Square::B8,
            promotion: Some(Piece::Queen),
        });
        app.sources_open = true;
        let logs = app.logs.snapshot();
        for cancel in [Message::CancelNewGame, Message::CancelPromotion] {
            let _ = app.update(Message::NewGame);
            assert!(app.confirm_new_game);
            let _ = app.update(Message::Navigate(Navigation::Latest));
            let _ = app.update(Message::Resign(Side::Black));
            let _ = app.update(cancel);
            assert!(!app.confirm_new_game);
            assert_eq!(app.game.moves().len(), 1);
            assert_eq!(app.game.status(), GameStatus::Ongoing);
            assert!(app.history.is_reviewing());
            assert_eq!(app.selected_square, Some(Square::B7));
            assert!(app.pending_promotion.is_some());
        }
        let _ = app.update(Message::NewGame);
        let _ = app.update(Message::ConfirmNewGame);
        assert!(!app.confirm_new_game);
        assert!(app.game.moves().is_empty());
        assert_eq!(app.game.status(), GameStatus::Ongoing);
        assert!(!app.history.is_reviewing());
        assert_eq!(app.selected_square, None);
        assert_eq!(app.pending_promotion, None);
        for square in [Square::E2, Square::E4, Square::E7] {
            assert_eq!(
                app.game.position().piece_at(square),
                Position::standard().piece_at(square)
            );
        }
        assert!(app.sources_open);
        assert_eq!(app.logs.snapshot(), logs);
    }

    #[test]
    fn new_game_resets_finished_and_unplayed_games_without_confirmation() {
        let mut app = app();
        app.selected_square = Some(Square::E2);
        let _ = app.update(Message::NewGame);
        assert!(!app.confirm_new_game);
        assert_eq!(app.selected_square, None);
        let _ = app.update(Message::Resign(Side::White));
        let _ = app.update(Message::NewGame);
        assert!(!app.confirm_new_game);
        assert_eq!(app.game.status(), GameStatus::Ongoing);
    }

    #[test]
    fn review_clears_selection_blocks_play_and_resignation_then_resumes_latest() {
        use Square::{E2, E4, E5, E7};
        let mut app = app();
        let _ = app.update(Message::SelectSquare(E2));
        let _ = app.update(Message::SelectSquare(E4));
        let _ = app.update(Message::SelectSquare(E7));
        assert_eq!(app.selected_square, Some(E7));
        let _ = app.update(Message::Navigate(Navigation::First));
        assert_eq!(app.selected_square, None);
        assert_eq!(app.history.position(&app.game), &Position::standard());
        let live = app.game.position().clone();
        for message in [
            Message::SelectSquare(E2),
            Message::SelectSquare(E4),
            Message::Resign(Side::White),
            Message::Promote(Piece::Queen),
        ] {
            let _ = app.update(message);
        }
        assert_eq!(app.selected_square, None);
        assert_eq!(app.game.position(), &live);
        assert_eq!(app.game.moves().len(), 1);
        assert_eq!(app.game.status(), GameStatus::Ongoing);
        let _ = app.update(Message::Navigate(Navigation::Latest));
        let _ = app.update(Message::SelectSquare(E7));
        let _ = app.update(Message::SelectSquare(E5));
        assert_eq!(app.history.ply(&app.game), 2);
        assert_eq!(app.game.moves().len(), 2);
    }

    #[test]
    fn console_and_promotion_picker_block_navigation() {
        use Square::{E2, E4};
        let mut app = app();
        let _ = app.update(Message::SelectSquare(E2));
        let _ = app.update(Message::SelectSquare(E4));
        for console in [true, false] {
            app.console_open = console;
            app.pending_promotion = (!console).then_some(Move {
                from: Square::B7,
                to: Square::A8,
                promotion: Some(Piece::Queen),
            });
            app.selected_square = Some(Square::B7);
            for navigation in [
                Navigation::First,
                Navigation::Previous,
                Navigation::Next,
                Navigation::Latest,
                Navigation::Ply(0),
            ] {
                let _ = app.update(Message::Navigate(navigation));
                assert!(!app.history.is_reviewing());
                assert_eq!(app.selected_square, Some(Square::B7));
                assert_eq!(app.pending_promotion.is_some(), !console);
            }
        }
    }

    #[test]
    fn reviewing_mate_does_not_show_the_live_result_on_earlier_positions() {
        use Square::{D8, E5, E7, F2, F3, G2, G4, H4};
        let mut app = app();
        for (from, to) in [(F2, F3), (E7, E5), (G2, G4), (D8, H4)] {
            assert!(app.game.play(Move {
                from,
                to,
                promotion: None
            }));
        }
        let _ = app.update(Message::Navigate(Navigation::Previous));
        assert_eq!(app.history.status(&app.game), GameStatus::Ongoing);
        let _ = app.update(Message::Navigate(Navigation::Next));
        assert_eq!(app.history.status(&app.game), GameStatus::Checkmate(Side::White));
    }
}

#[derive(Debug, Clone)]
enum Message {
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

impl ChessVault {
    fn boot(logs: logs::Logs) -> Self {
        let mut app = Self {
            history: History::default(),
            logs,
            console_open: false,
            log_text: String::new(),
            log_content: text_editor::Content::new(),
            sources_open: false,
            game: Game::new(Position::standard()),
            selected_square: None,
            pending_promotion: None,
            confirm_new_game: false,
            saving_pgn: false,
            pgn_error: None,
        };

        if let Err(err) = app.initialize() {
            error!(error = ?err, "Application initialization failed");
            std::process::exit(1);
        }

        app
    }

    fn initialize(&mut self) -> Result<()> {
        match platform_dirs::user_state_dir(Some("chessvault")).context("Failed to resolve application state directory")
        {
            Ok(path) => std::fs::create_dir_all(&path)
                .with_context(|| format!("Failed to create application state directory: {}", path.display())),
            Err(err) => Err(err),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if self.confirm_new_game
            && matches!(
                message,
                Message::Navigate(_)
                    | Message::SelectSquare(_)
                    | Message::Promote(_)
                    | Message::Resign(_)
                    | Message::ClaimDraw
                    | Message::ClaimDrawWithMove(_)
                    | Message::OfferDraw(_)
                    | Message::AcceptDraw(_)
                    | Message::DeclineDraw(_)
            )
        {
            return Task::none();
        }
        let previous_move_count = self.game.moves().len();
        match message {
            Message::SavePgn => {
                if self.saving_pgn {
                    return Task::none();
                }
                let Some(pgn) = pgn::export(&self.game) else {
                    return Task::none();
                };
                self.saving_pgn = true;
                self.pgn_error = None;
                return Task::perform(
                    async move {
                        let Some(file) = rfd::AsyncFileDialog::new()
                            .add_filter("PGN", &["pgn"])
                            .set_file_name("game.pgn")
                            .save_file()
                            .await
                        else {
                            return Ok(());
                        };
                        file.write(pgn.as_bytes()).await.map_err(|err| err.to_string())
                    },
                    Message::PgnSaved,
                );
            }
            Message::PgnSaved(result) => {
                self.saving_pgn = false;
                if let Err(err) = result {
                    error!(error = %err, "Failed to save PGN");
                    self.pgn_error = Some(format!("Could not save PGN: {err}"));
                }
            }
            Message::NewGame => {
                if self.game.status() == GameStatus::Ongoing && !self.game.moves().is_empty() {
                    self.confirm_new_game = true;
                } else {
                    self.reset_game();
                }
            }
            Message::ConfirmNewGame => {
                if self.confirm_new_game {
                    self.reset_game();
                }
            }
            Message::CancelNewGame => self.confirm_new_game = false,
            Message::Navigate(navigation) => {
                if !self.console_open && self.pending_promotion.is_none() {
                    self.history.sync(&self.game);
                    self.history.navigate(&self.game, navigation);
                    self.selected_square = None;
                    return self.history.scroll_to_current(&self.game);
                }
            }
            Message::SelectSquare(square) => {
                if !self.history.is_reviewing()
                    && self.game.status() == GameStatus::Ongoing
                    && self.pending_promotion.is_none()
                {
                    self.pending_promotion =
                        self.game.position().legal_moves().into_iter().find(|mv| {
                            Some(mv.from) == self.selected_square && mv.to == square && mv.promotion.is_some()
                        });
                    if self.pending_promotion.is_none() {
                        self.selected_square = board::click(&mut self.game, self.selected_square, square);
                    }
                }
            }
            Message::Promote(piece) => {
                if let Some(mv) = self.pending_promotion
                    && !self.history.is_reviewing()
                    && self.game.status() == GameStatus::Ongoing
                    && self.game.play(Move {
                        promotion: Some(piece),
                        ..mv
                    })
                {
                    self.pending_promotion = None;
                    self.selected_square = None;
                }
            }
            Message::CancelPromotion => {
                if self.confirm_new_game {
                    self.confirm_new_game = false;
                } else {
                    // Keep the pawn selected so another destination can be chosen.
                    self.pending_promotion = None;
                }
            }
            Message::OfferDraw(side) => {
                if self.draw_actions_enabled() {
                    self.game.offer_draw(side);
                }
            }
            Message::AcceptDraw(side) => {
                if self.draw_actions_enabled() && self.game.accept_draw(side) {
                    self.selected_square = None;
                }
            }
            Message::DeclineDraw(side) => {
                if self.draw_actions_enabled() {
                    self.game.decline_draw(side);
                }
            }
            Message::ClaimDraw => {
                if !self.history.is_reviewing()
                    && !self.console_open
                    && self.pending_promotion.is_none()
                    && self.game.claim_draw()
                {
                    self.selected_square = None;
                }
            }
            Message::ClaimDrawWithMove(mv) => {
                if !self.history.is_reviewing()
                    && !self.console_open
                    && self.pending_promotion.is_none()
                    && self.game.claim_draw_with_move(mv)
                {
                    self.selected_square = None;
                }
            }
            Message::Resign(side) => {
                if !self.history.is_reviewing() && self.game.resign(side) {
                    self.selected_square = None;
                    self.pending_promotion = None;
                }
            }
            Message::ToggleConsole => {
                self.console_open = !self.console_open;
                debug!(open = self.console_open, "Developer console toggled");
                self.refresh_logs();
            }
            Message::ClearLogs => {
                self.logs.clear();
                self.log_text.clear();
                self.log_content = text_editor::Content::new();
            }
            Message::RefreshLogs => {
                // Keep a selection stable while the user is copying, even if
                // new events arrive. Capture continues in the log buffer.
                if self.log_content.selection().is_none() {
                    self.refresh_logs();
                }
            }
            Message::LogAction(action) => {
                if !action.is_edit() {
                    self.log_content.perform(action);
                }
            }
            Message::CopyLogs => return iced::clipboard::write(self.logs.snapshot()),
            Message::ToggleSources => self.sources_open = !self.sources_open,
            Message::SetSourceEnabled(source, enabled) => {
                self.logs.set_source_enabled(source, enabled);
                self.refresh_logs();
            }
            Message::WindowCloseRequested(id) => {
                return window::size(id).then(move |size| {
                    if let Err(err) = WindowRecreateInfo::from(size).save() {
                        error!(error = ?err, "Failed to save window recreate info");
                    }

                    window::close(id)
                });
            }
        }
        self.history.sync(&self.game);
        if self.game.moves().len() != previous_move_count {
            return self.history.scroll_to_current(&self.game);
        }
        Task::none()
    }

    fn draw_actions_enabled(&self) -> bool {
        !self.history.is_reviewing()
            && !self.console_open
            && self.pending_promotion.is_none()
            && !self.confirm_new_game
            && self.game.status() == GameStatus::Ongoing
    }

    fn reset_game(&mut self) {
        self.game = Game::new(Position::standard());
        self.history = History::default();
        self.selected_square = None;
        self.pending_promotion = None;
        self.confirm_new_game = false;
        self.pgn_error = None;
    }

    fn refresh_logs(&mut self) {
        let logs = self.logs.snapshot();
        if logs != self.log_text {
            self.log_content = text_editor::Content::with_text(&logs);
            self.log_content
                .perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));
            self.log_text = logs;
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let keyboard = iced::event::listen_with(|event, status, _window| match event {
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::F12),
                repeat: false,
                ..
            }) => Some(Message::ToggleConsole),
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(Named::Escape),
                repeat: false,
                ..
            }) => Some(Message::CancelPromotion),
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: Key::Named(key),
                modifiers,
                ..
            }) if status == iced::event::Status::Ignored && modifiers.is_empty() => {
                Navigation::from_key(key).map(Message::Navigate)
            }
            _ => None,
        });

        let close_requests = window::close_requests().map(Message::WindowCloseRequested);

        if self.console_open {
            Subscription::batch([
                keyboard,
                close_requests,
                iced::time::every(Duration::from_millis(250)).map(|_| Message::RefreshLogs),
            ])
        } else {
            Subscription::batch([keyboard, close_requests])
        }
    }

    fn view(&self) -> Element<'_, Message> {
        if self.confirm_new_game {
            return container(
                column![
                    text("Start a new game?").size(24),
                    text("The current game and its move history will be discarded."),
                    row![
                        button("Cancel").on_press(Message::CancelNewGame),
                        button("Start new game").on_press(Message::ConfirmNewGame),
                    ]
                    .spacing(12),
                ]
                .spacing(16),
            )
            .padding(24)
            .center_x(Fill)
            .center_y(Fill)
            .into();
        }
        let game_status = self.history.status(&self.game);
        let ongoing = !self.history.is_reviewing() && game_status == GameStatus::Ongoing;
        let navigation_enabled = !self.console_open && self.pending_promotion.is_none();
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
                    (!self.game.moves().is_empty() || self.game.status() != GameStatus::Ongoing)
                        .then_some(Message::NewGame),
                )
                .width(Fill)
        ]
        .spacing(12)
        .height(Fill);
        let controls = if !self.history.is_reviewing() && self.game.can_claim_draw() {
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
            let enabled = self.draw_actions_enabled();
            if let Some(offering) = self.game.draw_offer() {
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
                let side = self.game.position().side_to_move();
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
        if ongoing && !self.game.can_claim_draw() {
            let mut intended_claims = column![].spacing(8);
            let mut has_intended_claim = false;
            for mv in self.game.position().legal_moves() {
                if self.game.can_claim_draw_with_move(mv) {
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
        if self.game.status() != GameStatus::Ongoing && !self.history.is_reviewing() {
            controls = controls.push(
                button("Save PGN")
                    .on_press_maybe((!self.saving_pgn).then_some(Message::SavePgn))
                    .width(Fill),
            );
            if let Some(error) = &self.pgn_error {
                controls = controls.push(text(error));
            }
        }
        let controls = controls.extend([
            button("Black resigns")
                .on_press_maybe(ongoing.then_some(Message::Resign(Side::Black)))
                .width(Fill)
                .into(),
            status.into(),
            button("White resigns")
                .on_press_maybe(ongoing.then_some(Message::Resign(Side::White)))
                .width(Fill)
                .into(),
            text("Move history").size(20).into(),
            self.history.controls(&self.game, navigation_enabled),
            self.history.move_list(&self.game, navigation_enabled),
        ]);

        let content = column![
            container(
                row![
                    column![
                        board::view(
                            self.history.position(&self.game),
                            self.selected_square,
                            Message::SelectSquare,
                            self.pending_promotion,
                            Message::Promote,
                            Message::CancelPromotion,
                        ),
                        container(text("← Previous · → Next · ↑/Home Start · ↓/End Latest").size(14)).center_x(Fill),
                    ]
                    .spacing(12)
                    .width(Fill)
                    .height(Fill),
                    container(controls).width(260).height(Fill),
                ]
                .spacing(24)
            )
            .padding(24)
            .width(Fill)
            .height(Fill)
        ];

        if !self.console_open {
            return content.into();
        }

        let header = row![
            text("Developer console").size(18),
            Space::new().width(Fill),
            button(if self.sources_open { "Hide sources" } else { "Sources" }).on_press(Message::ToggleSources),
            button("Copy all").on_press(Message::CopyLogs),
            button("Clear").on_press(Message::ClearLogs),
            button("Close (F12)").on_press(Message::ToggleConsole),
        ]
        .spacing(12)
        .align_y(iced::Center);

        let editor = text_editor(&self.log_content)
            .on_action(Message::LogAction)
            .placeholder("No logs match the selected sources.")
            .font(Font::MONOSPACE)
            .size(13)
            .height(Fill);

        let mut body = row![editor].spacing(12).height(Fill);
        if self.sources_open {
            let mut sources = column![text("Log sources").size(16)].spacing(8);
            for (source, enabled) in self.logs.sources() {
                sources = sources.push(
                    checkbox(enabled)
                        .label(source.clone())
                        .on_toggle(move |enabled| Message::SetSourceEnabled(source.clone(), enabled)),
                );
            }
            body = body.push(scrollable(sources).width(200).height(Fill));
        }

        let console = container(column![header, body].spacing(12))
            .padding(16)
            .width(Fill)
            .height(300)
            .style(container::rounded_box);

        content.push(console).into()
    }
}
