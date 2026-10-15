"""Integration tests for the Linux installer; no Rust build required."""

import hashlib
import os
import pty
import shutil
import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
PAYLOAD = (
    "bin/chessvault",
    "share/applications/chessvault.desktop",
    "share/icons/hicolor/256x256/apps/chessvault.png",
    "share/icons/hicolor/scalable/apps/chessvault.svg",
    "share/licenses/chessvault/LICENSE",
)


@pytest.fixture
def installer(tmp_path):
    return Installer(tmp_path)


class Installer:
    def __init__(self, root):
        self.root = root
        self.home = root / "home with spaces"
        self.home.mkdir()
        self.package = root / "package with spaces"
        for name in PAYLOAD:
            path = self.package / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("fake payload\n")
        shutil.copyfile(
            ROOT / "apps/chessvault/data/chessvault.desktop",
            self.package / PAYLOAD[1],
        )
        shutil.copyfile(
            ROOT / "scripts/install-linux.sh", self.package / "install-linux.sh"
        )
        commands = root / "commands"
        commands.mkdir()
        # Avoid touching host caches; exercise optional-tool failures instead.
        for name in ("update-desktop-database", "gtk-update-icon-cache"):
            command = commands / name
            command.write_text(
                '#!/bin/sh\nprintf "%s\\n" "$0" >> "$CACHE_CALLS"\nexit 1\n'
            )
            command.chmod(0o755)
        self.env = {
            "HOME": str(self.home),
            "PATH": f"{commands}:/usr/bin:/bin",
            "LC_ALL": "C",
            "TMPDIR": str(root),
            "CACHE_CALLS": str(root / "cache-calls"),
        }

    def run(self, *args, answer=None):
        master = slave = None
        try:
            if answer is not None:
                master, slave = pty.openpty()
                os.write(master, answer.encode() + b"\n")
            return subprocess.run(
                ["/bin/sh", str(self.package / "install-linux.sh"), *map(str, args)],
                cwd=self.root,
                env=self.env,
                stdin=slave if slave is not None else subprocess.DEVNULL,
                capture_output=True,
                text=True,
                timeout=10,
                check=False,
            )
        finally:
            for descriptor in (master, slave):
                if descriptor is not None:
                    os.close(descriptor)

    def verify(self, prefix=None, data=None):
        prefix = prefix or self.home / ".local"
        data = data or self.home / ".local/share"
        manifest = data / "chessvault/install-manifest-v1"
        assert manifest.stat().st_mode & 0o777 == 0o600
        expected_records = [
            "chessvault-install-manifest\t1",
            f"prefix\t{prefix.resolve()}",
            f"data-home\t{data}",
        ]
        for identifier, mode, destination in (
            ("binary", "0755", prefix / "bin/chessvault"),
            ("desktop", "0644", data / "applications/chessvault.desktop"),
            ("png", "0644", data / "icons/hicolor/256x256/apps/chessvault.png"),
            ("svg", "0644", data / "icons/hicolor/scalable/apps/chessvault.svg"),
            ("license", "0644", data / "licenses/chessvault/LICENSE"),
        ):
            checksum = hashlib.sha256(destination.read_bytes()).hexdigest()
            expected_records.append(f"file\t{identifier}\t{mode}\t{checksum}")
        assert manifest.read_text() == "\n".join(expected_records) + "\n"
        binary = prefix / "bin/chessvault"
        assert binary.read_bytes() == (self.package / PAYLOAD[0]).read_bytes()
        assert binary.stat().st_mode & 0o777 == 0o755
        for name in PAYLOAD[1:]:
            destination = data / name.removeprefix("share/")
            assert destination.stat().st_mode & 0o777 == 0o644
            if name.endswith(".desktop"):
                expected = (
                    (self.package / name)
                    .read_text()
                    .replace("Exec=chessvault", f'Exec="{binary}"')
                )
                assert destination.read_text() == expected
            else:
                assert destination.read_bytes() == (self.package / name).read_bytes()


def success(result):
    assert result.returncode == 0, result.stdout + result.stderr


def failure(result, message):
    assert result.returncode != 0, result.stdout + result.stderr
    assert message in result.stderr


def test_default_install_from_unrelated_directory(installer):
    success(installer.run())
    installer.verify()


