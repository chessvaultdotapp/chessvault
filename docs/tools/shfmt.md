# shfmt

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

[shfmt](https://github.com/mvdan/sh) formats shell scripts. Use it to keep
`*.sh` files consistently indented and spaced. Local installation is optional and
is not required to build, test, or run Chessvault. The Shell CI workflow runs
shfmt when shell scripts or the shared [EditorConfig](../../.editorconfig) change.

Install a release binary and verify the installation:

```console
$ shfmt --version
v3.14.1
```

Follow the [shfmt installation instructions](https://github.com/mvdan/sh#shfmt)
for other methods. shfmt reads our `.editorconfig`: `*.sh` files use four-space
indentation, indented `case` labels, and a space after redirections. The shell
dialect is detected from each script's shebang.

Format files in place, or check them without modifying anything:

<!-- rumdl-disable MD014 -->
```console
$ shfmt -w scripts/install-linux.sh
$ shfmt -d scripts/install-linux.sh
```
<!-- rumdl-enable MD014 -->

See the [shell checks](../development/testing.md#shell-checks) for repository
usage.
