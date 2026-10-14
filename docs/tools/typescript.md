# TypeScript

[TypeScript](https://www.typescriptlang.org/docs/) provides type checking for the
[Astro website](astro.md). It is a website development dependency, not a
requirement for the Rust application.

## Setup and usage

Follow the [Astro setup](astro.md#setup) to install the locked TypeScript version.
Verify it from the workspace root:

```console
$ pnpm --filter website exec tsc --version
Version 6.0.3
```

Example versions vary with the lockfile. The website's
[`tsconfig.json`](../../website/tsconfig.json) extends `astro/tsconfigs/strict`,
includes generated Astro types and project files, and excludes `dist`.

Use [Astro Check](astro-check.md) for project verification:

```console
$ pnpm --filter website exec astro check
...
```

Standalone `tsc` does not check `.astro` components, so it is not a replacement
for this command. See [Astro's TypeScript guide](https://docs.astro.build/en/guides/typescript/)
for component typing and configuration.