@pytest.mark.parametrize("xdg", ["absolute", "relative", ""])
def test_xdg_data_home(installer, xdg):
    data = installer.root / "custom data" if xdg == "absolute" else None
    installer.env["XDG_DATA_HOME"] = str(data) if data else xdg
    success(installer.run())
    installer.verify(data=data)
    if data:
        assert not (installer.home / ".local/share").exists()
    assert not (installer.root / "relative").exists()


def test_dry_run_changes_nothing(installer):
    before = set(installer.root.rglob("*"))
    result = installer.run("--dry-run", "--prefix", installer.root / "custom prefix")
    success(result)
    assert "Generated desktop entry:" in result.stdout
    assert set(installer.root.rglob("*")) == before


def test_custom_prefix_requires_confirmation(installer):
    prefix = installer.root / "custom prefix"
    failure(installer.run("--prefix", prefix), "requires interactive confirmation")
    failure(installer.run("--prefix", prefix, answer="n"), "Installation cancelled")
    assert not prefix.exists()
    success(installer.run("--prefix", prefix, answer="yes"))
    installer.verify(prefix=prefix)


@pytest.mark.parametrize(
    "prefix",
    [
        "/",
        "/usr",
        "/usr/local",
        "/opt/app",
        "/bin",
        "/sbin/app",
        "/lib/app",
        "/lib64/app",
        "//usr//local/./",
        "/tmp/../usr/local",
    ],
)
def test_system_prefixes_are_rejected(installer, prefix):
    failure(
        installer.run("--dry-run", "--prefix", prefix),
        "does not support system-wide installation",
    )
    assert list(installer.home.iterdir()) == []


@pytest.mark.parametrize(
    ("args", "message"),
    [
        (("--unknown",), "unknown argument"),
        (("--prefix",), "requires a path"),
        (("--prefix", ""), "requires a path"),
        (("--prefix", "relative"), "must be an absolute path"),
    ],
)
def test_invalid_arguments(installer, args, message):
    failure(installer.run(*args), message)
    assert list(installer.home.iterdir()) == []


def test_help_needs_no_payload(installer):
    for name in PAYLOAD:
        (installer.package / name).unlink()
    result = installer.run("--help")
    success(result)
    assert "Usage:" in result.stdout


@pytest.mark.parametrize("name", PAYLOAD)
def test_missing_payload_fails_before_installing(installer, name):
    (installer.package / name).unlink()
    failure(installer.run(), "missing or unreadable package file")
    assert list(installer.home.iterdir()) == []


@pytest.mark.parametrize("name", PAYLOAD)
@pytest.mark.parametrize("symlink", [False, True])
def test_existing_destinations_are_preserved(installer, name, symlink):
    destination = installer.home / ".local" / name
    destination.parent.mkdir(parents=True, exist_ok=True)
    if symlink:
        destination.symlink_to(installer.root / "missing target")
    else:
        destination.write_text("keep me")
    before = set(installer.home.rglob("*"))
    failure(installer.run(), "refusing to overwrite")
    assert set(installer.home.rglob("*")) == before
    if symlink:
        assert destination.readlink() == installer.root / "missing target"
    else:
        assert destination.read_text() == "keep me"


def test_non_directory_parent_fails_before_installing(installer):
    obstruction = installer.home / ".local/share/applications"
    obstruction.parent.mkdir(parents=True)
    obstruction.write_text("keep me")
    failure(installer.run(), "unsafe destination parent")
    assert not (installer.home / ".local/bin").exists()
    assert obstruction.read_text() == "keep me"


def test_exec_escaping(installer):
    installer.home = installer.root / 'special \\"$`% home'
    installer.home.mkdir()
    installer.env["HOME"] = str(installer.home)
    success(installer.run())
    binary = str(installer.home / ".local/bin/chessvault")
    escaped = (
        binary.replace("\\", "\\" * 4)
        .replace('"', "\\" * 2 + '"')
        .replace("$", "\\" * 2 + "$")
        .replace("`", "\\" * 2 + "`")
        .replace("%", "%%")
        .replace("\t", "\\t")
    )
    desktop = (
        installer.home / ".local/share/applications/chessvault.desktop"
    ).read_text()
    assert f'Exec="{escaped}"\n' in desktop


