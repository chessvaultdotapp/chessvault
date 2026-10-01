# rumdl

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

[rumdl](https://github.com/rvben/rumdl) checks Markdown documentation for lint
issues. Local installation is optional and is not required to build or run
Chessvault. The Markdown CI workflow runs rumdl when Markdown files change.

Install it through Cargo and verify the installation:

```console
$ cargo install rumdl --locked
    Updating crates.io index
...
$ rumdl --version
rumdl 0.2.73
```

See the [official rumdl installation guide](https://rumdl.dev/getting-started/installation/) for
installation details and alternative methods and the
[documentation checks](../development/testing.md#documentation-checks) for usage in this
repository.
