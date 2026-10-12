# Mermaid CLI

[Mermaid CLI](https://github.com/mermaid-js/mermaid-cli) renders Mermaid `.mmd`
sources into documentation SVGs. It is a local development dependency in the
root [`package.json`](../../package.json), not a global installation. It is not
required to build, test, or run Chessvault.

## Installation

Install [Node.js](nodejs.md) and [pnpm](pnpm.md), then run from the workspace root:

```console
$ pnpm install --frozen-lockfile
Lockfile is up to date, resolution step is skipped
...
$ pnpm exec mmdc --version
12.0.0
```

Mermaid CLI uses Puppeteer to render diagrams in a headless browser. The root
[`pnpm-workspace.yaml`](../../pnpm-workspace.yaml) approves Puppeteer's installation
script, which downloads browser binaries into `~/.cache/puppeteer` by default.
Installation therefore needs network access and space for those binaries.
Do not enable all dependency scripts to work around an installation failure.
See [Puppeteer's troubleshooting guide](https://pptr.dev/troubleshooting) for
browser startup failures or missing system libraries.

## Generate diagrams

Edit the `.mmd` sources in `crates/platform-dirs/docs/`, then run from the workspace
root with [just](just.md) installed:

```console
$ just --justfile crates/platform-dirs/justfile diagram
...
```

This renders both diagrams with transparent backgrounds, then optimizes the SVGs
with [SVGO](svgo.md). Review the rendered diagrams and commit the source and SVG
changes together. `just optimize-assets` also regenerates these diagrams while
optimizing desktop artwork.

To render one source directly:

```console
$ pnpm exec mmdc -i crates/platform-dirs/docs/architecture.mmd -o crates/platform-dirs/docs/architecture.svg -b transparent
Generating single mermaid chart
```

The direct command does not run SVGO. Background transparency does not override
explicit node or container fills in the Mermaid source.
