"""Assemble reviewable release drafts from committed development changelogs."""

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).absolute().parents[2]
NUMBER = r"(?:0|[1-9][0-9]*)"
STABLE = re.compile(rf"v{NUMBER}\.{NUMBER}\.{NUMBER}")
DEV = re.compile(
    rf"v{NUMBER}\.{NUMBER}\.{NUMBER}"
    r"(?:-(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)"
    r"(?:\.(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*))*)?"
    r"\+(?:[0-9A-Za-z-]+\.)*[0-9]{2}w[0-9]{2}[a-z]+"
)


def git(root: Path, *args: str) -> str:
    """Run read-only Git commands, surfacing failures to the caller."""
    return subprocess.run(
        ["git", *args], cwd=root, check=True, capture_output=True, text=True
    ).stdout


def merge_changelogs(bodies: list[str]) -> str:
    """Group level-two sections without rewriting entries or dropping unknown sections."""
    sections: dict[str, list[str]] = {}
    introductions: list[str] = []
    for body in bodies:
        body = re.sub(r"\A# [^\n]*(?:\n|$)", "", body).strip()
        parts = re.split(r"(?m)^## ([^\n]+)\n?", body)
        if parts[0].strip():
            introductions.append(parts[0].strip())
        for index in range(1, len(parts), 2):
            heading = parts[index].strip()
            content = parts[index + 1].strip()
            sections.setdefault(heading, [])
            if content:
                sections[heading].append(content)
    if not sections:
        sections = {
            name: [] for name in ("Added", "Fixed", "Changed", "Breaking changes")
        }
    blocks = ["# Unreleased", *introductions]
    for heading, contents in sections.items():
        blocks.append(f"## {heading}")
        blocks.extend(contents)
    return "\n\n".join(blocks) + "\n"


def draft(
    root: Path,
    revision: str = "HEAD",
    base: str | None = None,
    *,
    first_release: bool = False,
) -> tuple[str, list[str]]:
    """Combine the working-tree unreleased changelog and tagged sources without writing."""
    end = git(root, "rev-parse", "--verify", f"{revision}^{{commit}}").strip()
    tags = git(root, "tag", "--merged", end).splitlines()
    commits = {
        tag: git(root, "rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}").strip()
        for tag in tags
        if STABLE.fullmatch(tag) or DEV.fullmatch(tag)
    }
    history = git(root, "rev-list", "--topo-order", end).splitlines()
    if first_release and base:
        raise ValueError("--first-release and --base cannot be combined")
    if base is not None:
        if not STABLE.fullmatch(base) or base not in commits:
            raise ValueError("base must be a reachable plain stable tag")
    elif not first_release:
        # Nearest in topological traversal, not by version or tag date.
        stable = {commits[tag] for tag in commits if STABLE.fullmatch(tag)}
        nearest = next((commit for commit in history if commit in stable), None)
        if nearest is None:
            raise ValueError("no reachable stable tag; use --first-release explicitly")
        base = min(
            tag for tag in commits if STABLE.fullmatch(tag) and commits[tag] == nearest
        )
    excluded = set(git(root, "rev-list", commits[base]).splitlines()) if base else set()
    ordered = [commit for commit in reversed(history) if commit not in excluded]
    sources: list[str] = []
    missing: list[str] = []
    for commit in ordered:
        for tag in sorted(
            tag for tag in commits if commits[tag] == commit and DEV.fullmatch(tag)
        ):
            path = f"changelogs/{tag}.md"
            try:
                body = git(root, "show", f"{commit}:{path}")
            except subprocess.CalledProcessError:
                missing.append(tag)
                continue
            sources.append(body)
    unreleased = root / "changelogs" / "unreleased.md"
    try:
        current = unreleased.read_text(encoding="utf-8")
    except FileNotFoundError:
        current = ""
    text = merge_changelogs([current, *sources])
    boundary = base or "repository start (first release)"
    text += (
        f"\n<!-- Sources after {boundary} through {end}.\n"
        "Review, deduplicate, and remove superseded details. -->\n"
    )
    if missing:
        text += "\n<!-- Missing changelogs: " + ", ".join(missing) + " -->\n"
    return text, missing


def main() -> None:
    """Preview generated material, or write it exclusively to an explicit path."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="explicit reachable stable tag")
    parser.add_argument(
        "--revision", default="HEAD", help="release commit (default: HEAD)"
    )
    parser.add_argument(
        "--first-release", action="store_true", help="collect all reachable history"
    )
    output_group = parser.add_mutually_exclusive_group()
    output_group.add_argument(
        "--stdout", action="store_true", help="preview without writing (default)"
    )
    output_group.add_argument(
        "--output", type=Path, help="save generated material to a new file"
    )
    args = parser.parse_args()
    try:
        text, missing = draft(
            ROOT, args.revision, args.base, first_release=args.first_release
        )
        for tag in missing:
            print(f"warning: missing changelogs/{tag}.md at its tag", file=sys.stderr)
        if args.output is None:
            print(text, end="")
        else:
            destination = args.output
            destination.parent.mkdir(parents=True, exist_ok=True)
            try:
                with destination.open("x", encoding="utf-8") as output:
                    output.write(text)
            except FileExistsError:
                parser.exit(
                    1,
                    f"error: {destination} already exists; "
                    "use --stdout to preview without overwriting it.\n",
                )
            print(
                f"Wrote {destination}; review and merge into changelogs/unreleased.md."
            )
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"error: {error}\n")


if __name__ == "__main__":
    main()
