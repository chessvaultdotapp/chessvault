"""Test version handling with temporary manifests, a fixed date, and mocked Git."""

import argparse
from datetime import UTC, datetime
from pathlib import Path
import subprocess
from unittest.mock import Mock

import pytest
import tomli

import main as tag_tool


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
    git = Mock()
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
    git.assert_called_once_with(
        ["git", "tag", f"v{expected}"], cwd=manifest.parent, check=True
    )


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


def test_git_failure_propagates(
    cli_environment: tuple[Path, Mock],
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    _, git = cli_environment
    monkeypatch.setattr("sys.argv", ["tag-tool", "v2.0.0"])
    git.side_effect = subprocess.CalledProcessError(128, ["git", "tag", "v2.0.0"])
    with pytest.raises(subprocess.CalledProcessError):
        tag_tool.main()
    assert "Created tag:" not in capsys.readouterr().out
