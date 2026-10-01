# Tombi

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

[Tombi](https://tombi-toml.github.io/tombi/) provides a TOML language server (LSP),
formatter, and linter. Use it for editor support and for formatting and checking
TOML files such as Cargo manifests and tool configuration.

Follow the official Tombi documentation for installation and editor setup.
Local installation is optional and is not required to build, test, or run
Chessvault. The TOML CI workflow uses Tombi to check formatting and lint TOML
files. Our [Tombi configuration](../../tombi.toml) sets a 120-character line
width and four-space indentation, matching [EditorConfig](../../.editorconfig).
See the [TOML checks](../development/testing.md#toml-checks) for local usage.
