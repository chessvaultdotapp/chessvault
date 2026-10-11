# Chessvault

Chessvault is an early-stage chess desktop application built with Rust and Iced.
This repository contains the desktop app and its supporting Rust libraries.

## Current status

- Play both sides on an interactive board with legal-move hints. Click a piece,
  then a highlighted destination to move.
- Legal moves include castling, en passant, and promotion with an on-board piece chooser.
- Checkmate, stalemate, and resignation end the game, with the result displayed
  beside the board. Other draw rules are not yet adjudicated.
- Review numbered moves in standard algebraic notation. Use **Left/Right** for
  previous/next, **Up/Home** for the starting position, and **Down/End** for latest,
  or click the navigation buttons or a move. Shortcut hints appear below the board.
  Review is read-only: return to latest to resume play. Navigation is disabled
  while the console or promotion picker is open.
- Window dimensions are saved between launches. Press **F12** to toggle the
  developer console.

The app is under active development, not a complete chess platform. Packaged
builds currently target Linux x86_64; native state-directory support is
Linux-only.

## Getting started

Development prereleases are available on the
[GitHub releases page](https://github.com/chessvaultdotapp/chessvault/releases),
including Linux x86_64 AppImage and native archive downloads. See the
[AppImage guide](docs/development/appimage.md) for launch instructions,
requirements, and compatibility caveats.

To run from source, follow the
[setup guide](docs/development/getting-started.md), then launch from the repository
root in a graphical desktop session:

```sh
cargo run -p chessvault --locked
```

## Documentation

General project documentation is available in [docs](docs/index.md), including
architecture and component guides.

## Development

See the [development documentation](docs/development/index.md) for setup,
building, running, and testing.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution workflow and commit
message conventions.

## Acknowledgements

### Chess piece assets

Chess piece artwork by [RhosGFX](https://rhosgfx.itch.io/), from the
[Vector Chess Pieces pack on itch.io](https://rhosgfx.itch.io/vector-chess-pieces),
released under [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/).
The SVGs used here were sourced from [Lichess's lila repository](https://github.com/lichess-org/lila)
and adapted for Chessvault.
