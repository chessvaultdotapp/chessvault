"""Verify macOS bundle layout without requiring macOS or a Rust build."""

import importlib.util
import plistlib
import subprocess
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

SCRIPT = Path(__file__).with_name("macos.py")
SPEC = importlib.util.spec_from_file_location("macos_prepare", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
macos = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(macos)


@pytest.fixture
def inputs(tmp_path: Path) -> tuple[Path, Path, Path]:
    binary = tmp_path / "chessvault"
    binary.write_bytes(b"fake Mach-O")
    binary.chmod(0o600)
    icon = tmp_path / "icon.icns"
    icon.write_bytes(b"test icon")
    icon.chmod(0o600)
    return binary, icon, tmp_path / "dist/ChessVault.app"


def prepare(inputs: tuple[Path, Path, Path], **overrides: str) -> None:
    metadata = {
        "version": "0.1.0",
        "build_version": "1.2.3",
        "minimum_macos": "13.0",
    }
    metadata.update(overrides)
    macos.prepare(*inputs, **metadata)


def test_bundle(inputs: tuple[Path, Path, Path]) -> None:
    prepare(inputs)
    binary, icon, output = inputs
    contents = output / "Contents"
    executable = contents / "MacOS/chessvault"
    assert executable.read_bytes() == binary.read_bytes()
    assert executable.stat().st_mode & 0o777 == 0o755
    assert (contents / "Resources/chessvault.icns").read_bytes() == icon.read_bytes()
    assert (contents / "Resources/LICENSE").read_bytes() == (
        macos.ROOT / "LICENSE"
    ).read_bytes()
    metadata = plistlib.loads((contents / "Info.plist").read_bytes())
    assert metadata == {
        "CFBundleDevelopmentRegion": "en",
        "CFBundleDisplayName": "ChessVault",
        "CFBundleExecutable": "chessvault",
        "CFBundleIconFile": "chessvault.icns",
        "CFBundleIdentifier": "app.chessvault.ChessVault",
        "CFBundleInfoDictionaryVersion": "6.0",
        "CFBundleName": "ChessVault",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": "0.1.0",
        "CFBundleVersion": "1.2.3",
        "LSMinimumSystemVersion": "13.0",
        "NSHighResolutionCapable": True,
        "LSApplicationCategoryType": "public.app-category.board-games",
    }
    assert sorted(str(path.relative_to(output)) for path in output.rglob("*")) == [
        "Contents",
        "Contents/Info.plist",
        "Contents/MacOS",
        "Contents/MacOS/chessvault",
        "Contents/Resources",
        "Contents/Resources/LICENSE",
        "Contents/Resources/chessvault.icns",
    ]
    for path in [output, *output.rglob("*")]:
        expected = 0o755 if path.is_dir() or path == executable else 0o644
        assert path.stat().st_mode & 0o777 == expected


@pytest.mark.parametrize("kind", ["directory", "file", "dangling-symlink"])
def test_existing_output(inputs: tuple[Path, Path, Path], kind: str) -> None:
    output = inputs[2]
    output.parent.mkdir()
    if kind == "directory":
        output.mkdir()
    elif kind == "file":
        output.write_bytes(b"keep")
    else:
        output.symlink_to("missing")
    with pytest.raises(ValueError, match="output already exists"):
        prepare(inputs)
    assert output.exists() or output.is_symlink()


@pytest.mark.parametrize("index", [0, 1])
def test_missing_inputs(inputs: tuple[Path, Path, Path], index: int) -> None:
    inputs[index].unlink()
    with pytest.raises(ValueError, match="not a regular file"):
        prepare(inputs)
    assert not inputs[2].exists()


@pytest.mark.parametrize("key", ["version", "build_version", "minimum_macos"])
@pytest.mark.parametrize(
    "value", ["", "v1.2.3", "1.2.3+dev", "1.2.3-beta", "1.2.3.4", "../1"]
)
def test_invalid_metadata(
    inputs: tuple[Path, Path, Path], key: str, value: str
) -> None:
    with pytest.raises(ValueError, match="numeric version"):
        prepare(inputs, **{key: value})
    assert not inputs[2].exists()


def test_output_suffix(inputs: tuple[Path, Path, Path]) -> None:
    with pytest.raises(ValueError, match=r"\.app suffix"):
        prepare((inputs[0], inputs[1], inputs[2].with_suffix(".zip")))


def test_copy_failure_is_atomic(inputs: tuple[Path, Path, Path]) -> None:
    with (
        patch.object(macos.shutil, "copyfile", side_effect=OSError("copy failed")),
        pytest.raises(OSError, match="copy failed"),
    ):
        prepare(inputs)
    assert list(inputs[2].parent.iterdir()) == []


def test_cli(inputs: tuple[Path, Path, Path]) -> None:
    binary, icon, output = inputs
    command = [
        sys.executable,
        str(SCRIPT),
        str(binary),
        "--icon",
        str(icon),
        "--output",
        str(output),
        "--version",
        "0.1.0",
        "--build-version",
        "1",
        "--minimum-macos",
        "13.0",
    ]
    result = subprocess.run(
        command, capture_output=True, text=True, check=False, timeout=30
    )
    assert result.returncode == 0, result.stderr
    assert output.is_dir()
    result = subprocess.run(
        command, capture_output=True, text=True, check=False, timeout=30
    )
    assert result.returncode == 1
    assert "output already exists" in result.stderr
    assert "Traceback" not in result.stderr
