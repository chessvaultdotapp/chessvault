"""Smoke-test the release archive supplied through CHESSVAULT_PACKAGE."""

import hashlib
import os
import shutil
import subprocess
import tarfile
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").digest()


@pytest.mark.parametrize("custom_data", [False, True], ids=["default", "xdg"])
def test_install_built_package(tmp_path, custom_data):
    # Missing artifacts are errors, not skipped smoke tests.
    archive = Path(os.environ["CHESSVAULT_PACKAGE"]).resolve(strict=True)
    package = tmp_path / "extracted package"
    with tarfile.open(archive, "r:gz") as tar:
        tar.extractall(package, filter="data")

    installer = package / "install-linux.sh"
    assert installer.stat().st_mode & 0o777 == 0o755
    assert installer.read_bytes() == (ROOT / "scripts/install-linux.sh").read_bytes()
    binary = package / "bin/chessvault"
    with binary.open("rb") as stream:
        assert stream.read(4) == b"\x7fELF", "Expected the built Linux binary"
    assert binary.stat().st_mode & 0o777 == 0o755
    with (package / "share/icons/hicolor/256x256/apps/chessvault.png").open(
        "rb"
    ) as stream:
        assert stream.read(8) == b"\x89PNG\r\n\x1a\n"
    assert (
        package / "share/icons/hicolor/scalable/apps/chessvault.svg"
    ).read_bytes() == (ROOT / "apps/chessvault/assets/logo.svg").read_bytes()
    assert (package / "share/licenses/chessvault/LICENSE").read_bytes() == (
        ROOT / "LICENSE"
    ).read_bytes()
    template = (ROOT / "apps/chessvault/data/chessvault.desktop").read_text()
    assert (package / "share/applications/chessvault.desktop").read_text() == template

    home = tmp_path / "isolated home"
    home.mkdir()
    data = tmp_path / "custom data" if custom_data else home / ".local/share"
    commands = tmp_path / "commands"
    commands.mkdir()
    for name in ("update-desktop-database", "gtk-update-icon-cache"):
        command = commands / name
        command.write_text("#!/bin/sh\nexit 0\n")
        command.chmod(0o755)
    env = {
        "HOME": str(home),
        "PATH": f"{commands}:/usr/bin:/bin",
        "LC_ALL": "C",
        "TMPDIR": str(tmp_path),
    }
    if custom_data:
        env["XDG_DATA_HOME"] = str(data)

    def run(*args):
        # Execute the shipped script directly, from outside the extracted package.
        return subprocess.run(
            [str(installer), *args],
            cwd=tmp_path,
            env=env,
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )

    preview = run("--dry-run")
    assert preview.returncode == 0, preview.stdout + preview.stderr
    assert list(home.iterdir()) == []
    assert not data.exists()

    result = run()
    assert result.returncode == 0, result.stdout + result.stderr
    installed_binary = home / ".local/bin/chessvault"
    destinations = {installed_binary: binary}
    for icon in (
        "icons/hicolor/256x256/apps/chessvault.png",
        "icons/hicolor/scalable/apps/chessvault.svg",
        "licenses/chessvault/LICENSE",
    ):
        destinations[data / icon] = package / "share" / icon
    for destination, source in destinations.items():
        assert digest(destination) == digest(source)
        assert destination.stat().st_mode & 0o777 == (
            0o755 if destination == installed_binary else 0o644
        )
    desktop = data / "applications/chessvault.desktop"
    assert desktop.stat().st_mode & 0o777 == 0o644
    assert desktop.read_text() == template.replace(
        "Exec=chessvault", f'Exec="{installed_binary}"'
    )
    if custom_data:
        assert not (home / ".local/share").exists()

    manifest = data / "chessvault/install-manifest-v1"
    assert manifest.stat().st_mode & 0o777 == 0o600
    records = manifest.read_text().splitlines()
    assert records[:3] == [
        "chessvault-install-manifest\t1",
        f"prefix\t{home / '.local'}",
        f"data-home\t{data}",
    ]
    for record, identifier, mode, destination in zip(
        records[3:],
        ("binary", "desktop", "png", "svg", "license"),
        ("0755", "0644", "0644", "0644", "0644"),
        (
            installed_binary,
            desktop,
            data / "icons/hicolor/256x256/apps/chessvault.png",
            data / "icons/hicolor/scalable/apps/chessvault.svg",
            data / "licenses/chessvault/LICENSE",
        ),
        strict=True,
    ):
        assert record == f"file\t{identifier}\t{mode}\t{digest(destination).hex()}"

    before = {path: digest(path) for path in [*destinations, desktop, manifest]}
    repeated = run()
    assert repeated.returncode == 0, repeated.stdout + repeated.stderr
    assert {path: digest(path) for path in before} == before

    # Simulate a new package revision without requiring a second Rust build.
    revised_icon = package / "share/icons/hicolor/scalable/apps/chessvault.svg"
    revised_icon.write_bytes(
        revised_icon.read_bytes() + b"\n<!-- upgrade fixture -->\n"
    )
    upgraded = run()
    assert upgraded.returncode == 0, upgraded.stdout + upgraded.stderr
    installed_icon = data / "icons/hicolor/scalable/apps/chessvault.svg"
    assert digest(installed_icon) == digest(revised_icon)
    assert (
        f"file\tsvg\t0644\t{digest(installed_icon).hex()}"
        in manifest.read_text().splitlines()
    )
    before = {path: digest(path) for path in before}

    for source in (package / "bin", package / "share"):
        # Uninstall must not depend on retaining the original package payload.
        shutil.rmtree(source)
    preview = run("--uninstall", "--dry-run")
    assert preview.returncode == 0, preview.stdout + preview.stderr
    assert {path: digest(path) for path in before} == before
    removed = run("--uninstall")
    assert removed.returncode == 0, removed.stdout + removed.stderr
    assert all(not path.exists() for path in before)
