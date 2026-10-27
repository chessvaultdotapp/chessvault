use super::*;
use chess_core::{Piece, Side, Square};

use crate::{
    components::move_history::{self as history, Navigation},
    logs,
};

mod draw_tests {
    use super::*;
    use chess_core::DrawReason;

    fn ready() -> State {
        let mut app = State {
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

    fn intended_claim_ready() -> (State, Move) {
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

mod resignation_tests {
    use super::*;

    #[test]
    fn resigning_ends_play_and_clears_selection() {
        for side in [Side::White, Side::Black] {
            let mut app = State {
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

mod promotion_tests {
    use super::*;

    fn ready() -> State {
        use Square::{A2, A4, A5, A6, B7, B8, C6, F3, G1, G2, H3, H4, H5, H7};
        let mut app = State {
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

mod navigation_tests {
    use super::*;

    fn app() -> State {
        State {
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
    fn enabling_multiline_dependency_logs_refreshes_console() {
        use tracing_subscriber::layer::SubscriberExt;

        let mut app = app();
        let subscriber = tracing_subscriber::registry().with(app.logs.capture_layer());
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(target: "iced_wgpu", "Settings {{\n    present_mode: AutoVsync,\n    backends: Backends(\n        VULKAN | GL,\n    ),\n}}");
        });
        let _ = app.update(Message::SetSourceEnabled("iced_wgpu".into(), true));
        assert!(app.log_text.contains("AutoVsync"));
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
