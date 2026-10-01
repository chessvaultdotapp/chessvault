# ty

Commands follow our [shell command conventions](../reference.md#shell-commands).
Run them from the workspace root. Example output is illustrative; versions and
diagnostics vary.

[ty](https://docs.astral.sh/ty/) is a Python type checker and language server.
It checks annotated code for type inconsistencies without executing it. It is
optional for Python tool development and is not required to build or run the
Rust application.

## Installation

Follow the [shared Python environment setup](index.md#shared-python-development-environment).
The version is pinned in
[`venvs/development.requirements.txt`](../../venvs/development.requirements.txt).

```console
$ venvs/development/bin/ty --version
ty 0.0.84
```

## Check Python code

First install the [tag tool's dependencies](tag-tool.md#setup), including pytest.
Then check its implementation and tests:

```console
$ venvs/development/bin/ty check --python venvs/tag-tool tools/tag-tool
...
```

The checker runs from the shared development environment, while `--python`
selects the environment whose interpreter and dependencies describe the code
being checked. Pointing it at `venvs/tag-tool` lets it resolve `tomli` and `pytest`
without duplicating those dependencies in the development environment. The
option accepts an environment directory or a Python interpreter path.

The command reports type diagnostics without modifying files and exits
unsuccessfully for errors. Fix the reported issues and rerun it; an unresolved
import may mean the selected environment is missing a dependency rather than
that the source import is incorrect.

Type checking complements [Ruff linting and formatting](ruff.md) and the
[tag tool tests](tag-tool.md#tests); it does not replace either.

See the official [environment configuration](https://docs.astral.sh/ty/modules/)
and [editor integration](https://docs.astral.sh/ty/editors/) guides for details.
