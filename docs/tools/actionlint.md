# actionlint

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

[actionlint](https://github.com/rhysd/actionlint) checks GitHub Actions workflows
for syntax errors, invalid expressions, and other common mistakes. Use it when
editing files in `.github/workflows/`. It is optional and is not currently run
by the repository's CI workflows.

Install it with [Go](go.md):

<!-- rumdl-disable MD014 -->
```console
$ go install github.com/rhysd/actionlint/cmd/actionlint@v1.7.7
```
<!-- rumdl-enable MD014 -->

Add Go's binary installation directory to your `PATH`: `GOBIN` if configured,
otherwise `bin` beneath `GOPATH` (normally `$HOME/go/bin`). Verify the installation:

```console
$ actionlint -version
v1.7.7
...
```

From the workspace root, check all workflows with `actionlint`, or pass specific
files to check only those workflows:

<!-- rumdl-disable MD014 -->
```console
$ actionlint .github/workflows/docs-platform-dirs.yml
```
<!-- rumdl-enable MD014 -->

Successful checks produce no output. If available, actionlint also invokes
ShellCheck for embedded shell scripts; use `-shellcheck=` to explicitly disable
that integration. See the
[installation guide](https://github.com/rhysd/actionlint/blob/main/docs/install.md)
for prebuilt binaries that do not require Go, and the
[usage guide](https://github.com/rhysd/actionlint/blob/main/docs/usage.md)
for additional options.
