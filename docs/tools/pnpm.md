# pnpm

[pnpm](https://pnpm.io/installation) manages the shared SVG-tooling dependencies.
It is not required to build, test, or run Chessvault.

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
See [SVGO](svgo.md) for optimizer usage.
