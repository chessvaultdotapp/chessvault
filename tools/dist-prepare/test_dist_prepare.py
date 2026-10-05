"""Exercise staging and the command-line interface without a real Rust build."""

import importlib.util
import os
import subprocess
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

SCRIPT = Path(__file__).with_name("main.py")
SPEC = importlib.util.spec_from_file_location("dist_prepare", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
dist_prepare = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(dist_prepare)


@pytest.fixture(autouse=True)
def icon(tmp_path: Path) -> Path:
    path = tmp_path / "chessvault.png"
    path.write_bytes(b"test icon")
    path.chmod(0o600)
    return path


def test_prepare(tmp_path: Path, icon: Path) -> None:
    binary = tmp_path / "built-binary"
    binary.write_bytes(b"\x00fake executable\xff")
    binary.chmod(0o600)
    dist = tmp_path / "dist"
    dist.mkdir()
    appimage = dist / "ChessVault.AppImage"
    appimage.write_bytes(b"unrelated artifact")
    output = dist / "package"

    dist_prepare.prepare(binary, output)

    staged = output / "bin" / "chessvault"
    assert staged.read_bytes() == binary.read_bytes()
    desktop = output / "share/applications/chessvault.desktop"
    assert desktop.read_bytes() == dist_prepare.DESKTOP_ENTRY.read_bytes()
    assert sorted(
        path.relative_to(output).as_posix() for path in output.rglob("*")
    ) == [
        "bin",
        "bin/chessvault",
        "share",
        "share/applications",
        "share/applications/chessvault.desktop",
        "share/icons",
        "share/icons/hicolor",
        "share/icons/hicolor/256x256",
        "share/icons/hicolor/256x256/apps",
        "share/icons/hicolor/256x256/apps/chessvault.png",
        "share/icons/hicolor/scalable",
        "share/icons/hicolor/scalable/apps",
        "share/icons/hicolor/scalable/apps/chessvault.svg",
    ]
    staged_icon = output / "share/icons/hicolor/256x256/apps/chessvault.png"
    assert staged_icon.read_bytes() == icon.read_bytes()
    scalable_icon = output / "share/icons/hicolor/scalable/apps/chessvault.svg"
    assert scalable_icon.read_bytes() == dist_prepare.SCALABLE_ICON.read_bytes()
    if os.name == "posix":
        assert staged.stat().st_mode & 0o777 == 0o755
        assert desktop.stat().st_mode & 0o777 == 0o644
        assert staged_icon.stat().st_mode & 0o777 == 0o644
        assert scalable_icon.stat().st_mode & 0o777 == 0o644
        assert icon.stat().st_mode & 0o777 == 0o600
        assert binary.stat().st_mode & 0o777 == 0o600
    assert appimage.read_bytes() == b"unrelated artifact"
    assert sorted(path.name for path in dist.iterdir()) == [
        "ChessVault.AppImage",
        "package",
    ]


@pytest.mark.parametrize("kind", ["missing", "directory"])
def test_invalid_binary_leaves_no_output(tmp_path: Path, kind: str) -> None:
    binary = tmp_path / "binary"
    if kind == "directory":
        binary.mkdir()
    output = tmp_path / "dist" / "package"
    with pytest.raises(ValueError, match="not a regular file"):
        dist_prepare.prepare(binary, output)
    assert not output.parent.exists()


@pytest.mark.parametrize("kind", ["file", "directory", "symlink"])
def test_existing_output_is_preserved(tmp_path: Path, kind: str) -> None:
    binary = tmp_path / "binary"
    binary.write_bytes(b"binary")
    output = tmp_path / "package"
    if kind == "directory":
        output.mkdir()
        (output / "stale").write_bytes(b"keep")
    elif kind == "symlink":
        output.symlink_to(tmp_path / "missing", target_is_directory=True)
    else:
        output.write_bytes(b"keep")
    with pytest.raises(ValueError, match="output already exists"):
        dist_prepare.prepare(binary, output)
    if kind == "directory":
        assert (output / "stale").read_bytes() == b"keep"
    elif kind == "symlink":
        assert output.is_symlink()
    else:
        assert output.read_bytes() == b"keep"


def test_copy_failure_cleans_staging(tmp_path: Path) -> None:
    binary = tmp_path / "binary"
    binary.write_bytes(b"binary")
    output = tmp_path / "dist" / "package"
    with (
        patch.object(
            dist_prepare.shutil, "copyfile", side_effect=OSError("copy failed")
        ),
        pytest.raises(OSError, match="copy failed"),
    ):
        dist_prepare.prepare(binary, output)
    assert list(output.parent.iterdir()) == []


@pytest.mark.parametrize("asset", ["DESKTOP_ENTRY", "SCALABLE_ICON"])
def test_missing_asset_cleans_staging(tmp_path: Path, asset: str) -> None:
    binary = tmp_path / "binary"
    binary.write_bytes(b"binary")
    output = tmp_path / "dist/package"
    with (
        patch.object(dist_prepare, asset, tmp_path / "missing-asset"),
        pytest.raises(FileNotFoundError),
    ):
        dist_prepare.prepare(binary, output)
    assert list(output.parent.iterdir()) == []


def test_cli_from_another_directory(tmp_path: Path) -> None:
    (tmp_path / "binary").write_bytes(b"binary")
    result = subprocess.run(
        [sys.executable, str(SCRIPT), "binary", "--output", "dist/package"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    assert (tmp_path / "dist/package/bin/chessvault").read_bytes() == b"binary"
    assert (
        tmp_path / "dist/package/share/applications/chessvault.desktop"
    ).read_bytes() == dist_prepare.DESKTOP_ENTRY.read_bytes()
    assert "Prepared" in result.stdout


@pytest.mark.parametrize("kind", ["missing", "directory"])
def test_invalid_icon_leaves_no_output(tmp_path: Path, icon: Path, kind: str) -> None:
    binary = tmp_path / "binary"
    binary.write_bytes(b"binary")
    icon.unlink()
    if kind == "directory":
        icon.mkdir()
    output = tmp_path / "dist/package"
    with pytest.raises(ValueError, match="icon is not a regular file"):
        dist_prepare.prepare(binary, output)
    assert not output.parent.exists()


def test_cli_reports_error(tmp_path: Path) -> None:
    result = subprocess.run(
        [sys.executable, str(SCRIPT), "missing", "--output", "dist/package"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 1
    assert "not a regular file" in result.stderr
    assert "Traceback" not in result.stderr
    assert not (tmp_path / "dist").exists()
