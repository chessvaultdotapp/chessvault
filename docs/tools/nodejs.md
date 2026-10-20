# Node.js

[Node.js](https://nodejs.org/en/download) runs [Mermaid CLI](mermaid.md), our diagram
renderer, and [SVGO](svgo.md), our SVG optimizer.
It is not needed to build, test, or run the Rust desktop application.

Install a supported LTS release using the official download instructions, then
verify the installation:

```console
$ node --version
v24.18.0
```

Example versions vary by installation. Next, install [pnpm](pnpm.md) to manage
the diagram and SVG-tooling dependencies.
