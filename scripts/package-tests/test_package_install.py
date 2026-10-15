"""Smoke-test the release archive supplied through CHESSVAULT_PACKAGE."""

import hashlib
import os
import shutil
import subprocess
import tarfile
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
FILES = (
    ("binary", "0755", "bin/chessvault"),
    ("desktop", "0644", "share/applications/chessvault.desktop"),
    ("png", "0644", "share/icons/hicolor/256x256/apps/chessvault.png"),
    ("svg", "0644", "share/icons/hicolor/scalable/apps/chessvault.svg"),
    ("license", "0644", "share/licenses/chessvault/LICENSE"),
)


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").digest()


def success(result):
    assert result.returncode == 0, result.stdout + result.stderr


@pytest.fixture
def package(tmp_path):
    # Missing artifacts are errors, not skipped smoke tests.
    archive = Path(os.environ["CHESSVAULT_PACKAGE"]).resolve(strict=True)
    extracted = tmp_path / "extracted package"
    with tarfile.open(archive, "r:gz") as tar:
        tar.extractall(extracted, filter="data")
    return extracted


@pytest.fixture(params=[False, True], ids=["default", "xdg"])
def installation(tmp_path, package, request):
    return Installation(tmp_path, package, request.param)


@pytest.fixture
def installed(installation):
    success(installation.run())
    return installation


class Installation:
    def __init__(self, root, package, custom_data):
        self.root = root
        self.package = package
        self.home = root / "isolated home"
        self.home.mkdir()
        self.data = root / "custom data" if custom_data else self.home / ".local/share"
        commands = root / "commands"
        commands.mkdir()
        for name in ("update-desktop-database", "gtk-update-icon-cache"):
            command = commands / name
            command.write_text("#!/bin/sh\nexit 0\n")
            command.chmod(0o755)
        self.env = {
            "HOME": str(self.home),
            "PATH": f"{commands}:/usr/bin:/bin",
            "LC_ALL": "C",
            "TMPDIR": str(root),
        }
        if custom_data:
            self.env["XDG_DATA_HOME"] = str(self.data)

    @property
    def manifest(self):
        return self.data / "chessvault/install-manifest-v1"

    def destination(self, relative):
        if relative.startswith("share/"):
            return self.data / relative.removeprefix("share/")
        return self.home / ".local" / relative

    def run(self, *args):
        # Execute the shipped script directly, from outside the extracted package.
        return subprocess.run(
            [str(self.package / "install-linux.sh"), *args],
            cwd=self.root,
            env=self.env,
            stdin=subprocess.DEVNULL,
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )

    def snapshot(self):
        paths = [self.destination(relative) for _, _, relative in FILES]
        return {
            path: (digest(path), path.stat().st_mode & 0o777)
            for path in [*paths, self.manifest]
        }

    def verify(self):
        binary = self.destination("bin/chessvault")
        for identifier, mode, relative in FILES:
            destination = self.destination(relative)
            assert destination.stat().st_mode & 0o777 == int(mode, 8)
            source = self.package / relative
            if identifier == "desktop":
                assert destination.read_text() == source.read_text().replace(
                    "Exec=chessvault", f'Exec="{binary}"'
                )
            else:
                assert digest(destination) == digest(source)
        if "XDG_DATA_HOME" in self.env:
            assert not (self.home / ".local/share").exists()
        assert self.manifest.stat().st_mode & 0o777 == 0o600
        expected = [
            "chessvault-install-manifest\t1",
            f"prefix\t{self.home / '.local'}",
            f"data-home\t{self.data}",
        ]
        for identifier, mode, relative in FILES:
            checksum = digest(self.destination(relative)).hex()
            expected.append(f"file\t{identifier}\t{mode}\t{checksum}")
        assert self.manifest.read_text() == "\n".join(expected) + "\n"


def test_package_contents(package):
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
    assert (package / "share/applications/chessvault.desktop").read_bytes() == (
        ROOT / "apps/chessvault/data/chessvault.desktop"
    ).read_bytes()


def test_install_dry_run(installation):
    before = set(installation.root.rglob("*"))
    success(installation.run("--dry-run"))
    assert set(installation.root.rglob("*")) == before
    assert list(installation.home.iterdir()) == []
    assert not installation.data.exists()


def test_install_and_manifest_creation(installed):
    installed.verify()


def test_reinstall(installed):
    before = installed.snapshot()
    success(installed.run())
    assert installed.snapshot() == before
    installed.verify()


def test_upgrade(installed):
    # Simulate a new package revision without requiring a second Rust build.
    revised_icon = (
        installed.package / "share/icons/hicolor/scalable/apps/chessvault.svg"
    )
    revised_icon.write_bytes(
        revised_icon.read_bytes() + b"\n<!-- upgrade fixture -->\n"
    )
    before = installed.snapshot()
    success(installed.run("--dry-run"))
    assert installed.snapshot() == before
    success(installed.run())
    installed.verify()
    assert installed.snapshot() != before


def test_uninstall(installed):
    before = installed.snapshot()
    state = installed.home / ".local/state/chessvault/recreate"
    state.parent.mkdir(parents=True)
    state.write_text("preserved state")
    unrelated = installed.data / "applications/other.desktop"
    unrelated.write_text("preserved launcher")
    for source in (installed.package / "bin", installed.package / "share"):
        # Uninstall must not depend on retaining the original package payload.
        shutil.rmtree(source)
    success(installed.run("--uninstall", "--dry-run"))
    assert installed.snapshot() == before
    success(installed.run("--uninstall"))
    assert all(not path.exists() for path in before)
    assert state.read_text() == "preserved state"
    assert unrelated.read_text() == "preserved launcher"
