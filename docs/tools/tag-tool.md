# Release tag tool

[`tools/tag-tool/main.py`](../../tools/tag-tool/main.py) updates the desktop
version, commits the release files, and creates a local lightweight Git tag.
It does not push commits or tags, build binaries, or publish a release.

Commands follow our [shell command conventions](../reference.md#shell-commands).
Run the examples from the workspace root. Output is illustrative; dates,
versions, and commit IDs vary.

## Setup

Install [uv](uv.md), [just](just.md), [Git](git.md), and the repository's pinned
[Rust toolchain](rust.md). Configure your Git commit identity before
creating a release.

Create the dedicated environment and install its pinned dependencies:

```console
$ just --justfile venvs/justfile tag-tool
uv venv tag-tool
...
uv pip install --python tag-tool/bin/python --requirements tag-tool.requirements.txt
...
```

The recipe runs inside `venvs/` and creates the Git-ignored `venvs/tag-tool/`
environment. See [Python environment recipes](just.md#python-environments) for
Python version selection. These recipes use the POSIX environment layout;
on Windows, run the uv commands manually with `Scripts/python.exe` instead of
`bin/python`. Subsequent examples use `venvs/tag-tool/bin/python`; on Windows,
use `venvs/tag-tool/Scripts/python.exe`.

The tool refreshes `Cargo.lock` with `cargo metadata --offline --format-version 1`.
Prepare the Cargo dependency cache beforehand if necessary:

```console
$ cargo fetch --locked
...
```

## Explicit release

Start on the branch you intend to release, with all intended changes committed.
The working tree must be clean, including staged changes and untracked files.
Ignored files, such as the virtual environment, do not block a release.

```console
$ venvs/tag-tool/bin/python tools/tag-tool/main.py v1.2.3
Desktop version: 0.0.0 -> 1.2.3
...
Created tag: v1.2.3
```

Tags require a lowercase `v` prefix followed by a
[SemVer 2.0.0](https://semver.org/spec/v2.0.0.html) version. The manifest stores the
version without `v`. Prerelease and build metadata are supported, for example
`v1.2.3-rc.1+build.01`; incomplete versions and numeric components with forbidden
leading zeros are rejected.

The tool performs these steps in order:

1. Reject a dirty working tree.
2. Determine the version and reject an existing tag of the same name.
3. Update only `[package].version` in `apps/chessvault/Cargo.toml`, preserving
   unrelated content and formatting.
4. Refresh `Cargo.lock` offline using Cargo.
5. If `changelogs/unreleased.md` exists, rename it to `changelogs/<tag>.md`
   and replace its opening `# Unreleased` title with `# <tag>`.
   The tool rejects an existing destination before changing any release files.
6. Stage and commit the desktop manifest, lockfile, and optional changelog rename with a message such as
   `[desktop] Release v1.2.3`.
7. Create `v1.2.3` at the new commit, which contains the release version.

Commit `changelogs/unreleased.md` before running the tool so the working tree is
clean. The rename and title update apply to both explicit and development releases.
Other contents and line endings are preserved; files without the opening title are
renamed without content changes. If the file is absent, no changelog is created.

Choose a version that changes the release files. If there is nothing to commit,
Git fails and no tag is created.

## Development release

To derive a tag from the current desktop version instead of supplying one:

```console
$ venvs/tag-tool/bin/python tools/tag-tool/main.py --dev-rel
Desktop version: 1.2.3 -> 1.2.3+26w40a
...
Created tag: v1.2.3+26w40a
```

`--dev-rel` cannot be combined with an explicit tag. It preserves the base
version and appends SemVer build metadata with a two-digit ISO week-year,
`w`, a two-digit ISO week number, and a lowercase alphabetic suffix. New metadata
uses the current date in UTC. For example, ISO week 40 of 2026 becomes `26w40a`.

If trailing week metadata already exists, its week is retained and its suffix
is incremented, even if the current week has changed:

| Current version | Next development version |
| --- | --- |
| `1.2.3+26w40` | `1.2.3+26w40a` |
| `1.2.3+26w40a` | `1.2.3+26w40b` |
| `1.2.3+26w40z` | `1.2.3+26w40aa` |
| `1.2.3+26w40aa` | `1.2.3+26w40ab` |

Existing build metadata is preserved: in week 40 of 2026, `1.2.3+build.01`
becomes `1.2.3+build.01.26w40a`. Build metadata does not affect SemVer precedence;
these suffixes distinguish tags, not version ordering.

Development releases update and commit the manifest and lockfile just like
explicit releases.

### Publish on GitHub

After reviewing the generated commit and tag, push the development tag explicitly:

```console
$ git push origin v1.2.3+26w40a
...
```

The [development release workflow](../../.github/workflows/development-release.yml)
creates a GitHub **prerelease** with generated release notes. If the tagged commit
contains `changelogs/<dev-tag>.md` (for example, `changelogs/v1.2.3+26w40a.md`),
its contents are included in the release body before the generated notes. Without
that file, only generated notes are used. It does not mark the prerelease
as the latest release or build/upload binaries. Existing releases are left unchanged
when a workflow is rerun. Stable tags and tags without trailing development week
metadata are ignored.

The workflow must be included in the tagged commit. It uses the built-in
`GITHUB_TOKEN` with `contents: write`; no additional secret is needed. Push using
your normal Git credentials: pushes made by another workflow's `GITHUB_TOKEN`
do not trigger this workflow. Pushing the tag publishes its commit, but does not
update the remote branch; push the branch separately if desired.

## Failures and recovery

Commands stop on failure. The tool does not roll back modified files, staged
changes, or an already-created commit, and it never overwrites an existing tag.

Before retrying, inspect the repository:

```console
$ git status --short
...
$ git diff HEAD -- apps/chessvault/Cargo.toml Cargo.lock
...
$ git log -1 --oneline
...
```

If Cargo or the commit fails, review and resolve the remaining changes first;
a retry requires a clean tree. If the commit succeeds but tagging fails, verify
that the intended release commit is at `HEAD` and create the missing tag manually
rather than rerunning `--dev-rel` and incrementing the suffix again. Do not move
an existing release tag without investigating why it already exists.

Review the release commit and tag before any manual push.

## Tests

```console
$ venvs/tag-tool/bin/python -m pytest tools/tag-tool -q
...
66 passed in 0.12s
```

Tests cover SemVer validation, suffix progression, manifest preservation,
clean-tree and existing-tag checks, command ordering, and failure propagation.
Most CLI tests mock external commands. An integration test uses real Git in a
temporary repository with Cargo mocked, verifying that the tag contains both
committed release files. Tests do not create tags in your working repository.
