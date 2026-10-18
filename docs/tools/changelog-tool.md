# Stable changelog drafts

`tools/changelog-tool/main.py` uses Python's standard library and Git to assemble a reviewable draft.
Run it from the workspace root with Python 3.10 or newer:

```sh
python3 tools/changelog-tool/main.py --stdout
python3 tools/changelog-tool/main.py
```

The default output is `changelogs/unreleased.md`. Existing files are never overwritten.
`--stdout` previews the draft without writing, even when that file exists.

## Release boundaries

By default, the tool finds the nearest reachable plain `vMAJOR.MINOR.PATCH` tag in Git's
 topological traversal, not the highest version or newest tag date. For merged release lines,
use `--base v1.2.3` to select the intended boundary explicitly.
`--revision <commit>` selects the release commit instead of HEAD. The base must be reachable from it.
For a first release without a stable predecessor, pass `--first-release` explicitly;
this collects all reachable history and cannot be combined with `--base`.

Development tags must end in build metadata like `26w40a` or `26w40aa`; earlier build metadata
such as `v1.2.3+build.01.26w40aa` is supported. Tags outside the selected ancestry are excluded.
The range excludes every ancestor of the base, including the base itself.
Sources are ordered oldest-first by reversed Git topological traversal, with tags on the same commit
sorted by name. Each source file is read from its tagged commit, never from the working tree.
Missing files produce warnings and a note in the draft; other available sources are still included.

## Review and tag

The draft includes empty Added, Fixed, Changed, and Breaking changes categories followed by
source-tag sections. Move entries into the appropriate categories, manually deduplicate them,
remove superseded details, and retain useful source-tag references. No semantic rewriting is attempted.

Commit the reviewed `changelogs/unreleased.md`, then run the [tag tool](tag-tool.md):

```sh
git add changelogs/unreleased.md
git commit -m '[docs] Prepare stable release changelog'
venvs/tag-tool/bin/python tools/tag-tool/main.py v1.2.3
```

The tag tool requires a clean tree and renames the committed draft to `changelogs/v1.2.3.md`,
updates its title, and includes it in the release commit. Draft generation does not commit,
tag, push, or publish anything. Stable GitHub release publishing is not provided by this tool.

## Tests

The existing tag-tool environment supplies pytest; no additional runtime dependencies are needed:

```sh
venvs/tag-tool/bin/python -m pytest tools/changelog-tool -q
```
