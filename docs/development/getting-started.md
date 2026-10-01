# Getting started

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is abbreviated; versions, timings, paths, and interactive prompts
vary by installation. `...` marks omitted output.

## Development tools

Install these tools before starting:

- [Git](../tools/git.md) to clone the repository and manage source history.
- [Rust and Cargo](../tools/rust.md) through rustup. The repository selects its
  pinned Rust toolchain automatically.
- [cargo-nextest](../tools/cargo-nextest.md) for the preferred test workflow.
  If unavailable, use the Cargo test fallback in the [testing guide](testing.md#ordered-checks).

See [Development tools](../tools/index.md) for installation details and optional
tools for task shortcuts, documentation, artwork, and CI maintenance.

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
standard starting position. Press **F12** to open the developer console.

Default debug builds store local state under `.local/state/chessvault/` relative
to the workspace root when launched from there. The app saves its window size
when closed and restores it on the next launch.

## Verify changes

Follow the [testing guide](testing.md) for ordered checks, focused package tests,
and desktop interaction verification. With [just](../tools/just.md) installed,
`just check` runs the workspace verification sequence.
