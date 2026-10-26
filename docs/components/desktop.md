# Desktop application

The binary-only `chessvault` package uses Iced 0.14 and depends directly on
`chess-core` and `platform-dirs`. It owns UI state, filesystem effects, logging,
and window persistence; chess rules and game history belong to the core.

It supports local play, legal-move hints, promotion selection, read-only history
review, results, and finished-game PGN export.

Detailed documentation lives alongside the application in
[`apps/chessvault/docs`](https://github.com/chessvaultdotapp/chessvault/tree/main/apps/chessvault/docs):

- [Desktop documentation](https://chessvaultdotapp.github.io/chessvault/dev/docs/apps/chessvault/)
- [Architecture and UI flow](https://chessvaultdotapp.github.io/chessvault/dev/docs/apps/chessvault/architecture/)
- [Runtime state and logging](https://chessvaultdotapp.github.io/chessvault/dev/docs/apps/chessvault/runtime/)
- [Development and verification](https://chessvaultdotapp.github.io/chessvault/dev/docs/apps/chessvault/development/)

Shared [setup](../development/getting-started.md), [testing](../development/testing.md),
and [workspace architecture](../architecture/index.md) remain in this documentation root.
