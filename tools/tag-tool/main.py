"""Update the desktop manifest version and create a local Git tag at HEAD.

Manifest changes are not committed, and tags are not pushed.
"""

import argparse
import re
import subprocess
from datetime import UTC, datetime
from pathlib import Path

import tomli

CHESSVAULT_ROOT = Path(__file__).absolute().parents[2]
DESKTOP_APP_ROOT = CHESSVAULT_ROOT / "apps" / "chessvault"

SEMVER_TAG = re.compile(
    r"v(?P<version>"
    r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"
    r"(?:-(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)"
    r"(?:\.(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*))*)?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
    r")"
)


def semver_tag(value: str) -> str:
    """Validate a v-prefixed SemVer 2.0.0 tag and return its version without v."""
    match = SEMVER_TAG.fullmatch(value)
    if match is None:
        raise argparse.ArgumentTypeError(
            "expected a v-prefixed SemVer tag, e.g. v1.0.0"
        )
    return match.group("version")


def next_suffix(suffix: str) -> str:
    """Increment a lowercase ASCII suffix: empty -> a, z -> aa, aa -> ab."""
    if not suffix:
        return "a"
    letters = list(suffix)
    for index in range(len(letters) - 1, -1, -1):
        if letters[index] != "z":
            letters[index] = chr(ord(letters[index]) + 1)
            return "".join(letters)
        letters[index] = "a"
    return "a" + "".join(letters)


def update_version(manifest_path: Path, version: str) -> str:
    """Write a validated version into [package], returning the previous version.

    Preserve unrelated content, comments, and line endings. Parse and verify the
    replacement before writing so unsupported manifest layouts fail unchanged.
    """
    original = manifest_path.read_bytes().decode("utf-8")
    desktop = tomli.loads(original)
    previous_version = desktop["package"]["version"]

    package = re.search(
        r"(?m)^\[package\][^\r\n]*(?:\r?\n|$)(?P<body>(?:(?!^\[).)*)",
        original,
        re.DOTALL,
    )
    if package is None:
        raise ValueError("could not locate [package] section")
    body, count = re.subn(
        r"(?m)^(\s*version\s*=\s*)[\"'][^\"'\r\n]*[\"']",
        lambda match: f'{match.group(1)}"{version}"',
        package.group("body"),
    )
    if count != 1:
        raise ValueError("expected exactly one package version")
    updated = original[: package.start("body")] + body + original[package.end("body") :]
    expected = tomli.loads(original)
    expected["package"]["version"] = version
    if tomli.loads(updated) != expected:
        raise ValueError("manifest update changed unexpected fields")
    manifest_path.write_bytes(updated.encode("utf-8"))
    return previous_version


def main() -> None:
    """Update the manifest from an explicit tag or development metadata, then tag HEAD.

    Development releases append UTC ISO week-year/week metadata with an alphabetic
    suffix, or increment the suffix of existing trailing week metadata.
    Git failures propagate; the preceding manifest update is not rolled back.
    """
    parser = argparse.ArgumentParser(
        description="Set the desktop version from a SemVer tag."
    )
    parser.add_argument(
        "tag", nargs="?", type=semver_tag, help="v-prefixed version, e.g. v1.0.0"
    )
    parser.add_argument(
        "--dev-rel",
        action="store_true",
        help="append development metadata and update Cargo.toml",
    )
    args = parser.parse_args()
    if args.dev_rel and args.tag is not None:
        parser.error("--dev-rel cannot be combined with a version tag")
    if not args.dev_rel and args.tag is None:
        parser.error("provide a version tag or --dev-rel")

    manifest_path = DESKTOP_APP_ROOT / "Cargo.toml"
    if args.dev_rel:
        with manifest_path.open("rb") as manifest:
            version = tomli.load(manifest)["package"]["version"]
        semver_tag(f"v{version}")
        year, week, _ = datetime.now(tz=UTC).date().isocalendar()
        week_metadata = f"{year % 100:02d}w{week:02d}"
        # Reuse an existing week marker; only new metadata uses today's ISO week.
        existing_week = re.search(
            r"(?P<prefix>[+.][0-9]{2}w[0-9]{2})(?P<suffix>[a-z]*)$", version
        )
        if existing_week:
            development_version = version[
                : existing_week.start("suffix")
            ] + next_suffix(existing_week.group("suffix"))
        else:
            separator = "." if "+" in version else "+"
            development_version = f"{version}{separator}{week_metadata}a"
        target_version = development_version
    else:
        target_version = args.tag

    previous_version = update_version(manifest_path, target_version)
    print(f"Desktop version: {previous_version} -> {target_version}")
    tag = f"v{target_version}"
    # Tag the current commit, not the uncommitted manifest contents.
    subprocess.run(["git", "tag", tag], cwd=CHESSVAULT_ROOT, check=True)
    print(f"Created tag: {tag}")


if __name__ == "__main__":
    main()
