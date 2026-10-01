# uv

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

[uv](https://docs.astral.sh/uv/) manages Python versions, project dependencies,
virtual environments, and Python-based command-line tools. It can support
Python-based documentation tooling; it is not required to build, test, or run
Chessvault.

Follow the [official installation instructions](https://docs.astral.sh/uv/getting-started/installation/)
for your operating system. The standalone installer does not require an existing
Python installation. Open a new terminal after installation and verify:

```console
$ uv --version
uv 0.11.16 (x86_64-unknown-linux-gnu)
```

Use `uv tool run <tool>` to run a Python tool in an isolated environment, or
`uv tool install <tool>` to make its commands available persistently. See the
[tools guide](https://docs.astral.sh/uv/guides/tools/) for details.

For this repository, use the [Python environment recipes](just.md#python-environments)
in `venvs/justfile` to install the shared development tools, release tag tool,
or Zensical documentation dependencies from their pinned requirements files.
