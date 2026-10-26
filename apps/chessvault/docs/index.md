# Desktop application

`chessvault` is a binary-only Rust application built with Iced 0.14. It supports
local play for both sides, legal-move hints, promotion choice, read-only history,
game results, and finished-game PGN export. There is no PGN import or chess-position persistence.

- [Architecture and UI flow](architecture.md): ownership, messages, board interactions, and source map.
- [Runtime state and logging](runtime.md): window persistence, console, and failure handling.
- [Desktop development](development.md): running, assets, checks, and interaction checklist.

Start with the shared
[getting-started guide](https://chessvaultdotapp.github.io/chessvault/dev/docs/development/getting-started/)
for tool installation and workspace setup. The
[workspace architecture](https://chessvaultdotapp.github.io/chessvault/dev/docs/architecture/)
explains the boundaries between the desktop, chess core, and platform directory resolver.
