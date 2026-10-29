# Testing

Commands follow our [shell command conventions](../reference.md#shell-commands).
Run them from the workspace root using the toolchain selected by
[`rust-toolchain.toml`](../../rust-toolchain.toml). See
[cargo-nextest](../tools/cargo-nextest.md) for runner installation.

Output examples below are abbreviated and illustrative, not verification results.
Timings, test counts, and build messages vary; `...` marks omitted output.
`cargo fmt --all -- --check` produces no output when formatting passes.

## Ordered checks

With the optional [just task runner](../tools/just.md), `just check` runs
this sequence. Use `just check-package <package>` for focused checks or
`just check-platform-native` for the additional no-default-features checks.
These recipes require cargo-nextest; the manual fallback is described below.

Run these checks in order, stopping if a command fails. Fix the failure before
continuing:

```console
$ cargo build --workspace --all-targets --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
$ cargo nextest run --workspace --locked
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
...
$ cargo test --workspace --doc --locked
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
...
$ cargo fmt --all -- --check
$ cargo clippy --workspace --all-targets --locked --no-deps
...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
```

Check workflows run on pushes to `main` and on pull requests, subject to their existing path filters.
Feature-branch pushes without a pull request and tag pushes do not run these checks.
Release workflows handle tags separately; documentation deployment remains limited to `main`.

This is the sequence used by [CI](../../.github/workflows/ci.yml). Eligible pushes and pull
requests trigger it only when Rust source, desktop assets, Cargo manifests or
lockfiles, `rust-toolchain.toml`, `.cargo` configuration, differential-test inputs,
or the root/venv recipes change. Markdown-only changes do not trigger Rust builds. Nextest is our
preferred test runner, but it does not execute doctests, so the separate
`cargo test --doc` step is required. Formatting checks do not modify files.
Clippy's `--no-deps` excludes dependency linting, though dependencies may still
need to be compiled or checked.

If nextest is unavailable, replace both test commands with
`cargo test --workspace --locked`, which runs unit tests, integration tests, and
doctests. Keep the build, formatting, and Clippy steps in the same order.

## CI cache measurement

The debug CI cache uses separate restore and save steps. Pull requests restore the
main-branch cache but do not save; successful pushes to `main` save on a primary-key
miss. CI disables incremental compilation and excludes incremental directories from
the archive. Local Cargo defaults and the installer release cache are unchanged.
The `cargo-v2` namespace deliberately starts cold rather than restoring the old policy.

Baseline: [CI run 40065880898](https://github.com/chessvaultdotapp/chessvault/actions/runs/40065880898)
on October 24, 2026 restored **2,038,223,963 compressed bytes** in **38 seconds**,
including about 19 seconds extracting the archive. Saving **2,108,702,889 bytes**
took **29 seconds**, including about 20 seconds creating/compressing the archive.
The build step took **8 seconds** and the whole job approximately **90 seconds**.
Across the audit's 25 CI runs, restore and save consumed 24.1 of 38.3 job minutes.
These are observed timings, not a controlled benchmark or a claim of future savings.

After rollout, compare the following before considering further cache pruning:

- The first cold `cargo-v2` build versus subsequent warm main and PR builds.
- Restore and save durations and compressed bytes in the cache action logs.
- Build, tests, Clippy, and total job durations; disabling incremental compilation
  may increase workspace compilation time even as transfers shrink.
- The job summary's uncompressed cache footprint, including `target/debug/deps`,
  `target/debug/build`, and `target/debug/incremental` when present. Parent and child
  sizes overlap and must not be summed.
- Active cache storage and eviction behavior after several main pushes. Per-commit
  main caches still accumulate; this change does not impose a storage cap.

Use similar changes and the same toolchain when comparing warm runs. Do not treat
cold-start costs or ordinary runner variance as steady-state results. The next
hosted runs are required to establish actual savings; local `target` sizes are not
representative of the CI archive.

## Focused checks

For package-local changes, replace `--workspace` in the build, test, and Clippy
commands with one of:

- `-p chessvault` — desktop application.
- `-p chess-core` — chess primitives and positions.
- `-p platform-dirs` — platform directory resolution.

Keep `cargo fmt --all -- --check` workspace-wide. Use workspace checks for shared
configuration, cross-package changes, and API changes affecting consumers.

For example, the core-only sequence is:

```console
$ cargo build -p chess-core --all-targets --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
$ cargo nextest run -p chess-core --locked
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
...
$ cargo test -p chess-core --doc --locked
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
...
$ cargo fmt --all -- --check
$ cargo clippy -p chess-core --all-targets --locked --no-deps
...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
```

### A single test

After a successful build, use nextest's filter expression to select a test by
its exact name:

```console
$ cargo nextest run -p chess-core --locked -E 'test(=side::tests::side_uses_one_byte)'
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
...
```

The equivalent Cargo test command explicitly selects the library target:

```console
$ cargo test -p chess-core --lib --locked side::tests::side_uses_one_byte -- --exact
...
running 1 test
test side::tests::side_uses_one_byte ... ok
...
```

The desktop is binary-only. For example, to test saved window dimensions:

```console
$ cargo test -p chessvault --bin chessvault --locked window_recreate_tests::saved_dimensions_round_trip -- --exact
...
running 1 test
test window_recreate_tests::saved_dimensions_round_trip ... ok
...
```

Focused tests shorten iteration; they do not replace the relevant full check
sequence before submitting changes. See the
[nextest filtering documentation](https://nexte.st/docs/filtersets/) for more
selection options.

## Test organization and isolation

Tests currently live in inline `#[cfg(test)]` modules beside implementations.
Use these nearby tests as examples when extending a component:

- **Chess core:** pin compact representations and discriminants, bitboard
  operations, position construction, and square lookup. Include sentinel cases;
  empty pieces, sides, and squares are not valid occupancy indices.
- **Desktop:** test log capture and filtering, window-state binary-format validation, and
  file-path composition independently of the GUI.
- **Platform directories:** inject upstream path results,
  environment values, and home-directory lookups. Do not mutate process-wide
  environment variables in tests. Preserve non-Unicode path coverage where
  supported.
- **Logging:** use scoped subscribers via `tracing::subscriber::with_default`,
  not `Logs::init()`, which installs the application's global subscriber.

Put executable public API examples in Rustdoc when they should be covered by
doctests. Code blocks in Markdown guides are not automatically tested by Cargo.

## Platform and feature variants

The default-enabled `platform-dirs` development feature makes debug builds use
working-directory-local state and share paths. Native resolution takes a different
compiled branch, so default debug checks alone do not cover every configuration.

For changes to that policy, also run:

```console
$ cargo build -p platform-dirs --all-targets --locked --no-default-features
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
$ cargo nextest run -p platform-dirs --locked --no-default-features
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
...
$ cargo test -p platform-dirs --doc --locked --no-default-features
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
...
$ cargo fmt --all -- --check
$ cargo clippy -p platform-dirs --all-targets --locked --no-deps --no-default-features
...
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
```

Stop on failure here too. Linux resolver tests are compiled only on Linux and
use the `linux::tests::` prefix. Passing tests on another OS does not verify
Linux resolution. Native directory resolution supports Linux and macOS;
the development path working elsewhere is not evidence of release app
support. See [Platform directories](../components/platform-dirs.md) for the path policy.

### Differential platform-directory tests

On Linux or macOS, compare the native public APIs against pinned Python `platformdirs`:

```sh
just --justfile venvs/justfile platform-dirs-differential
just check-platform-differential
```

Setup requires uv; the comparison requires the pinned Rust toolchain and the generated
Python environment. Without just, run
`venvs/platform-dirs-differential/bin/python -m pytest tools/platform-dirs-differential -q`.
The session builds `differential_probe` once with `--locked --no-default-features` and
uses Cargo's reported executable path (including custom target directories). The probe
rejects builds with the development feature. Ordinary Rust tests need no Python installation.

Each case launches Rust and Python in separate child processes with equivalent controlled
XDG variables and an explicit home directory; the test process environment is never mutated.
The reference uses `Unix` on Linux and `MacOS` on macOS with `appauthor=False`, `version=None`, `multipath=True`,
`ensure_exists=False`, and `use_site_for_root=False`, including when tests run as root.
It compares state with user state, share with user data, and system share with the complete
ordered site-data list. All three Rust log opinions are compared against the corresponding
Python opinion. Failures report the case inputs, both outputs, and reference version;
the installed version must match the requirements pin.

macOS cases verify Application Support data/state paths, Library/Logs for every log opinion,
and that XDG variables are ignored. The suite distinguishes parity from these explicit policy differences
(the XDG differences apply only to Linux):

- An explicitly configured system list containing no absolute entries produces an empty Rust
  list; Python falls back to the default system directories. Both results are asserted.
- Rust preserves whitespace in native paths; Python strips surrounding XDG whitespace.
- Rust rejects empty application names, dot components, separators, colons, and NUL.
  Python rejects parent traversal but accepts the other tested names. Error text is not a contract.
- Rust rejects a relative home fallback; Python can return relative paths.

Relative user XDG paths and mixed absolute/relative system lists are parity cases with this
reference version, not skipped exceptions. Missing/invalid injected home lookups, error precedence,
non-Unicode byte preservation, and development-local paths remain covered by Rust-only tests;
the JSON probe intentionally handles Unicode paths only. Real unset-home lookup depends on the
OS account database, so deterministic fallback failures use injected resolver tests instead.

The Linux and macOS differential jobs in `ci.yml` run on Rust/native inputs and changes to the runner,
pin, or recipes. Python CI separately lints and type-checks the runner. When updating the pin,
review changed reference behavior rather than normalizing away new mismatches.

## Documentation checks

Use [rumdl](../tools/rumdl.md) for optional local Markdown linting,
separate from the ordered Cargo checks. From the workspace root, check the
shared documentation:

```console
$ rumdl check README.md CONTRIBUTING.md AGENTS.md docs
Success: No issues found in 14 files (10ms)
```

For a focused check, pass only the files you changed:

```console
$ rumdl check docs/development/getting-started.md docs/development/testing.md
Success: No issues found in 2 files (10ms)
```

`rumdl check` reports issues without modifying files and exits unsuccessfully
when lint issues are found. The [Markdown workflow](../../.github/workflows/markdown.yml)
runs on pushes and pull requests that change `*.md` files, including nested and
hidden directories. The shared [setup-rumdl action](https://github.com/chessvaultdotapp/actions-setup-rumdl)
installs the official rumdl 0.2.75 release binary, following the
[upstream binary installation method](https://rumdl.dev/getting-started/installation/#download-binary).
The workflow checks all tracked Markdown files, not just changed files. Configuration-only or workflow-only changes do
not trigger it; run the checks locally when changing those files. The repository's
[rumdl configuration](../../.rumdl.toml) sets the line-length limit to 120 characters.
Preserve our [`$` shell-prompt convention](../reference.md#shell-commands) and
include representative command output where applicable. Commands that are silent
on success do not need invented output.

## Website checks

Website development and checks are maintained in the separate
[website repository](https://github.com/chessvaultdotapp/website). Follow its README
and agent instructions; website checks are not part of this repository's `just check`.

## Python checks

Install the [shared Python development environment](../tools/index.md#shared-python-development-environment)
and the test environments, then run the same lint, type-check, and unit-test steps as CI:

```sh
just --justfile venvs/justfile development tag-tool dist-prepare platform-dirs-differential
```

The differential runner has its own reference dependency, so type-check it using
its dedicated environment rather than the tag-tool environment.

```console
$ git ls-files -z '*.py' '*.pyi' | xargs -0 -r venvs/development/bin/ruff check
All checks passed!
$ git ls-files -z '*.py' '*.pyi' | xargs -0 -r venvs/development/bin/ruff format --check
2 files already formatted
$ git ls-files -z '*.py' '*.pyi' | xargs -0 -r venvs/development/bin/ty check \
    --python venvs/tag-tool --force-exclude --exclude tools/platform-dirs-differential
All checks passed!
$ venvs/development/bin/ty check --python venvs/platform-dirs-differential tools/platform-dirs-differential
All checks passed!
$ venvs/tag-tool/bin/python -m pytest .github/scripts/test_website_pin.py -q
...
$ venvs/tag-tool/bin/python -m pytest tools/tag-tool -q
...
$ venvs/tag-tool/bin/python -m pytest tools/changelog-tool -q
...
$ venvs/dist-prepare/bin/python -m pytest tools/dist-prepare -q
...
```

These commands mirror the [Python workflow](../../.github/workflows/python.yml),
which runs on eligible pushes and pull requests changing `.py` or `.pyi` files,
`ruff.toml`, `venvs/platform-dirs-differential.requirements.txt`, `venvs/justfile`,
or the Python workflow itself. Other requirements/configuration changes do not
trigger it; run these checks locally when changing those files.
Pass new, untracked Python files explicitly to Ruff and ty before staging them.
See [Ruff](../tools/ruff.md) and [ty](../tools/ty.md) for usage details.

## TOML checks

Use [Tombi](../tools/tombi.md) to check formatting and lint all tracked
TOML files from the workspace root, including nested and hidden files:

```console
$ git ls-files -z '*.toml' | xargs -0 -r tombi format --check
9 files did not need formatting
$ git ls-files -z '*.toml' | xargs -0 -r tombi lint --error-on-warnings
9 files linted successfully
```

Pass individual paths to check new files before they are tracked. To apply
formatting, run `tombi format` with the paths you want to update. The shared
[configuration](../../tombi.toml) uses a 120-character line width and four-space
indentation. Lint warnings fail the check as well as errors.

The [TOML workflow](../../.github/workflows/toml.yml) mirrors the Markdown
workflow: pushes and pull requests changing `*.toml` files trigger checks of
all tracked TOML files. Workflow-only or installer-only changes do not trigger
it; run checks locally when changing those files.

The local [`setup-tombi` action](../../.github/actions/setup-tombi/action.yml)
downloads official binaries from [Tombi GitHub Releases](https://github.com/tombi-toml/tombi/releases).
It defaults to version `1.5.5`; its optional `version` input accepts an exact
release version without the `v` prefix. It supports Linux, macOS, and Windows
on x64 and ARM64, using Bash (Git Bash on Windows).

## Justfile checks

Use [just](../tools/just.md) version `1.58.0` (the shared setup action's default)
from the workspace root to check all tracked justfiles:

<!-- rumdl-disable MD014 -->
```console
$ while IFS= read -r -d '' file; do
    just --fmt --check --justfile "$file" || break
    just --list --justfile "$file" || break
  done < <(git ls-files -z -- 'justfile' '**/justfile' '.justfile' '**/.justfile' '*.just')
```
<!-- rumdl-enable MD014 -->

Run this loop in Bash. Formatting checks do not modify files; apply formatting
with `just --fmt --justfile <file>`. Pass new, untracked files explicitly to both
checks before staging them. `--list` validates parsing and lists recipes without
executing them; it does not verify that recipe commands work.

The [Just workflow](../../.github/workflows/just.yml) checks all tracked justfiles
on pushes and pull requests changing `justfile`, `.justfile`, or `*.just` files
at any depth, the workflow itself, or the shared
[`setup-just` action](../../.github/actions/setup-just/action.yml).

Just has no first-party recipe linter. ShellCheck cannot directly parse justfiles:
extracting recipe bodies would need to account for Just interpolation, per-line
shell execution, and recipes using other interpreters. A separate community
linter would add another tool dependency without replacing these checks. For now,
formatting and parse/list validation are the scope of this workflow; keep complex
shell logic in standalone scripts covered by the Shell checks below.

## Shell checks

Use [shfmt](../tools/shfmt.md) and [ShellCheck](../tools/shellcheck.md) to format
and lint tracked shell scripts from the workspace root:

<!-- rumdl-disable MD014 -->
```console
$ git ls-files -z '*.sh' | xargs -0 -r shfmt -d
$ git ls-files -z '*.sh' | xargs -0 -r shellcheck
```
<!-- rumdl-enable MD014 -->

`shfmt -d` prints formatting differences without modifying files. Run `shfmt -w`
with the paths you changed to apply formatting. shfmt reads the shared
[EditorConfig](../../.editorconfig), so `*.sh` files use four-space indentation,
indented `case` labels, and a space after redirections.

The [Shell workflow](../../.github/workflows/shell.yml) mirrors these commands. It
runs on pushes and pull requests that change `*.sh` files, `.editorconfig`, or the
workflow itself, and checks all tracked shell scripts rather than only changed
ones.

### Linux installer tests

The [Linux installer workflow](../../.github/workflows/install-script.yml) checks shell syntax
and runs pytest integration tests when the installer, tests, packaging tool, desktop entry
or SVG icon, test environment, or workflow dependencies change. Two jobs start in parallel:

- Fixture tests use a fake package and require no Rust build or release download.
- The package build compiles the current checkout in release mode on Ubuntu 22.04,
  stages it with `tools/dist-prepare/main.py`, and uploads a tar archive. This matches
  the native development-release layout and preserves executable permissions.

A third job waits for the package build, downloads the archive on a fresh Ubuntu runner,
and runs the packaged installer. It does not depend on the fixture job.
Both stable and development release workflows also run the package smoke tests
inside their combined build/packaging job, using the exact archive that will be uploaded.
A failure blocks the native artifact upload and release publication. These tests
reuse the release-produced binary; they do not compile another binary or start
another runner job. The same job then packages and validates the AppImage from
the already-built executable. Publication waits for the combined job to succeed.

The shared local action at `.github/actions/test-installer-package` installs the
Python test environment and runs `scripts/package-tests` against its required
`package` input. The standalone installer workflow uses the same action after
downloading its PR/main package, retaining independent validation for installer changes.

All installer tests require Linux and use isolated home/data directories. Optional
cache tools are stubbed so tests do not touch host caches.

Create the dedicated environment and run the fixture tests locally:

```console
$ just --justfile venvs/justfile install-script
...
$ venvs/install-script/bin/python -m pytest scripts/tests -q
...
39 passed in 0.54s
```

Coverage includes installed contents and permissions, desktop-entry escaping, XDG paths,
dry runs, interactive custom-prefix confirmation, rejected system prefixes and invalid
arguments, missing payloads, overwrite protection (including dangling symlinks), and
nonfatal cache-refresh failures.

To smoke-test a native package archive locally, use the same environment:

```console
$ CHESSVAULT_PACKAGE=/absolute/path/to/chessvault-linux-x86_64.tar.gz \
    venvs/install-script/bin/python -m pytest scripts/package-tests -q
...
2 passed in 0.51s
```

Use an archive built from the current checkout: tests compare the shipped installer,
desktop template, and SVG with repository sources. Missing archives fail rather than skip.
Smoke tests extract the archive, check the ELF binary and PNG signatures, execute the
installer directly, and verify installed contents, permissions, launcher paths, dry runs,
and overwrite refusal with both default and custom XDG data directories. These tests
verify packaging and installation, not GUI startup.

## Desktop interaction checks

Automated tests do not replace launching the app for visible desktop changes:

```console
$ cargo run -p chessvault --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running `target/debug/chessvault`
...
```

Exercise the affected interaction in a graphical desktop session. Depending on
the change, verify legal moves and special moves, promotion choice/cancellation,
read-only history navigation, results and draw claims/offers, new-game confirmation,
and PGN saving/cancellation/failures. Also check affected board resizing, console
filtering/copying during refresh, and window persistence. Use the detailed
[desktop checklist](https://chessvaultdotapp.github.io/chessvault/dev/docs/apps/chessvault/development/#verification);
exercise the changed
behavior rather than treating a successful launch as sufficient.

Include a brief description of what changed and how it was verified in the pull
request. Screenshots are useful for UI changes. Report any checks you could not
run and why, rather than treating them as passed.
