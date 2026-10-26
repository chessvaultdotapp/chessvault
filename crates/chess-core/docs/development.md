# Development

Tests live beside implementations and cover representation, move legality,
special moves, notation, history, draw rules, and result handling.
Follow the [workspace testing guide](https://chessvaultdotapp.github.io/chessvault/dev/docs/development/testing/)
for ordered checks with `-p chess-core`; use workspace checks for public APIs
affecting the desktop. Markdown examples are illustrative; put continuously
tested examples in Rustdoc.

## Documentation builds

The package-local Zensical configuration follows the existing `platform-dirs`
setup. From the workspace root, install the pinned documentation environment
with `just --justfile venvs/justfile zensical`, then build:

```sh
venvs/zensical/bin/zensical build --config-file crates/chess-core/zensical.toml
```

Output goes to `crates/chess-core/site/`. The documentation workflow builds the
shared and package-local roots and assembles this root at
`dev/docs/crates/chess-core/`. Cross-root links use published URLs; source-code
links use GitHub because Rust sources are not part of the generated site.
