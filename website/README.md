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
