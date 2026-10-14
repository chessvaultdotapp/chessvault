# oxfmt

[oxfmt](https://oxc.rs/docs/guide/usage/formatter) formats website source and
configuration. It is installed as a website development dependency.

## Setup and usage

Follow the [Astro setup](astro.md#setup). From the workspace root, run the same
formatting check as the [website workflow](../../.github/workflows/website.yaml):

```console
$ pnpm --filter website exec oxfmt --check src astro.config.mjs
...
```

`--check` reports formatting issues without modifying files and fails when
formatting differs. To apply formatting, omit `--check`:

```console
$ pnpm --filter website exec oxfmt src astro.config.mjs
...
```

Paths are relative to `website/` because pnpm executes in the selected package.
Settings live in [`website/.oxfmtrc.json`](../../website/.oxfmtrc.json).
This tool does not replace [rumdl](rumdl.md) for shared Markdown documentation
or Cargo formatting for Rust.
