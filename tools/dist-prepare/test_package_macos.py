"""Test macOS packaging orchestration with isolated stand-ins for Apple tools."""

import importlib
import plistlib
import shutil
import subprocess
from pathlib import Path
from types import ModuleType
from unittest.mock import patch

import pytest


@pytest.fixture
def driver(monkeypatch: pytest.MonkeyPatch) -> ModuleType:
    monkeypatch.syspath_prepend(str(Path(__file__).parent))
    module = importlib.import_module("package_macos")
    monkeypatch.setattr(module.sys, "platform", "darwin")
    return module


@pytest.fixture
def inputs(tmp_path: Path) -> tuple[Path, Path, Path]:
    binary = tmp_path / "chessvault"
    binary.write_bytes(b"executable")
    icon = tmp_path / "chessvault.icns"
    icon.write_bytes(b"icon")
    return binary, icon, tmp_path / "dist/app.zip"


def invoke(driver: ModuleType, inputs: tuple[Path, Path, Path]) -> None:
    driver.package(*inputs, version="0.1.0", build_version="1", minimum_macos="13.0")


BUILD_COMMANDS = """Load command 10
      cmd LC_BUILD_VERSION
  cmdsize 32
 platform 1
    minos 13.0
      sdk 15.0
"""
LIBRARIES = "binary:\n\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0)\n"


@pytest.mark.parametrize(
    ("architectures", "commands", "libraries", "error"),
    [
        ("x86_64", BUILD_COMMANDS, LIBRARIES, "arm64-only"),
        ("arm64 x86_64", BUILD_COMMANDS, LIBRARIES, "arm64-only"),
        (
            "arm64",
            BUILD_COMMANDS.replace("minos 13.0", "minos 14.0"),
            LIBRARIES,
            "deployment target",
        ),
        ("arm64", "", LIBRARIES, "deployment target"),
        (
            "arm64",
            BUILD_COMMANDS.replace("platform 1", "platform 2"),
            LIBRARIES,
            "deployment target",
        ),
        (
            "arm64",
            BUILD_COMMANDS,
            "binary:\n\t/opt/homebrew/lib/test.dylib (version 1)\n",
            "non-system",
        ),
        (
            "arm64",
            BUILD_COMMANDS,
            "binary:\n\t@rpath/test.dylib (version 1)\n",
            "non-system",
        ),
        ("arm64", BUILD_COMMANDS, "binary:\n", "linked libraries"),
    ],
)
def test_binary_validation_rejects(
    driver: ModuleType, architectures: str, commands: str, libraries: str, error: str
) -> None:
    with (
        patch.object(driver, "run", side_effect=[architectures, commands, libraries]),
        pytest.raises(ValueError, match=error),
    ):
        driver.validate_binary(Path("binary"), "13.0")


@pytest.mark.parametrize("platform", ["1", "macos"])
def test_binary_validation_accepts(driver: ModuleType, platform: str) -> None:
    with patch.object(
        driver,
        "run",
        side_effect=[
            "arm64\n",
            BUILD_COMMANDS.replace("platform 1", f"platform {platform}"),
            LIBRARIES,
        ],
    ):
        driver.validate_binary(Path("binary"), "13.0.0")


@pytest.mark.parametrize(
    "failure", [None, "iconutil", "plutil", "codesign", "ditto", "contents"]
)
def test_package(
    driver: ModuleType, inputs: tuple[Path, Path, Path], failure: str | None
) -> None:
    calls: list[tuple[str, ...]] = []

    def fake_run(*command: str) -> str:
        calls.append(command)
        if Path(command[0]).name == failure:
            raise subprocess.CalledProcessError(1, command, stderr="failed")
        if command[:3] == ("/usr/bin/ditto", "-c", "-k"):
            Path(command[-1]).write_bytes(b"archive")
        if command[:3] == ("/usr/bin/ditto", "-x", "-k"):
            extracted = Path(command[-1]) / "ChessVault.app"
            shutil.copytree(Path(command[-2]).parent / "ChessVault.app", extracted)
            if failure == "contents":
                (extracted / "Contents/Resources/LICENSE").unlink()
        return ""

    with (
        patch.object(driver, "validate_binary") as validate,
        patch.object(driver, "run", side_effect=fake_run),
    ):
        if failure:
            error = (
                ValueError if failure == "contents" else subprocess.CalledProcessError
            )
            with pytest.raises(error):
                invoke(driver, inputs)
        else:
            invoke(driver, inputs)
        validate.assert_called_once_with(inputs[0].resolve(), "13.0")
    output = inputs[2]
    if failure:
        assert list(output.parent.iterdir()) == []
    else:
        assert output.read_bytes() == b"archive"
        assert list(output.parent.iterdir()) == [output]
        assert [Path(call[0]).name for call in calls] == [
            "iconutil",
            "plutil",
            "codesign",
            "codesign",
            "ditto",
            "ditto",
            "codesign",
        ]
        assert calls[2][1:5] == ("--force", "--sign", "-", "--timestamp=none")
        assert "--keepParent" in calls[4]


@pytest.mark.parametrize(
    "problem", ["missing-executable", "empty-icon", "permissions", "version"]
)
def test_bundle_smoke_check(
    driver: ModuleType, inputs: tuple[Path, Path, Path], problem: str
) -> None:
    binary, icon, archive = inputs
    bundle = archive.with_suffix(".app")
    driver.macos.prepare(
        binary, icon, bundle, version="0.1.0", build_version="1", minimum_macos="13.0"
    )
    contents = bundle / "Contents"
    if problem == "missing-executable":
        (contents / "MacOS/chessvault").unlink()
    elif problem == "empty-icon":
        (contents / "Resources/chessvault.icns").write_bytes(b"")
    elif problem == "permissions":
        (contents / "MacOS/chessvault").chmod(0o644)
    else:
        plist = contents / "Info.plist"
        metadata = plistlib.loads(plist.read_bytes())
        metadata["CFBundleVersion"] = "999"
        plist.write_bytes(plistlib.dumps(metadata))
    with pytest.raises(ValueError, match="extracted bundle"):
        driver.smoke_check_bundle(
            bundle, version="0.1.0", build_version="1", minimum_macos="13.0"
        )


def test_existing_output(driver: ModuleType, inputs: tuple[Path, Path, Path]) -> None:
    inputs[2].parent.mkdir()
    inputs[2].write_bytes(b"keep")
    with pytest.raises(ValueError, match="output already exists"):
        invoke(driver, inputs)
    assert inputs[2].read_bytes() == b"keep"


def test_requires_macos(
    driver: ModuleType, inputs: tuple[Path, Path, Path], monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(driver.sys, "platform", "linux")
    with pytest.raises(ValueError, match="macOS host"):
        invoke(driver, inputs)
