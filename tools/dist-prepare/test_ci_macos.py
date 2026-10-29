"""Test Cargo output discovery and the macOS CI version policy."""

import importlib
import json
from pathlib import Path
from types import ModuleType

import pytest


@pytest.fixture
def ci(monkeypatch: pytest.MonkeyPatch) -> ModuleType:
    monkeypatch.syspath_prepend(str(Path(__file__).parent))
    return importlib.import_module("ci_macos")


@pytest.mark.parametrize(
    ("version", "number", "expected"),
    [
        ("0.1.0+26w44a", 1, ("0.1.0", "1.0.1")),
        ("1.2.3-beta.1+build", 12345, ("1.2.3", "2.23.45")),
        ("1.2.3", 9999, ("1.2.3", "1.99.99")),
        ("1.2.3", 10000, ("1.2.3", "2.0.0")),
        ("1.2.3", 99989999, ("1.2.3", "9999.99.99")),
    ],
)
def test_versions(
    ci: ModuleType, version: str, number: int, expected: tuple[str, str]
) -> None:
    assert ci.bundle_versions(version, number) == expected


@pytest.mark.parametrize("number", [0, -1, 99990000])
def test_invalid_build_number(ci: ModuleType, number: int) -> None:
    with pytest.raises(ValueError, match="build number"):
        ci.bundle_versions("0.1.0", number)


@pytest.mark.parametrize("version", ["v1.2.3", "1.2", "not a version"])
def test_invalid_version(ci: ModuleType, version: str) -> None:
    with pytest.raises(ValueError, match="Cargo version"):
        ci.bundle_versions(version, 1)


@pytest.mark.parametrize(
    "problem", [None, "missing-icon", "duplicate-binary", "wrong-package"]
)
def test_cargo_inputs(ci: ModuleType, tmp_path: Path, problem: str | None) -> None:
    output = tmp_path / "custom target/build/chessvault-hash/out"
    output.mkdir(parents=True)
    icon = output / "chessvault.icns"
    if problem != "missing-icon":
        icon.write_bytes(b"icon")
    executable = tmp_path / "custom target/release/chessvault"
    package_id = "path+file:///checkout/apps/chessvault#0.1.0+26w44a"
    metadata = tmp_path / "metadata.json"
    metadata.write_text(
        json.dumps(
            {
                "packages": [
                    {"id": package_id, "name": "chessvault", "version": "0.1.0+26w44a"},
                ]
            }
        )
    )
    binary = {
        "reason": "compiler-artifact",
        "package_id": package_id,
        "executable": str(executable),
        "target": {"name": "chessvault", "kind": ["bin"]},
        "profile": {"test": False},
    }
    records = [
        {
            "reason": "build-script-executed",
            "package_id": package_id,
            "out_dir": str(output),
        },
        binary,
        {**binary, "package_id": "unrelated"},
        {**binary, "profile": {"test": True}},
        {"reason": "build-finished", "success": True},
    ]
    if problem == "duplicate-binary":
        records.append(binary)
    if problem == "wrong-package":
        records[0]["package_id"] = "unrelated"
    messages = tmp_path / "messages.jsonl"
    messages.write_text("\n".join(json.dumps(record) for record in records))
    if problem:
        with pytest.raises(ValueError, match="one binary and icon"):
            ci.cargo_inputs(messages, metadata)
    else:
        assert ci.cargo_inputs(messages, metadata) == (executable, icon, "0.1.0+26w44a")
