# ShellCheck

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

[ShellCheck](https://www.shellcheck.net/) finds bugs and portability issues in
shell scripts. Local installation is optional and is not required to build, test,
or run Chessvault. The Shell CI workflow runs ShellCheck when shell scripts
change.

Install a release binary and verify the installation:

```console
$ shellcheck --version
ShellCheck - shell script analysis tool
version: 0.11.0
...
```

Follow the [ShellCheck installation instructions](https://github.com/koalaman/shellcheck#installing)
for other methods. ShellCheck detects the shell dialect from each script's
shebang, so POSIX `sh` and Bash scripts are checked with the right rules.

Suppress a deliberate pattern with an inline directive, such as
`# shellcheck disable=SC2016` above a command whose literal `$HOME` is intended
for display. Prefer a targeted directive over disabling a check repository-wide.

Lint one file or every tracked script:

<!-- rumdl-disable MD014 -->
```console
$ shellcheck scripts/install-linux.sh
$ git ls-files -z '*.sh' | xargs -0 -r shellcheck
```
<!-- rumdl-enable MD014 -->

See the [shell checks](../development/testing.md#shell-checks) for repository
usage.
