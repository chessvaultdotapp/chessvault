# Astro Check

[Astro Check](https://docs.astro.build/en/reference/cli-reference/#astro-check)
reports diagnostics and type errors in the website, including `.astro` components.
The package is named `@astrojs/check`; its command is `astro check`.

## Setup and usage

Follow the [Astro setup](astro.md#setup). The website declares both
`@astrojs/check` and [TypeScript](typescript.md) as development dependencies.
Run from the workspace root:

```console
$ pnpm --filter website exec astro check
...
```

The command checks without producing a production build and exits unsuccessfully
on errors. It uses [`website/tsconfig.json`](../../website/tsconfig.json), which
extends Astro's strict configuration.

The [website workflow](../../.github/workflows/website.yaml) runs this check after
[oxfmt](oxfmt.md) and [oxlint](oxlint.md). `astro build` does not replace it.
