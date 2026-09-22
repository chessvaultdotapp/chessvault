---
name: Bug report
about: Report a bug, build failure, or known dependency vulnerability.
title: ''
labels: ''
assignees: ''
---

<!--
Search existing issues before submitting. Remove sections that do not apply.
Build failures are valid bug reports. For build/contribution questions or routine
refactoring and dependency maintenance proposals, use GitHub Discussions:
https://github.com/chessvaultdotapp/chessvault/discussions
For known dependency vulnerabilities, include the affected dependency/version
and a public advisory link (CVE or RustSec, if available). Do not disclose
nonpublic security details in a public issue.
-->

## What went wrong?

Describe the problem and what you expected to happen.

## Steps to reproduce

1. Describe the initial setup.
2. Describe the action that triggers the problem.
3. Describe the observed result.

## Environment

- Chessvault version or commit:
- OS and version:
- Installation method or build/run command (including any feature flags):
- For desktop issues: desktop environment, Wayland/X11, and display scaling,
  if known:

## Additional context

Include relevant logs, screenshots, or a minimal reproducer if available.
For chess-core issues, include the position and operation that trigger the problem.

<!--
Desktop logs are available in the F12 developer console. For app debug events,
launch with: RUST_LOG=chessvault=debug cargo run -p chessvault --locked
Review logs and screenshots for private information before posting.
-->
