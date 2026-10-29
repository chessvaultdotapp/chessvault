"""Compare public native Rust APIs with an independently pinned Python reference."""

import json
import os
import subprocess
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[2]
REFERENCE = Path(__file__).with_name("reference.py")
PIN = (
    (ROOT / "venvs/platform-dirs-differential.requirements.txt")
    .read_text()
    .splitlines()[0]
    .split("==")[1]
)


@pytest.fixture(scope="session")
def probe() -> Path:
    if sys.platform not in {"linux", "darwin"}:
        pytest.fail("The native differential suite requires Linux or macOS")
    command = [
        "cargo",
        "build",
        "-p",
        "platform-dirs",
        "--example",
        "differential_probe",
        "--no-default-features",
        "--locked",
        "--message-format=json",
    ]
    build = subprocess.run(
        command, cwd=ROOT, text=True, capture_output=True, check=False, timeout=180
    )
    assert build.returncode == 0, f"Probe build failed:\n{build.stdout}\n{build.stderr}"
    for line in build.stdout.splitlines():
        message = json.loads(line)
        if (
            message.get("reason") == "compiler-artifact"
            and message.get("executable")
            and message["target"]["name"] == "differential_probe"
        ):
            return Path(message["executable"])
    pytest.fail(f"Cargo did not report the probe executable: {build.stdout}")


def compare(probe: Path, changes: dict[str, str], app: str | None):
    # Child-only environment changes: never mutate os.environ in the test process.
    env = {
        key: value for key, value in os.environ.items() if not key.startswith("XDG_")
    }
    env.update(HOME="/differential/home")
    env.update(changes)
    request = json.dumps({"app": app})

    def run(command: list[str]):
        result = subprocess.run(
            command,
            input=request,
            text=True,
            capture_output=True,
            env=env,
            cwd=ROOT,
            check=False,
            timeout=30,
        )
        assert result.returncode == 0, (
            f"app={app!r}, environment={changes!r}, reference pin={PIN}\n"
            f"command={command!r}, exit={result.returncode}\n"
            f"stdout={result.stdout}\nstderr={result.stderr}"
        )
        return json.loads(result.stdout)

    rust = run([str(probe)])
    reference = run([sys.executable, str(REFERENCE)])
    context = (
        f"app={app!r}, environment={changes!r}, platformdirs={reference['version']}\n"
        f"Rust={rust!r}\nPython={reference['outputs']!r}"
    )
    assert reference["version"] == PIN, context
    return rust, reference["outputs"], context


PARITY_CASES = [
    pytest.param({}, id="unset"),
    pytest.param(
        {"XDG_STATE_HOME": "", "XDG_DATA_HOME": "", "XDG_DATA_DIRS": ""}, id="empty"
    ),
    pytest.param({"HOME": "/custom/home"}, id="custom-home"),
    pytest.param(
        {"XDG_STATE_HOME": "/custom/state", "XDG_DATA_HOME": "/custom/data"},
        id="absolute",
    ),
    pytest.param(
        {"XDG_STATE_HOME": "relative/state", "XDG_DATA_HOME": "../data"},
        id="relative-user",
    ),
    pytest.param({"XDG_DATA_DIRS": "/second:/first:/second"}, id="ordered-duplicates"),
    pytest.param(
        {"XDG_DATA_DIRS": ":relative:/opt/share::../share:"}, id="mixed-system"
    ),
    pytest.param(
        {"XDG_STATE_HOME": "/state/with space", "XDG_DATA_HOME": "/data/échecs"},
        id="unicode-spaces",
    ),
]


@pytest.mark.parametrize("changes", PARITY_CASES)
@pytest.mark.parametrize("app", [None, "chessvault", "échecs", "two words"])
def test_parity(probe: Path, changes: dict[str, str], app: str | None):
    rust, python, context = compare(probe, changes, app)
    assert rust == python, context


