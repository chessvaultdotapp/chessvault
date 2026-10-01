# GitHub CLI

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

[GitHub CLI](https://cli.github.com/) (`gh`) lets you manage pull requests,
issues, and GitHub Actions runs from your terminal. It is useful for contributing,
but is not required to build or run Chessvault.

Follow the [official installation instructions](https://github.com/cli/cli#installation),
then verify the installation and sign in to GitHub:

```console
$ gh --version
gh version 2.98.0 (2026-08-21)
https://github.com/cli/cli/releases/tag/v2.98.0
$ gh auth login
? Where do you use GitHub?  [Use arrows to move, type to filter]
> GitHub.com
  Other
...
```

See the [GitHub CLI manual](https://cli.github.com/manual/) for available commands
and authentication options.
