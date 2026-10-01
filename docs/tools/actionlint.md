# actionlint

[actionlint](https://github.com/rhysd/actionlint) checks GitHub Actions workflows
for syntax errors, invalid expressions, and other common mistakes. It is not
required to build or run Chessvault.

## Installation and local usage

Install [Go](go.md), then run actionlint from the workspace root. The root
[`go.mod`](../../go.mod) pins it as a Go tool dependency, and
[`go.sum`](../../go.sum) records dependency checksums. No global installation is
needed; Go downloads and builds the tool on first use.

```console
$ go tool actionlint -version
v1.7.12
...
```

Check all workflows with `go tool actionlint`, or pass specific files:

<!-- rumdl-disable MD014 -->
```console
$ go tool actionlint -shellcheck= -pyflakes= .github/workflows/docs-platform-dirs.yml
```
<!-- rumdl-enable MD014 -->

Successful checks produce no output. The explicit flags disable optional
ShellCheck and Pyflakes integrations, matching CI regardless of which analyzers
are installed locally. Omit those flags to use available integrations.

See the [usage guide](https://github.com/rhysd/actionlint/blob/main/docs/usage.md)
for additional options and the
[installation guide](https://github.com/rhysd/actionlint/blob/main/docs/install.md)
for standalone binaries that do not require Go.

## Changed-file CI checks

The [Actionlint workflow](../../.github/workflows/actionlint.yml) runs on pushes
and pull requests that change workflows, local actions, the selection script,
or Go dependency files. It checks only added or modified workflow YAML files,
including the destinations of renames; deleted files are excluded.

Pull requests use the merge base with their target branch. Pushes compare the
before and after commits; a newly created branch is compared with an empty tree.
If no workflow YAML files changed, linting is skipped, even if the tool version
changed. Run all workflows locally when updating actionlint.

Actionlint does **not** validate standalone composite `action.yml` files or
installer scripts. Changed files under `.github/actions/` are reported as not
checked, rather than passed to actionlint as workflows. This job is not a
replacement for testing local actions or linting their shell scripts.
