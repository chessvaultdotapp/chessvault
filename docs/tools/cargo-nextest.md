# cargo-nextest

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

**cargo-nextest** is our preferred test runner and is used in CI. Our preferred
installation method is through Cargo:

```console
$ cargo install cargo-nextest --locked
    Updating crates.io index
...
$ cargo nextest --version
cargo-nextest 0.9.143 (60fa45f63 2026-08-04)
...
```

See the [official cargo-nextest documentation](https://nexte.st/docs/installation/)
for installation details and alternative methods. Nextest does not run doctests; those are
run separately with `cargo test --doc`.
