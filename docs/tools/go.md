# Go

Commands follow our [shell command conventions](../reference.md#shell-commands).
Example output is illustrative; versions and paths vary by installation.

[Go](https://go.dev/) can install and run Go-based development tools such as
[actionlint](actionlint.md). It is not required to build, test, or run Chessvault.

Follow the [official installation instructions](https://go.dev/doc/install)
for your operating system. The root [`go.mod`](../../go.mod) declares the Go
version used for repository tooling; CI reads that version too. Open a new
terminal and verify:

```console
$ go version
go version go1.26.3 linux/amd64
```
