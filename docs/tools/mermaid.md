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

## CI diagram cache

Markdown checks and documentation builds share the local
[prepare-diagrams action](../../.github/actions/prepare-diagrams/action.yml).
It caches only the generated `architecture.svg` and `resolution.svg`, not Node
modules or application artwork. Both workflows use the same exact cache key,
covering runner OS/architecture, Chrome version, `.mmd` sources, the generation
justfile, package manifests and lockfile, and the preparation/setup action code.
Tool versions and generation settings in the action therefore invalidate the cache.
There are no fallback restore keys: changed inputs regenerate the diagrams.

On a cache hit, CI skips Node, pnpm, dependency installation, and Mermaid/SVGO
execution. On a miss, it renders with the runner's sandbox-enabled Chrome and
skips downloading Puppeteer's browser. Both paths verify the expected SVGs exist
and are nonempty. Markdown link checks and both documentation builds still run;
a diagram cache hit does not skip validation. Local generation is unchanged.

Concurrent cold runs may both generate diagrams before one saves the shared key.
GitHub's cache branch scoping still applies: main caches can serve PRs, while a
PR-created cache cannot populate the main-branch cache. The first main run after
an input change may therefore be cold even if a PR already generated the diagrams.

After rollout, inspect the `Cache generated diagrams` step for exact hits and
confirm the four tooling/rendering steps are skipped on prose-only changes.
Compare total Markdown/documentation job times, including restore overhead, with
cold runs. Changing a `.mmd` source, the lockfile, or the generation action should
produce a miss. Hosted cache reuse and actual savings require subsequent CI runs.

## Generate diagrams

Edit the `.mmd` sources in `crates/platform-dirs/docs/`, then run from the workspace
root with [just](just.md) installed:

```console
$ just --justfile crates/platform-dirs/justfile diagram
...
```

This renders both diagrams with transparent backgrounds and no embedded fonts,
then optimizes the SVGs with [SVGO](svgo.md). Disabling font embedding keeps files
small; viewers use locally available fonts and fallbacks instead.
Generate diagrams before building or previewing the
documentation site. Review the rendered diagrams and commit only the `.mmd`
source changes; the generated SVGs are ignored by Git. GitHub Markdown previews
will not display these untracked diagrams; use the built documentation site.
`just optimize-assets` also regenerates these diagrams while optimizing desktop
artwork.

To render one source directly:

```console
$ cd crates/platform-dirs
$ pnpm exec mmdc -i docs/architecture.mmd -o docs/architecture.svg -b transparent --no-font-embed
Generating single mermaid chart
```

The direct command does not run SVGO. Background transparency does not override
explicit node or container fills in the Mermaid source.
