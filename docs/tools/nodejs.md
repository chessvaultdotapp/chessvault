# Node.js

[Node.js](https://nodejs.org/en/download) runs [Mermaid CLI](mermaid.md), our diagram
renderer, [SVGO](svgo.md), our SVG optimizer, and the [Astro website](astro.md).
It is not needed to build, test, or run the Rust desktop application.
The website requires Node.js 22.12.0 or newer, as declared in
[`website/package.json`](../../website/package.json).

Install a supported LTS release using the official download instructions, then
verify the installation:

```console
$ node --version
v24.18.0
```

Example versions vary by installation. Next, install [pnpm](pnpm.md) to manage
the website and shared tooling dependencies.
