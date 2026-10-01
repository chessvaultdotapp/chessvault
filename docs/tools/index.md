# Development tools

Install only the tools needed for your work. Rust builds do not require the
optional documentation and artwork tools.

## Core development

| Tool | Purpose | Requirement |
| --- | --- | --- |
| [Git](git.md) | Clone the repository and manage source history | Required for contribution |
| [Rust and Cargo](rust.md) | Build, test, and run the application | Required |
| [cargo-nextest](cargo-nextest.md) | Run the preferred test suite | Required by `just check`; Cargo test fallback available |

## Optional tools

| Tool | Purpose |
| --- | --- |
| [just](just.md) | Run repository development tasks |
| [Node.js](nodejs.md) | Run SVG tooling |
| [pnpm](pnpm.md) | Install shared SVG-tooling dependencies |
| [SVGO](svgo.md) | Optimize SVG artwork and diagrams |
| [uv](uv.md) | Manage Python installations, environments, and tools |
| [Python](python.md) | Run Python-based documentation and release tooling |
| [Go](go.md) | Install and run Go-based tools |
| [actionlint](actionlint.md) | Check GitHub Actions workflows |
| [Tombi](tombi.md) | Format and lint TOML |
| [rumdl](rumdl.md) | Lint Markdown |
| [GitHub CLI](github-cli.md) | Manage pull requests, issues, and Actions runs |

## Repository tools

- [Release tag tool](tag-tool.md) — update and commit the desktop
  version and lockfile, then create a local release tag.

Commands follow the [shell command conventions](../reference.md#shell-commands).
Example versions and output are illustrative, not additional version requirements
unless explicitly stated.

Return to [Getting started](../development/getting-started.md) for cloning and
running Chessvault, or [Testing](../development/testing.md) for verification.
