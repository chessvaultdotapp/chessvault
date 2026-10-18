"""Exercise ancestry and historical file reads using real temporary repositories."""

import importlib.util
import subprocess
from pathlib import Path

import pytest

SPEC = importlib.util.spec_from_file_location(
    "changelog_tool", Path(__file__).with_name("main.py")
)
assert SPEC and SPEC.loader
module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(module)


def run(root: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=root, check=True, capture_output=True, text=True
    ).stdout.strip()


@pytest.fixture
def repo(tmp_path: Path) -> Path:
    run(tmp_path, "init", "-b", "main")
    run(tmp_path, "config", "user.email", "test@example.com")
    run(tmp_path, "config", "user.name", "Test")
    run(tmp_path, "commit", "--allow-empty", "-m", "initial")
    return tmp_path


def release(root: Path, tag: str, body: str | None = None) -> None:
    if body is not None:
        path = root / "changelogs" / f"{tag}.md"
        path.parent.mkdir(exist_ok=True)
        path.write_text(f"# {tag}\n\n{body}\n")
        run(root, "add", ".")
    run(root, "commit", "--allow-empty", "-m", tag)
    run(root, "tag", tag)


def test_boundaries_and_historical_reads(repo: Path) -> None:
    release(repo, "v0.1.0+26w01a", "old")
    release(repo, "v0.1.0")
    release(repo, "v0.1.0+build.01.26w02aa", "first")
    run(repo, "branch", "other")
    release(repo, "v0.1.0+26w02ab", "second")
    (repo / "changelogs" / "v0.1.0+26w02ab.md").write_text("uncommitted")
    text, missing = module.draft(repo)
    assert not missing
    assert "old" not in text and "uncommitted" not in text
    assert text.index("first") < text.index("second")
    assert module.draft(repo)[0] == text
    run(repo, "checkout", "--", ".")
    run(repo, "checkout", "other")
    release(repo, "v9.0.0+26w03a", "unrelated")
    run(repo, "checkout", "main")
    assert "unrelated" not in module.draft(repo)[0]
    assert "old" in module.draft(repo, first_release=True)[0]


def test_first_release_and_missing(repo: Path) -> None:
    release(repo, "v0.1.0+26w01a")
    with pytest.raises(ValueError, match="first-release"):
        module.draft(repo)
    text, missing = module.draft(repo, first_release=True)
    assert missing == ["v0.1.0+26w01a"]
    assert "Missing changelogs" in text
    with pytest.raises(ValueError, match="reachable"):
        module.draft(repo, base="v1.0.0")


def test_explicit_base_and_revision(repo: Path) -> None:
    release(repo, "v0.1.0")
    release(repo, "v0.1.0+26w01a", "earlier entry")
    release(repo, "v0.2.0")
    release(repo, "v0.2.0+26w02a", "later entry")
    assert "earlier entry" not in module.draft(repo)[0]
    assert "earlier entry" in module.draft(repo, base="v0.1.0")[0]
    assert "later entry" not in module.draft(repo, revision="v0.2.0")[0]


def test_preserves_working_tree_unreleased(repo: Path) -> None:
    release(repo, "v0.1.0+26w01a", "## Added\n\nhistorical entry")
    path = repo / "changelogs" / "unreleased.md"
    path.write_text("# Unreleased\n\n## Added\n\nCommitted entry.\n")
    run(repo, "add", ".")
    run(repo, "commit", "-m", "maintain unreleased changelog")
    current = "# Unreleased\n\n## Added\n\nUncommitted entry.\n"
    path.write_text(current)

    text, missing = module.draft(repo, revision="v0.1.0+26w01a", first_release=True)

    assert not missing
    assert text.startswith(current)
    assert "Committed entry." not in text
    assert text.index("Uncommitted entry.") < text.index("historical entry")
    assert text.count("# Unreleased\n") == 1
    assert path.read_text() == current


def test_merges_categories_across_releases(repo: Path) -> None:
    release(
        repo, "v0.1.0+26w01a", "## Added\n\n- First feature.\n\n## Fixed\n\n- A fix."
    )
    release(
        repo,
        "v0.1.0+26w01b",
        "## Added\n\n- Second feature.\n  Continued detail.\n\n## Security\n\n- Hardening.",
    )
    current = repo / "changelogs" / "unreleased.md"
    current.write_text("# Unreleased\n\n## Added\n\n- Current feature.\n")

    text, missing = module.draft(repo, first_release=True)

    assert not missing
    assert text.count("## Added\n") == 1
    assert "## Source changelogs" not in text
    assert "### v0.1.0" not in text
    assert (
        text.index("Current feature")
        < text.index("First feature")
        < text.index("Second feature")
    )
    assert "- Second feature.\n  Continued detail." in text
    assert "## Fixed\n\n- A fix." in text
    assert "## Security\n\n- Hardening." in text


def test_cli_overwrite_and_preview(
    repo: Path, monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    release(repo, "v0.1.0")
    monkeypatch.setattr(module, "ROOT", repo)
    path = repo / "changelogs" / "unreleased.md"
    path.parent.mkdir()
    path.write_text("# Unreleased\n\nMaintained release entries.\n")
    original = path.read_bytes()
    monkeypatch.setattr("sys.argv", ["changelog-tool"])
    module.main()
    assert capsys.readouterr().out.startswith(original.decode())
    assert path.read_bytes() == original
    destination = repo / "generated" / "sources.md"
    monkeypatch.setattr("sys.argv", ["changelog-tool", "--output", str(destination)])
    module.main()
    generated = destination.read_bytes()
    assert generated.startswith(original)
    with pytest.raises(SystemExit) as error:
        module.main()
    assert error.value.code == 1
    message = capsys.readouterr().err
    assert f"{destination} already exists" in message
    assert "use --stdout to preview without overwriting it" in message
    assert "[Errno 17]" not in message
    assert destination.read_bytes() == generated
    assert path.read_bytes() == original
    monkeypatch.setattr("sys.argv", ["changelog-tool", "--stdout"])
    module.main()
    assert "# Unreleased" in capsys.readouterr().out
    assert path.read_bytes() == original