@pytest.mark.parametrize("character", ["\t", "\n", "\r"])
def test_line_break_in_executable_path_is_rejected(installer, character):
    failure(
        installer.run("--dry-run", "--prefix", installer.root / f"bad{character}path"),
        "must not contain tabs or line breaks",
    )
    assert list(installer.home.iterdir()) == []


@pytest.mark.parametrize("symlink", [False, True])
def test_existing_manifest_is_preserved(installer, symlink):
    manifest = installer.home / ".local/share/chessvault/install-manifest-v1"
    manifest.parent.mkdir(parents=True)
    if symlink:
        manifest.symlink_to(installer.root / "missing manifest")
    else:
        manifest.write_text("unrecognized record\n")
    failure(installer.run(), "installation manifest")
    assert not (installer.home / ".local/bin/chessvault").exists()
    if symlink:
        assert manifest.is_symlink()
    else:
        assert manifest.read_text() == "unrecognized record\n"


@pytest.mark.parametrize("character", ["\t", "\n", "\r"])
def test_manifest_delimiters_in_data_home_are_rejected(installer, character):
    data = installer.root / f"bad{character}data"
    installer.env["XDG_DATA_HOME"] = str(data)
    failure(installer.run(), "must not contain tabs or line breaks")
    assert not data.exists()
    assert list(installer.home.iterdir()) == []


@pytest.mark.parametrize("custom", [False, True])
def test_uninstall_without_payload_preserves_state(installer, custom):
    prefix = installer.root / "custom prefix" if custom else installer.home / ".local"
    data = installer.root / "custom data" if custom else installer.home / ".local/share"
    args = ("--prefix", prefix) if custom else ()
    if custom:
        installer.env["XDG_DATA_HOME"] = str(data)
    success(installer.run(*args, answer="yes" if custom else None))
    state = installer.home / ".local/state/chessvault/recreate"
    state.parent.mkdir(parents=True)
    state.write_text("keep state")
    unrelated = data / "applications/other.desktop"
    unrelated.write_text("keep launcher")
    for name in PAYLOAD:
        (installer.package / name).unlink()
    before = {p: p.read_bytes() for p in installer.home.rglob("*") if p.is_file()}
    success(installer.run(*args, "--uninstall", "--dry-run"))
    assert {p: p.read_bytes() for p in before} == before
    success(installer.run(*args, "--uninstall"))
    assert not (prefix / "bin/chessvault").exists()
    assert not (data / "chessvault/install-manifest-v1").exists()
    for name in PAYLOAD[1:]:
        assert not (data / name.removeprefix("share/")).exists()
    assert state.read_text() == "keep state"
    assert unrelated.read_text() == "keep launcher"
    assert (data / "applications").is_dir()
    failure(installer.run(*args, "--uninstall"), "no regular installation manifest")


@pytest.mark.parametrize("name", PAYLOAD)
@pytest.mark.parametrize(
    "change", ["content", "mode", "symlink", "directory", "missing"]
)
def test_uninstall_checks_all_files_before_removal(installer, name, change):
    success(installer.run())
    destination = installer.home / ".local" / name
    if change == "content":
        destination.write_text("modified")
    elif change == "mode":
        destination.chmod(0o600)
    else:
        destination.unlink()
        if change == "symlink":
            destination.symlink_to(installer.root / "missing target")
        elif change == "directory":
            destination.mkdir()
    if change == "missing":
        success(installer.run("--uninstall"))
        return
    before = {p: p.read_bytes() for p in installer.home.rglob("*") if p.is_file()}
    failure(installer.run("--uninstall"), "Error:")
    assert {p: p.read_bytes() for p in before} == before
    assert (installer.home / ".local/share/chessvault/install-manifest-v1").exists()


@pytest.mark.parametrize(
    "change",
    [
        "version",
        "prefix",
        "data-home",
        "checksum",
        "mode",
        "duplicate",
        "truncated",
        "trailing",
        "symlink",
    ],
)
def test_uninstall_rejects_invalid_manifest(installer, change):
    success(installer.run())
    manifest = installer.home / ".local/share/chessvault/install-manifest-v1"
    original = manifest.read_text()
    lines = original.splitlines()
    if change == "version":
        lines[0] = "chessvault-install-manifest\t2"
    elif change == "prefix":
        lines[1] = "prefix\t/tmp/other"
    elif change == "data-home":
        lines[2] = "data-home\t/tmp/other"
    elif change == "checksum":
        lines[3] = "file\tbinary\t0755\t" + "z" * 64
    elif change == "mode":
        lines[3] = lines[3].replace("0755", "0777")
    elif change == "duplicate":
        lines[4] = lines[3]
    elif change == "truncated":
        lines.pop()
    elif change == "trailing":
        lines.append("extra")
    if change == "symlink":
        target = installer.root / "manifest target"
        target.write_text(original)
        manifest.unlink()
        manifest.symlink_to(target)
    else:
        manifest.write_text("\n".join(lines) + "\n")
    failure(installer.run("--uninstall"), "Error:")
    assert (installer.home / ".local/bin/chessvault").exists()


