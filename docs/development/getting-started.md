# Getting started

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is abbreviated; versions, timings, paths, and interactive prompts
vary by installation. `...` marks omitted output.

## Development tools

### Install Git

Install **Git** to clone the repository and work with its source history. Follow
the [official Git installation instructions](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
for your operating system.

Open a new terminal and verify the installation:

```console
$ git --version
git version 2.55.0
```

### Install Rust

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

### Install cargo-nextest

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

## Additional tools (optional)

### Node.js, pnpm, and SVGO

These tools are only needed when optimizing the desktop app's SVG artwork.
They are not required to build, test, or run Chessvault: Cargo embeds the
existing SVG files directly.

- [Node.js](https://nodejs.org/en/download) runs the SVG optimizer. Install a
  supported LTS release.
- [pnpm](https://pnpm.io/installation) manages the asset-tooling dependencies.
  Follow its installation instructions for your Node.js version.
- [SVGO](https://svgo.dev/) optimizes SVG files. It is a local development
  dependency in `apps/chessvault/package.json`, not a global installation.

Verify Node.js and pnpm:

```console
$ node --version
v24.18.0
$ pnpm --version
12.5.1
```

After cloning the repository, install the locked dependencies from the workspace
root:

```console
$ cd apps/chessvault
$ pnpm install --frozen-lockfile
Lockfile is up to date, resolution step is skipped
...
$ pnpm exec svgo --help
Usage: svgo [options] [INPUT...]
...
$ cd ../..
```

The committed `pnpm-lock.yaml` keeps dependency resolution reproducible. SVGO
runs manually; it is not part of the Cargo build or CI checks. See the
[SVG optimization workflow](../components/desktop.md#optimizing-svg-artwork)
for usage and visual verification.

### Tombi

[Tombi](https://tombi-toml.github.io/tombi/) provides a TOML language server (LSP),
formatter, and linter. Use it for editor support and for formatting and checking
TOML files such as Cargo manifests and tool configuration.

Follow the official Tombi documentation for installation and editor setup.
Local installation is optional and is not required to build, test, or run
Chessvault. The TOML CI workflow uses Tombi to check formatting and lint TOML
files. Our [Tombi configuration](../../tombi.toml) sets a 120-character line
width and four-space indentation, matching [EditorConfig](../../.editorconfig).
See the [TOML checks](testing.md#toml-checks) for local usage.

### rumdl

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
[documentation checks](testing.md#documentation-checks) for usage in this
repository.

### GitHub CLI

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

## Clone the repository

Choose one of the following methods to clone the repository from GitHub.

### SSH (preferred)

We prefer SSH for authenticating with GitHub. First, follow
[GitHub's SSH setup instructions](https://docs.github.com/en/authentication/connecting-to-github-with-ssh)
to create an SSH key and add its public key to your GitHub account. Then clone:

```console
$ git clone git@github.com:chessvaultdotapp/chessvault.git
Cloning into 'chessvault'...
...
```

### HTTPS

If you do not want to configure an SSH key, clone over HTTPS instead:

```console
$ git clone https://github.com/chessvaultdotapp/chessvault.git
Cloning into 'chessvault'...
...
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
...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
```

The first build downloads and compiles dependencies, so it may take some time.
The `--locked` flag keeps dependency resolution consistent with the repository's
`Cargo.lock`.

After the build succeeds, launch the desktop application:

```console
$ cargo run -p chessvault --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running `target/debug/chessvault`
...
```

Run the app in a graphical desktop session. You should see a chessboard with the
standard starting position; playing moves is not supported yet. Press **F12**
to open the developer console.

Default debug builds store local state under `.local/state/chessvault/` relative
to the workspace root when launched from there. The app saves its window size
when closed and restores it on the next launch.