@pytest.mark.skipif(sys.platform != "darwin", reason="Native macOS directory policy")
@pytest.mark.parametrize("app", [None, "chessvault", "échecs", "two words"])
def test_macos_paths_and_log_opinions(probe: Path, app: str | None):
    changes = {
        "HOME": "/Users/player échecs",
        "XDG_STATE_HOME": "/ignored/state",
        "XDG_DATA_HOME": "/ignored/data",
        "XDG_DATA_DIRS": "/ignored/site",
    }
    rust, python, context = compare(probe, changes, app)
    suffix = f"/{app}" if app else ""
    data = f"/Users/player échecs/Library/Application Support{suffix}"
    logs = f"/Users/player échecs/Library/Logs{suffix}"
    expected = {
        "state": {"paths": [data]},
        "data": {"paths": [data]},
        "site": {"paths": [f"/Library/Application Support{suffix}"]},
        "log_default": {"paths": [logs]},
        "log_true": {"paths": [logs]},
        "log_false": {"paths": [logs]},
    }
    assert rust == python == expected, context


@pytest.mark.skipif(sys.platform != "linux", reason="Linux XDG policy divergence")
@pytest.mark.parametrize("value", ["relative", ":relative::../data:"])
@pytest.mark.parametrize("app", [None, "chessvault"])
def test_all_invalid_system_entries(probe: Path, value: str, app: str | None):
    rust, python, context = compare(probe, {"XDG_DATA_DIRS": value}, app)
    suffix = f"/{app}" if app else ""
    # Python falls back after filtering; Rust preserves an explicitly empty list.
    assert rust.pop("site") == {"paths": []}, context
    assert python.pop("site") == {
        "paths": [f"/usr/local/share{suffix}", f"/usr/share{suffix}"]
    }, context
    assert rust == python, context


@pytest.mark.skipif(sys.platform != "linux", reason="Linux XDG policy divergence")
def test_whitespace_is_preserved(probe: Path):
    changes = {
        "XDG_STATE_HOME": "/state ",
        "XDG_DATA_HOME": "/data ",
        "XDG_DATA_DIRS": "/site ",
    }
    rust, python, context = compare(probe, changes, None)
    # Python strips surrounding whitespace; Rust preserves native path contents.
    for outputs, space in [(rust, " "), (python, "")]:
        assert outputs == {
            "state": {"paths": [f"/state{space}"]},
            "data": {"paths": [f"/data{space}"]},
            "site": {"paths": [f"/site{space}"]},
            "log_default": {"paths": [f"/state{space}/log"]},
            "log_true": {"paths": [f"/state{space}/log"]},
            "log_false": {"paths": [f"/state{space}"]},
        }, context


@pytest.mark.parametrize(
    "app", ["", ".", "..", "nested/app", "back\\slash", "colon:name", "nul\0name"]
)
def test_invalid_application_names(probe: Path, app: str):
    rust, python, context = compare(probe, {}, app)
    assert all("error" in result for result in rust.values()), context
    # The reference rejects parent traversal, but accepts the other names that
    # Rust forbids (including empty names, separators, colons, and NUL).
    expected = "error" if app == ".." else "paths"
    assert all(expected in result for result in python.values()), context


def test_relative_home(probe: Path):
    rust, python, context = compare(probe, {"HOME": "relative/home"}, None)
    assert rust.pop("site") == python.pop("site"), context
    assert all("error" in result for result in rust.values()), context
    if sys.platform == "darwin":
        assert python == {
            "state": {"paths": ["relative/home/Library/Application Support"]},
            "data": {"paths": ["relative/home/Library/Application Support"]},
            "log_default": {"paths": ["relative/home/Library/Logs"]},
            "log_true": {"paths": ["relative/home/Library/Logs"]},
            "log_false": {"paths": ["relative/home/Library/Logs"]},
        }, context
        return
    assert python == {
        "state": {"paths": ["relative/home/.local/state"]},
        "data": {"paths": ["relative/home/.local/share"]},
        "log_default": {"paths": ["relative/home/.local/state/log"]},
        "log_true": {"paths": ["relative/home/.local/state/log"]},
        "log_false": {"paths": ["relative/home/.local/state"]},
    }, context
