# Astro

[Astro](https://docs.astro.build/) builds the website in [`website/`](../../website/).
It is separate from the Rust desktop application and the Zensical documentation sites.

## Setup

Install [Node.js](nodejs.md) and [pnpm](pnpm.md), then install the locked website
dependencies from the workspace root:

```console
$ pnpm --filter website install --frozen-lockfile
Lockfile is up to date, resolution step is skipped
...
```

[`website/package.json`](../../website/package.json) declares Astro and
`@astrojs/compiler-rs`, its additional compiler dependency. Both are installed
by pnpm; no separate global installation is needed.

## Usage

From the workspace root, start the development server in background mode:

```console
$ pnpm --filter website exec astro dev --background
...
$ pnpm --filter website exec astro dev status
...
$ pnpm --filter website exec astro dev logs
...
$ pnpm --filter website exec astro dev stop
...
```

Build the site and preview the result locally:

```console
$ pnpm --filter website build
...
$ pnpm --filter website preview
...
```

The build writes to `website/dist/`. Configuration lives in
[`website/astro.config.mjs`](../../website/astro.config.mjs).
A successful build does not replace [Astro Check](astro-check.md),
[formatting](oxfmt.md), or [linting](oxlint.md); see
[website checks](../development/testing.md#website-checks).
