# ChessVault website

Minimal Astro project, managed by the repository's pnpm workspace. Requires Node.js 22.12 or newer.

Run these commands from the repository root:

| Command | Action |
| --- | --- |
| `pnpm install --frozen-lockfile` | Install workspace dependencies |
| `pnpm --filter website dev` | Start the development server at `localhost:4321` |
| `pnpm --filter website build` | Build the static site into `website/dist/` |
| `pnpm --filter website preview` | Preview the production build |

Pages live in `src/pages/`; static assets live in `public/`.

## Smoke tests

From the repository root, install Chromium once and run the Playwright suite:

```sh
pnpm --filter website exec playwright install --with-deps chromium
pnpm --filter website test:e2e
```

The suite builds the production site with the deployment base path (`/chessvault/`), starts an Astro preview
server on `127.0.0.1:4321`, and stops it afterward. Keep that port free; an existing server is never reused.
Both desktop and mobile Chromium viewports check page headings, header and documentation navigation,
footer rendering, horizontal overflow, theme switching and persistence, keyboard header navigation,
external-link destinations, and unexpected console errors or uncaught exceptions.
The footer currently contains copyright text only, not navigation links.
Tests do not visit external links; third-party availability and external-link validation are outside this suite.
CI runs the same suite alongside formatting, linting, and Astro diagnostics.

To debug with a visible browser, run `pnpm --filter website test:e2e --headed`.
Failed runs retain traces in `website/test-results/`; open one with
`pnpm --filter website exec playwright show-trace test-results/<test-directory>/trace.zip`.

Include `playwright.config.ts tests` alongside `src astro.config.mjs` when running the website's
`oxfmt --check` and `oxlint --deny-warnings` checks.
