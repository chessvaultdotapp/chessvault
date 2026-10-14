# pnpm

[pnpm](https://pnpm.io/installation) manages the website and shared diagram and SVG-tooling dependencies.
It is not required to build, test, or run the Rust desktop application.

Follow the official installation instructions for your [Node.js](nodejs.md)
version, then verify:

```console
$ pnpm --version
12.5.1
```

Example versions vary by installation. After cloning, install the locked
dependencies from the workspace root:

```console
$ pnpm install --frozen-lockfile
Lockfile is up to date, resolution step is skipped
...
```

The root [`package.json`](../../package.json) declares the shared tools, and
[`pnpm-lock.yaml`](../../pnpm-lock.yaml) keeps dependency resolution reproducible.
[`pnpm-workspace.yaml`](../../pnpm-workspace.yaml) includes the `website` package and
allows Puppeteer's browser-download installation script. Website dependencies are
declared in [`website/package.json`](../../website/package.json). For website-only work:

```console
$ pnpm --filter website install --frozen-lockfile
Lockfile is up to date, resolution step is skipped
...
```

Run website tools with `pnpm --filter website exec <command>` from the workspace
root; paths passed to those tools are relative to `website/`.
See [Astro](astro.md) for website development, [Mermaid CLI](mermaid.md) for diagram
generation and browser setup, and [SVGO](svgo.md) for optimizer usage.
