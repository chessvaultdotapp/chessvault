use anyhow::{Context, Result};
use chess_core::{Game, Move, Position, Square};
use tracing::error;

use crate::{
    components::{developer_console, move_history::History},
    logs,
};

mod message;
mod subscription;
mod update;
mod view;

pub(crate) use message::Message;

pub(crate) struct State {
    pub(crate) console: developer_console::State,
    pub(crate) game: Game,
    pub(crate) history: History,
    pub(crate) selected_square: Option<Square>,
    pub(crate) pending_promotion: Option<Move>,
    pub(crate) confirm_new_game: bool,
    pub(crate) saving_pgn: bool,
    pub(crate) pgn_error: Option<String>,
}

impl State {
    pub(crate) fn boot(logs: logs::Logs) -> Self {
        let mut app = Self {
            history: History::default(),
            console: developer_console::State::new(logs),
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
}
