"""Test version handling with temporary manifests, a fixed date, and mocked Git."""

import argparse
import subprocess
from datetime import UTC, datetime
from pathlib import Path
from unittest.mock import Mock, call

import main as tag_tool
import pytest
import tomli


@pytest.mark.parametrize(
    "tag",
    [
        "v0.0.0",
        "v1.0.0",
        "v0.1.0",
        "v0.0.1",
        "v12.34.56",
        "v1.2.3-alpha",
        "v1.2.3-rc.0",
        "v1.2.3-01a",
        "v1.2.3+001",
        "v1.2.3-rc.1+build.01",
        "v1.2.3+26w40aa",
    ],
)
def test_semver_tag_accepts_valid_versions(tag: str) -> None:
    assert tag_tool.semver_tag(tag) == tag[1:]


@pytest.mark.parametrize(
    "tag",
    [
        "",
        "1.2.3",
        "v1",
        "v1.2",
        "v1.2.3.4",
        "v01.2.3",
        "v1.02.3",
        "v1.2.03",
        "v1.2.3-01",
        "v1.2.3-rc.01",
        "v1.2.3-",
        "v1.2.3+",
        "v1.2.3-alpha..1",
        "v1.2.3+build..1",
        "v1.2.3-a_b",
        "v1.2.3+é",
        " v1.2.3",
        "v1.2.3\n",
    ],
)
def test_semver_tag_rejects_invalid_versions(tag: str) -> None:
    with pytest.raises(argparse.ArgumentTypeError):
        tag_tool.semver_tag(tag)


@pytest.mark.parametrize(
    ("suffix", "expected"),
    [
        ("", "a"),
        ("a", "b"),
        ("y", "z"),
        ("z", "aa"),
        ("aa", "ab"),
        ("az", "ba"),
        ("zz", "aaa"),
        ("azz", "baa"),
    ],
)
def test_next_suffix(suffix: str, expected: str) -> None:
    assert tag_tool.next_suffix(suffix) == expected


@pytest.mark.parametrize("newline", ["\n", "\r\n"])
def test_update_version_preserves_other_content(tmp_path: Path, newline: str) -> None:
    manifest = tmp_path / "Cargo.toml"
    original = newline.join(
        [
            "# manifest",
            "[package]",
            'name = "example"',
            'version = "0.0.0" # keep comment',
            "",
            "[dependencies.example]",
            'version = "0.0.0"',
            "",
        ]
    ).encode()
    manifest.write_bytes(original)

    assert tag_tool.update_version(manifest, "1.2.3+26w40a") == "0.0.0"
    assert manifest.read_bytes() == original.replace(
        b'version = "0.0.0"',
        b'version = "1.2.3+26w40a"',
        1,
    )


def test_invalid_manifest_is_not_written(tmp_path: Path) -> None:
    manifest = tmp_path / "Cargo.toml"
    original = b'[package]\nversion = "unfinished\n'
    manifest.write_bytes(original)
    with pytest.raises(tomli.TOMLDecodeError):
        tag_tool.update_version(manifest, "1.2.3")
    assert manifest.read_bytes() == original


