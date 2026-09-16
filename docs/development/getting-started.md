# Getting started

Commands follow our [shell command conventions](../reference.md#shell-commands).

## Development tools

### Install Git

Install **Git** to clone the repository and work with its source history. Follow
the [official Git installation instructions](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
for your operating system.

Open a new terminal and verify the installation:

```console
$ git --version
```

### Install Rust

Follow the [official Rust installation instructions](https://www.rust-lang.org/tools/install)
to install Rust through **rustup**. Rustup manages Rust toolchains and includes
Cargo, Rust's build tool and package manager.

After installation, open a new terminal and verify that the tools are available:

```console
$ rustup --version
$ cargo --version
```

Chessvault pins its Rust toolchain in
[`rust-toolchain.toml`](../../rust-toolchain.toml). When you run Cargo from the
repository, rustup selects that toolchain automatically and installs it if
needed. You do not need to change your global default toolchain.

### Install cargo-nextest

**cargo-nextest** is our preferred test runner and is used in CI. Our preferred
installation method is through Cargo:

```console
$ cargo install cargo-nextest --locked
$ cargo nextest --version
```

See the [official cargo-nextest documentation](https://nexte.st/docs/installation/)
for installation details and alternative methods. Nextest does not run doctests; those are
run separately with `cargo test --doc`.

## Clone the repository

Choose one of the following methods to clone the repository from GitHub.

### SSH (preferred)

We prefer SSH for authenticating with GitHub. First, follow
[GitHub's SSH setup instructions](https://docs.github.com/en/authentication/connecting-to-github-with-ssh)
to create an SSH key and add its public key to your GitHub account. Then clone:

```console
$ git clone git@github.com:chessvaultdotapp/chessvault.git
```

### HTTPS

If you do not want to configure an SSH key, clone over HTTPS instead:

```console
$ git clone https://github.com/chessvaultdotapp/chessvault.git
```

### Enter the workspace

After cloning with either method, enter the repository directory:

```console
$ cd chessvault
```

Run subsequent Cargo commands from this workspace root.

## Build and run

Build all workspace packages and targets:

```console
$ cargo build --workspace --all-targets --locked
```

The first build downloads and compiles dependencies, so it may take some time.
The `--locked` flag keeps dependency resolution consistent with the repository's
`Cargo.lock`.

After the build succeeds, launch the desktop application:

```console
$ cargo run -p chessvault --locked
```

Run the app in a graphical desktop session. You should see a chessboard with the
standard starting position; playing moves is not supported yet. Press **F12**
to open the developer console.

Default debug builds store local state under `.local/state/chessvault/` relative
to the workspace root when launched from there. The app saves its window size
when closed and restores it on the next launch.