@pytest.mark.parametrize("custom", [False, True])
def test_reinstall_and_upgrade_back_up_before_replacing(installer, custom):
    prefix = installer.root / "custom prefix" if custom else installer.home / ".local"
    data = installer.root / "custom data" if custom else installer.home / ".local/share"
    args = ("--prefix", prefix) if custom else ()
    if custom:
        installer.env["XDG_DATA_HOME"] = str(data)
    success(installer.run(*args, answer="yes" if custom else None))
    success(installer.run(*args, answer="yes" if custom else None))
    installer.verify(prefix=prefix, data=data)
    command = installer.root / "commands/install"
    command.write_text(
        "#!/bin/sh\ntransaction=${4%/*}\n"
        "for id in binary desktop png svg license manifest; do\n"
        '  test -f "$transaction/old-$id" || exit 99\n'
        'done\nexec /usr/bin/install "$@"\n'
    )
    command.chmod(0o755)
    copy = installer.root / "commands/cp"
    copy.write_text(
        '#!/bin/sh\ncase "$3" in */new-binary)\n'
        "transaction=${3%/*}\n"
        "for id in binary desktop png svg license manifest; do\n"
        '  test -f "$transaction/old-$id" || exit 99\n'
        'done ;; esac\nexec /bin/cp "$@"\n'
    )
    copy.chmod(0o755)
    (installer.package / "bin/chessvault").write_text("upgraded binary\n")
    (installer.package / PAYLOAD[2]).write_text("upgraded icon\n")
    before = {p: p.read_bytes() for p in installer.home.rglob("*") if p.is_file()}
    success(installer.run(*args, "--dry-run"))
    assert {p: p.read_bytes() for p in before} == before
    success(installer.run(*args, answer="yes" if custom else None))
    installer.verify(prefix=prefix, data=data)
    command.unlink()
    success(installer.run(*args, "--uninstall"))


@pytest.mark.parametrize("name", PAYLOAD)
def test_upgrade_restores_missing_files(installer, name):
    success(installer.run())
    (installer.home / ".local" / name).unlink()
    success(installer.run())
    installer.verify()


@pytest.mark.parametrize("phase", ["backup", "stage", "binary", "desktop", "manifest"])
def test_upgrade_failure_preserves_previous_installation(installer, phase):
    success(installer.run())
    before = {
        p: (p.read_bytes(), p.stat().st_mode)
        for p in installer.home.rglob("*")
        if p.is_file()
    }
    (installer.package / "bin/chessvault").write_text("new binary")
    if phase in ("backup", "stage"):
        command = installer.root / "commands/cp"
        pattern = "*/old-desktop" if phase == "backup" else "*/new-binary"
        command.write_text(
            f'#!/bin/sh\ncase "$*" in {pattern}) exit 1 ;; esac\nexec /bin/cp "$@"\n'
        )
    else:
        command = installer.root / "commands/install"
        command.write_text(
            f'#!/bin/sh\ncase "$4" in */new-{phase}) exit 1 ;; esac\n'
            'exec /usr/bin/install "$@"\n'
        )
    command.chmod(0o755)
    result = installer.run()
    assert result.returncode != 0
    assert {p: (p.read_bytes(), p.stat().st_mode) for p in before} == before
    command.unlink()
    success(installer.run("--uninstall"))


def test_failed_upgrade_removes_restored_missing_file(installer):
    success(installer.run())
    binary = installer.home / ".local/bin/chessvault"
    binary.unlink()
    command = installer.root / "commands/install"
    command.write_text(
        '#!/bin/sh\ncase "$4" in */new-desktop) exit 1 ;; esac\n'
        'exec /usr/bin/install "$@"\n'
    )
    command.chmod(0o755)
    assert installer.run().returncode != 0
    assert not binary.exists()
    command.unlink()
    success(installer.run())
    installer.verify()


