# Ruff

Commands follow our [shell command conventions](../reference.md#shell-commands).
Run them from the workspace root. Example output is illustrative; versions and
diagnostics vary.

[Ruff](https://docs.astral.sh/ruff/) is a Python linter and formatter. It is
optional for Python tool development and is not required to build or run the
Rust application.

## Installation

Follow the [shared Python environment setup](index.md#shared-python-development-environment).
The version is pinned in
[`venvs/development.requirements.txt`](../../venvs/development.requirements.txt).

```console
$ venvs/development/bin/ruff --version
ruff 0.16.9
```

## Check Python code

Lint the release tag tool and its tests without modifying files:

```console
$ venvs/development/bin/ruff check tools/tag-tool
...
```

Check formatting separately:

```console
$ venvs/development/bin/ruff format --check tools/tag-tool
2 files already formatted
```

Both commands exit unsuccessfully when they find issues. Formatting and linting
are separate operations; passing one does not imply that the other passes.

## Apply changes

Apply available safe lint fixes and format the files:

```console
$ venvs/development/bin/ruff check --fix tools/tag-tool
...
$ venvs/development/bin/ruff format tools/tag-tool
...
```

These commands modify files. Review the diff, address remaining diagnostics,
and rerun checks and the [tag tool tests](tag-tool.md#tests). Ruff does not replace
[type checking with ty](ty.md) or runtime tests.

See the official [linter](https://docs.astral.sh/ruff/linter/),
[formatter](https://docs.astral.sh/ruff/formatter/), and
[editor integration](https://docs.astral.sh/ruff/editors/) guides for details.
