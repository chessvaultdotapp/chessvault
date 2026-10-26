# SVGO

[SVGO](https://svgo.dev/) optimizes SVG artwork and documentation diagrams.
It is a local development dependency in the root
[`package.json`](../../package.json), not a global installation.
It is not required to build, test, or run Chessvault: Cargo embeds the existing
SVG files directly.

Install [Node.js](nodejs.md) and [pnpm](pnpm.md), then install the locked
dependencies from the workspace root:

```console
$ pnpm install --frozen-lockfile
Lockfile is up to date, resolution step is skipped
...
$ pnpm exec svgo --help
Usage: svgo [options] [INPUT...]
...
```

Output is abbreviated. SVGO is not part of the Cargo build. Desktop artwork
optimization is manual; the platform-dirs documentation workflow regenerates
and optimizes its [Mermaid diagrams](mermaid.md) before building the site.

See the [SVG optimization workflow](https://chessvaultdotapp.github.io/chessvault/dev/docs/apps/chessvault/development/#artwork-and-icons)
for usage and visual verification. With [just](just.md) installed,
`just optimize-assets` optimizes desktop artwork in place and regenerates the
documentation diagrams.
