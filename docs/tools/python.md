# Python

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

[Python](https://www.python.org/) **3.14.5 or newer** is optional tooling for
Python-based documentation workflows. It is not required to build, test, or run
Chessvault.

You can install Python through [uv](uv.md), without installing it separately from
your operating system's package manager:

```console
$ uv python install 3.14.5
...
$ uv run --no-project --python 3.14.5 python --version
Python 3.14.5
```

The explicit version selects uv's managed interpreter rather than relying on
which `python` executable is on your `PATH`. To use a newer version, substitute
that version in both commands. See uv's
[Python installation guide](https://docs.astral.sh/uv/guides/install-python/)
for details, or use the [official Python downloads](https://www.python.org/downloads/)
if you prefer a separate installation.
