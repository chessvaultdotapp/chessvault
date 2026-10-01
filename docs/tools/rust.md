# Rust

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

Follow the [official Rust installation instructions](https://www.rust-lang.org/tools/install)
to install Rust through **rustup**. Rustup manages Rust toolchains and includes
Cargo, Rust's build tool and package manager.

After installation, open a new terminal and verify that the tools are available:

```console
$ rustup --version
rustup 1.29.1 (d95a37b6a 2026-08-13)
...
$ cargo --version
cargo 1.98.0 (797e8a9bc 2026-08-05)
```

Chessvault pins its Rust toolchain in
[`rust-toolchain.toml`](../../rust-toolchain.toml). When you run Cargo from the
repository, rustup selects that toolchain automatically and installs it if
needed. You do not need to change your global default toolchain.
