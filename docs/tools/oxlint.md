# oxlint

[oxlint](https://oxc.rs/docs/guide/usage/linter) lints website code. It is installed
as a website development dependency, separate from Rust's Clippy checks.

## Setup and usage

Follow the [Astro setup](astro.md#setup). From the workspace root, run the same
lint command as the [website workflow](../../.github/workflows/website.yaml):

```console
$ pnpm --filter website exec oxlint --deny-warnings src astro.config.mjs
...
```

Paths are relative to `website/`. `--deny-warnings` makes warnings fail the check
as well as errors. The command does not modify source files.

Linting does not replace [oxfmt](oxfmt.md) or
[Astro diagnostics and type checking](astro-check.md). In particular, do not
interpret a successful lint run as verification of every `.astro` component;
run the complete [website checks](../development/testing.md#website-checks).
