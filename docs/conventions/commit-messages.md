# Commit messages

Prefix commit subjects with the relevant tags, followed by a short, imperative
description of the change:

```text
[desktop] Preserve console selection during refresh
```

Use wording such as “Add”, “Fix”, or “Preserve” rather than “Added” or “Fixed”.
Keep the subject focused on what the commit accomplishes; use the commit body
when additional rationale is useful.

## Tags

| Tag | Use |
| --- | --- |
| `[desktop]` | Desktop application changes, including its tests and package-specific documentation |
| `[chess-core]` | Chess core changes, including its tests and package-specific documentation |
| `[platform-dirs]` | Platform directory resolution changes, including its tests and package-specific documentation |
| `[application-runtime]` | Application runtime changes, including its tests and package-specific documentation |
| `[docs]` | Documentation changes |
| `[ci]` | CI configuration and CI-specific documentation |

Combine tags when applicable, placing them next to each other without spaces.
A commit changing both the desktop and chess core uses
`[desktop][chess-core]`. Combine `[ci]` with package tags when CI changes also
change a package.

Shared documentation uses `[docs]` without a package tag. Package-specific
documentation retains its package tag; CI-specific documentation also uses
`[ci]`. Tests use the tag of the package they test.

Workspace-only infrastructure changes do not require a package tag; use `[ci]`
when the change is CI-specific.

## Examples

```text
[desktop] Preserve console selection during refresh
[chess-core] Add empty position construction
[desktop][chess-core] Display core positions in the desktop app
[platform-dirs] Handle invalid XDG state paths
[application-runtime] Add application state directory resolution
[docs] Add development setup instructions
[docs][desktop] Explain console source filtering
[ci] Run tests before formatting and linting
[docs][ci] Document nextest setup in CI
```

See [Contributing](../../CONTRIBUTING.md#small-atomic-focused-commits) for guidance
on keeping commits small, atomic, and focused.
