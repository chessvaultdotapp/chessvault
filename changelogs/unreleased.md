# Unreleased

## Added

- Added a **Save PGN** button alongside the finished-game result. Save the complete game to a `.pgn` file as
  move numbers, SAN moves, and the final result, without headers, comments, or variations.

- Added a **Claim draw** button for threefold repetition and the fifty-move rule. Qualifying intended moves also
  appear as draw-claim buttons, allowing a claim before playing the move without changing the board or move history.
  The fifty-move clock resets after any pawn move or capture.

- Added automatic draws for fivefold repetition and the seventy-five-move rule. Checkmate on the final move takes
  precedence over a seventy-five-move draw.

- Added automatic dead-position draws for bare kings, king and one bishop or knight against a bare king, and
  bishop-only material where all bishops occupy the same square colour. Other dead positions, such as blocked-pawn
  positions, are not yet detected. Two knights and opposite-coloured bishops are not automatically treated as dead.

- Added draw offers with opponent accept and decline controls. An offer survives the offering player's move;
  the opponent's next legal move automatically declines it. Accepted offers end the game by agreement without
  changing the board or move history. Draw actions are unavailable during history review or modal interactions.

- A resizable chessboard with coordinates and chess pieces in the standard starting position.

- Click-to-move controls with legal move validation, including castling and en passant.

- Green highlighting for the selected piece, with dots for legal destinations and rings for captures.

- Window size is remembered between sessions.

- Added an application/window icon and corrected Linux dock icon association.

- An in-app log console opened with **F12**, with source filters, selectable text, copy-all, and clear controls.

- Grayscale board colors and refined piece artwork with more consistent colors and fewer rendering seams.

- Development releases now include a downloadable Linux x86-64 binary in a `.tar.gz` archive.
  Extract the archive and run `./bin/chessvault` in a graphical Linux session.
  Compatible system libraries and graphics drivers are required.

- Added a centered result box beside the board showing the winner after checkmate or resignation, or the draw result.

- Added resignation buttons for White and Black. Resigning ends the game and disables further play.

- Added numbered move history in standard algebraic notation with first, previous, next, and latest navigation.
  Use the buttons, click a move, or use Left/Right, Up/Home, and Down/End; shortcut hints appear below the board.
  The viewed move is highlighted, and earlier positions are read-only to preserve the continuation. Return to latest
  to resume play ([#19](https://github.com/chessvaultdotapp/chessvault/issues/19)).

- Simplified move-history labels by omitting repeated move numbers for Black and removing the redundant live-position
  caption.

- Added an on-board promotion picker to choose a queen, knight, rook, or bishop instead of automatically promoting
  to a queen. Click outside the picker or press Escape to cancel
  ([#16](https://github.com/chessvaultdotapp/chessvault/issues/16)).

- Added timestamped JSON log files to the application log directory, including application exit errors, for
  troubleshooting ([#15](https://github.com/chessvaultdotapp/chessvault/issues/15)).

- Added a red glow behind the king when it is in check. The glow clears when check is resolved and remains visible
  after checkmate ([#17](https://github.com/chessvaultdotapp/chessvault/issues/17)).

- Added a per-user Linux installer to native release tarballs. Run `./install-linux.sh` to install the executable,
  application-menu launcher, icons, and license without sudo. Supports installation previews with `--dry-run`,
  a custom executable prefix with `--prefix`, and `XDG_DATA_HOME` for desktop integration files.
  Running the installer again reinstalls or upgrades a recorded installation, backing up existing files before
  replacement and attempting rollback on failure. Modified or untracked files are not overwritten.
  Use `--uninstall` to remove a recorded installation while preserving user state and unrelated files;
  removal also supports `--dry-run`. Installations made without a manifest still require manual removal.

- Added a README and license to Linux distribution packages, with instructions for running, installing, upgrading,
  and uninstalling the application, including manual removal of older installations without a manifest.

- Added a **New game** button to reset the board and move history. Starting over during an ongoing game asks for
  confirmation; choose Cancel or press Escape to keep the current game. The button is available after a move is played
  or the game ends, including games ended without any moves.

- Added a website installation guide for Linux development previews, covering installation, upgrades, and removal.