@pytest.fixture
def cli_environment(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> tuple[Path, Mock]:
    """Isolate CLI side effects and fix the clock to ISO week 40 of 2026."""
    manifest = tmp_path / "Cargo.toml"
    manifest.write_text('[package]\nname = "example"\nversion = "1.2.3"\n')
    monkeypatch.setattr(tag_tool, "DESKTOP_APP_ROOT", tmp_path)
    monkeypatch.setattr(tag_tool, "CHESSVAULT_ROOT", tmp_path)
    git = Mock(return_value=subprocess.CompletedProcess([], 0, stdout=""))
    monkeypatch.setattr(tag_tool.subprocess, "run", git)
    clock = Mock()
    clock.now.return_value = datetime(2026, 10, 1, tzinfo=UTC)
    monkeypatch.setattr(tag_tool, "datetime", clock)
    return manifest, git


@pytest.mark.parametrize(
    ("args", "initial", "expected"),
    [
        (["v2.0.0"], "1.2.3", "2.0.0"),
        (["v2.0.0-rc.1+build.01"], "1.2.3", "2.0.0-rc.1+build.01"),
        (["--dev-rel"], "1.2.3", "1.2.3+26w40a"),
        (["--dev-rel"], "1.2.3+26w40", "1.2.3+26w40a"),
        (["--dev-rel"], "1.2.3+26w40a", "1.2.3+26w40b"),
        (["--dev-rel"], "1.2.3+26w40z", "1.2.3+26w40aa"),
        (["--dev-rel"], "1.2.3+26w40aa", "1.2.3+26w40ab"),
        (["--dev-rel"], "1.2.3-rc.1+build.01", "1.2.3-rc.1+build.01.26w40a"),
    ],
)
def test_cli_updates_manifest_and_tags(
    cli_environment: tuple[Path, Mock],
    monkeypatch: pytest.MonkeyPatch,
    args: list[str],
    initial: str,
    expected: str,
) -> None:
    manifest, git = cli_environment
    manifest.write_text(f'[package]\nversion = "{initial}"\n')
    monkeypatch.setattr("sys.argv", ["tag-tool", *args])
    tag_tool.main()
    assert tomli.loads(manifest.read_text())["package"]["version"] == expected
    root = manifest.parent
    assert git.call_args_list == [
        call(
            [
                "git",
                "status",
                "--porcelain",
                "--untracked-files=all",
                "--ignore-submodules=none",
            ],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ),
        call(
            ["git", "tag", "--list", f"v{expected}"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
        ),
        call(
            ["cargo", "metadata", "--offline", "--format-version", "1"],
            cwd=root,
            check=True,
            stdout=subprocess.DEVNULL,
        ),
        call(["git", "add", "--", "Cargo.toml", "Cargo.lock"], cwd=root, check=True),
        call(
            [
                "git",
                "commit",
                "-m",
                f"[desktop] Release v{expected}",
                "--",
                "Cargo.toml",
                "Cargo.lock",
            ],
            cwd=root,
            check=True,
        ),
        call(["git", "tag", f"v{expected}"], cwd=root, check=True),
    ]


@pytest.mark.parametrize("args", [[], ["bad"], ["--dev-rel", "v1.0.0"], ["--unknown"]])
def test_invalid_cli_does_not_write_or_tag(
    cli_environment: tuple[Path, Mock],
    monkeypatch: pytest.MonkeyPatch,
    args: list[str],
) -> None:
    manifest, git = cli_environment
    original = manifest.read_bytes()
    monkeypatch.setattr("sys.argv", ["tag-tool", *args])
    with pytest.raises(SystemExit) as error:
        tag_tool.main()
    assert error.value.code == 2
    assert manifest.read_bytes() == original
    git.assert_not_called()


@pytest.mark.parametrize("failure_index", range(6))
def test_git_failure_propagates(
    cli_environment: tuple[Path, Mock],
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
    failure_index: int,
) -> None:
    _, git = cli_environment
    monkeypatch.setattr("sys.argv", ["tag-tool", "v2.0.0"])
    git.side_effect = [
        *[subprocess.CompletedProcess([], 0, stdout="") for _ in range(failure_index)],
        subprocess.CalledProcessError(128, ["release-command"]),
    ]
    with pytest.raises(subprocess.CalledProcessError):
        tag_tool.main()
    assert git.call_count == failure_index + 1
    assert "Created tag:" not in capsys.readouterr().out


@pytest.mark.parametrize(
    "status", [" M Cargo.toml\n", "M  Cargo.toml\n", "?? new.txt\n"]
)
def test_dirty_tree_is_rejected(
    cli_environment: tuple[Path, Mock],
    monkeypatch: pytest.MonkeyPatch,
    status: str,
) -> None:
    manifest, git = cli_environment
    original = manifest.read_bytes()
    git.return_value.stdout = status
    monkeypatch.setattr("sys.argv", ["tag-tool", "v2.0.0"])
    with pytest.raises(SystemExit) as error:
        tag_tool.main()
    assert error.value.code == 2
    assert manifest.read_bytes() == original
    assert git.call_count == 1


def test_existing_tag_is_rejected(
    cli_environment: tuple[Path, Mock],
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    manifest, git = cli_environment
    original = manifest.read_bytes()
    git.side_effect = [
        subprocess.CompletedProcess([], 0, stdout=""),
        subprocess.CompletedProcess([], 0, stdout="v2.0.0\n"),
    ]
    monkeypatch.setattr("sys.argv", ["tag-tool", "v2.0.0"])
    with pytest.raises(SystemExit) as error:
        tag_tool.main()
    assert error.value.code == 2
    assert manifest.read_bytes() == original
    assert git.call_count == 2


@pytest.mark.parametrize("with_changelog", [False, True])
@pytest.mark.parametrize("title", [b"# Unreleased", b"# Changes"])
def test_tag_contains_committed_release_files(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    with_changelog: bool,
    title: bytes,
) -> None:
    """Exercise real Git in a temporary repository, substituting only Cargo."""
    real_run = subprocess.run

    def git(*args: str) -> str:
        return real_run(
            ["git", *args],
            cwd=tmp_path,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()

    # Ignore user signing/hooks configuration for this disposable repository.
    monkeypatch.setenv("GIT_CONFIG_GLOBAL", str(tmp_path / "no-global-config"))
    monkeypatch.setenv("GIT_CONFIG_NOSYSTEM", "1")
    monkeypatch.setenv("GIT_AUTHOR_NAME", "Tag Tool Test")
    monkeypatch.setenv("GIT_AUTHOR_EMAIL", "tag-tool@example.invalid")
    monkeypatch.setenv("GIT_COMMITTER_NAME", "Tag Tool Test")
    monkeypatch.setenv("GIT_COMMITTER_EMAIL", "tag-tool@example.invalid")
    git("init")
    manifest = tmp_path / "apps" / "chessvault" / "Cargo.toml"
    manifest.parent.mkdir(parents=True)
    manifest.write_text('[package]\nname = "chessvault"\nversion = "1.2.3"\n')
    lockfile = tmp_path / "Cargo.lock"
    lockfile.write_text(
        'version = 4\n[[package]]\nname = "chessvault"\nversion = "1.2.3"\n'
    )
    if with_changelog:
        changelog = tmp_path / "changelogs" / "unreleased.md"
        changelog.parent.mkdir()
        changelog.write_bytes(title + b"\r\n\r\n- Improved chess.\r\n")
    git("add", ".")
    git("commit", "-m", "Initial state")
    original_head = git("rev-parse", "HEAD")

    def run(
        args: list[str],
        *,
        cwd: Path,
        check: bool,
        capture_output: bool = False,
        text: bool = False,
        stdout: int | None = None,
    ) -> subprocess.CompletedProcess:
        if args[0] == "cargo":
            lockfile.write_text(lockfile.read_text().replace('"1.2.3"', '"2.0.0"'))
            return subprocess.CompletedProcess(args, 0)
        return real_run(
            args,
            cwd=cwd,
            check=check,
            capture_output=capture_output,
            text=text,
            stdout=stdout,
        )

    monkeypatch.setattr(tag_tool, "CHESSVAULT_ROOT", tmp_path)
    monkeypatch.setattr(tag_tool, "DESKTOP_APP_ROOT", manifest.parent)
    monkeypatch.setattr(tag_tool.subprocess, "run", run)
    monkeypatch.setattr("sys.argv", ["tag-tool", "v2.0.0"])
    tag_tool.main()

    assert git("rev-parse", "HEAD") != original_head
    assert git("rev-parse", "v2.0.0") == git("rev-parse", "HEAD")
    assert git("status", "--porcelain") == ""
    assert 'version = "2.0.0"' in git("show", "v2.0.0:apps/chessvault/Cargo.toml")
    assert 'version = "2.0.0"' in git("show", "v2.0.0:Cargo.lock")
    if with_changelog:
        assert not changelog.exists()
        expected_title = b"# v2.0.0" if title == b"# Unreleased" else title
        assert changelog.with_name("v2.0.0.md").read_bytes() == (
            expected_title + b"\r\n\r\n- Improved chess.\r\n"
        )
        assert "Improved chess." in git("show", "v2.0.0:changelogs/v2.0.0.md")
        assert "changelogs/unreleased.md" not in git(
            "ls-tree", "-r", "--name-only", "v2.0.0"
        )


@pytest.mark.parametrize("collision", [False, True])
def test_development_changelog(
    cli_environment: tuple[Path, Mock],
    monkeypatch: pytest.MonkeyPatch,
    collision: bool,
) -> None:
    manifest, git = cli_environment
    original = manifest.read_bytes()
    changelog = manifest.parent / "changelogs" / "unreleased.md"
    changelog.parent.mkdir()
    changelog.write_text("# Unreleased\n\nDevelopment changes\n")
    target = changelog.with_name("v1.2.3+26w40a.md")
    monkeypatch.setattr("sys.argv", ["tag-tool", "--dev-rel"])
    if collision:
        target.write_text("Existing notes\n")
        with pytest.raises(SystemExit) as error:
            tag_tool.main()
        assert error.value.code == 2
        assert manifest.read_bytes() == original
        assert changelog.read_text() == "# Unreleased\n\nDevelopment changes\n"
        assert target.read_text() == "Existing notes\n"
        assert git.call_count == 2
    else:
        tag_tool.main()
        assert not changelog.exists()
        assert target.read_text() == "# v1.2.3+26w40a\n\nDevelopment changes\n"
        for command in (git.call_args_list[3], git.call_args_list[4]):
            assert command.args[0][-2:] == [
                "changelogs/unreleased.md",
                "changelogs/v1.2.3+26w40a.md",
            ]