def test_rollback_failure_retains_backups(installer):
    success(installer.run())
    original = (installer.home / ".local/bin/chessvault").read_bytes()
    (installer.package / "bin/chessvault").write_text("new binary")
    command = installer.root / "commands/install"
    command.write_text(
        '#!/bin/sh\ncase "$4" in */new-desktop) exit 1 ;; esac\n'
        'exec /usr/bin/install "$@"\n'
    )
    command.chmod(0o755)
    rename = installer.root / "commands/mv"
    rename.write_text(
        '#!/bin/sh\ncase "$3" in */old-binary) exit 1 ;; esac\nexec /bin/mv "$@"\n'
    )
    rename.chmod(0o755)
    result = installer.run()
    failure(result, "recovery backups retained")
    prefix = "Existing installation backed up to: "
    backup = Path(
        next(
            line.removeprefix(prefix)
            for line in result.stdout.splitlines()
            if line.startswith(prefix)
        )
    )
    assert (backup / "old-binary").read_bytes() == original
    assert backup.stat().st_mode & 0o777 == 0o700


@pytest.mark.parametrize("change", ["content", "mode", "symlink", "manifest", "prefix"])
def test_upgrade_refuses_conflicts_without_changes(installer, change):
    success(installer.run())
    binary = installer.home / ".local/bin/chessvault"
    manifest = installer.home / ".local/share/chessvault/install-manifest-v1"
    args = ()
    if change == "content":
        binary.write_text("edited")
    elif change == "mode":
        binary.chmod(0o600)
    elif change == "symlink":
        binary.unlink()
        binary.symlink_to(installer.package / "bin/chessvault")
    elif change == "manifest":
        manifest.write_text("invalid")
    else:
        args = ("--prefix", installer.root / "other prefix")
    before = {p: p.read_bytes() for p in installer.home.rglob("*") if p.is_file()}
    failure(installer.run(*args, answer="yes" if args else None), "Error:")
    assert {p: p.read_bytes() for p in before} == before


def test_uninstall_without_manifest_preserves_untracked_files(installer):
    destination = installer.home / ".local/bin/chessvault"
    destination.parent.mkdir(parents=True)
    destination.write_text("untracked file")
    failure(installer.run("--uninstall"), "no regular installation manifest")
    assert destination.read_text() == "untracked file"


def test_uninstall_rejects_unterminated_manifest(installer):
    success(installer.run())
    manifest = installer.home / ".local/share/chessvault/install-manifest-v1"
    manifest.write_bytes(manifest.read_bytes().rstrip(b"\n"))
    failure(installer.run("--uninstall"), "incomplete installation manifest")
    assert (installer.home / ".local/bin/chessvault").exists()


def test_uninstall_rejects_symlinked_parent(installer):
    success(installer.run())
    applications = installer.home / ".local/share/applications"
    moved = installer.root / "moved applications"
    applications.rename(moved)
    applications.symlink_to(moved)
    failure(installer.run("--uninstall"), "unsafe destination parent")
    assert (moved / "chessvault.desktop").is_file()
    assert (installer.home / ".local/bin/chessvault").exists()


def test_uninstall_partial_failure_can_be_retried(installer):
    success(installer.run())
    command = installer.root / "commands/rm"
    command.write_text(
        '#!/bin/sh\ncase "$2" in *.desktop) exit 1 ;; esac\nexec /bin/rm "$@"\n'
    )
    command.chmod(0o755)
    failure(installer.run("--uninstall"), "could not remove")
    assert not (installer.home / ".local/bin/chessvault").exists()
    assert (installer.home / ".local/share/chessvault/install-manifest-v1").exists()
    command.unlink()
    success(installer.run("--uninstall"))


def test_cache_refresh_failures_are_nonfatal(installer):
    theme = installer.home / ".local/share/icons/hicolor/index.theme"
    theme.parent.mkdir(parents=True)
    theme.write_text("fake theme")
    result = installer.run()
    success(result)
    installer.verify()
    assert "could not refresh the desktop database" in result.stderr
    assert "could not refresh the icon cache" in result.stderr
    assert len((installer.root / "cache-calls").read_text().splitlines()) == 2
