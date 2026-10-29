"""Package Cargo-reported macOS outputs with numeric CI bundle versions."""

import argparse
import json
import re
import subprocess
from pathlib import Path

import package_macos


def bundle_versions(
    cargo_version: str, build_number: int, tag: str | None = None
) -> tuple[str, str]:
    """Keep SemVer's release core; encode a run number in Apple's 4/2/2 digits.

    Reruns keep the same bundle version; counters are scoped to each workflow.
    The range is deliberately bounded rather than producing invalid Apple versions.
    """
    if tag is not None and tag != f"v{cargo_version}":
        raise ValueError(
            f"release tag {tag!r} does not match Cargo version {cargo_version!r}"
        )
    match = re.fullmatch(r"([0-9]+\.[0-9]+\.[0-9]+)(?:[-+].+)?", cargo_version)
    if match is None:
        raise ValueError(f"invalid Cargo version: {cargo_version!r}")
    if not 1 <= build_number < 99_990_000:
        raise ValueError("build number must be between 1 and 99989999")
    major = build_number // 10_000 + 1
    minor = build_number // 100 % 100
    patch = build_number % 100
    return match[1], f"{major}.{minor}.{patch}"


def cargo_inputs(messages: Path, metadata: Path) -> tuple[Path, Path, str]:
    """Use Cargo's package IDs and output paths, never guess hashed OUT_DIRs."""
    packages = [
        package
        for package in json.loads(metadata.read_text())["packages"]
        if package["name"] == "chessvault"
    ]
    if len(packages) != 1:
        raise ValueError(f"expected one chessvault package, found {len(packages)}")
    package = packages[0]
    records = [
        json.loads(line) for line in messages.read_text().splitlines() if line.strip()
    ]
    records = [
        record for record in records if record.get("package_id") == package["id"]
    ]
    binaries = [
        Path(record["executable"])
        for record in records
        if record.get("reason") == "compiler-artifact"
        and record.get("executable")
        and record["target"]["name"] == "chessvault"
        and "bin" in record["target"]["kind"]
        and not record["profile"]["test"]
    ]
    icons = [
        Path(record["out_dir"]) / "chessvault.icns"
        for record in records
        if record.get("reason") == "build-script-executed"
        and (Path(record["out_dir"]) / "chessvault.icns").is_file()
    ]
    if len(binaries) != 1 or len(icons) != 1:
        raise ValueError(
            f"expected one binary and icon, found {len(binaries)} and {len(icons)}"
        )
    return binaries[0], icons[0], package["version"]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--messages", type=Path, required=True)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--build-number", type=int, required=True)
    parser.add_argument("--tag", help="release tag, which must equal v<Cargo version>")
    parser.add_argument("--minimum-macos", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        binary, icon, cargo_version = cargo_inputs(args.messages, args.metadata)
        version, build_version = bundle_versions(
            cargo_version, args.build_number, args.tag
        )
        print(
            f"Packaging Cargo {cargo_version}: display={version}, build={build_version}",
            flush=True,
        )
        package_macos.package(
            binary,
            icon,
            args.output,
            version=version,
            build_version=build_version,
            minimum_macos=args.minimum_macos,
        )
    except subprocess.CalledProcessError as error:
        parser.exit(1, f"error: {error}\n{error.stdout}{error.stderr}")
    except (OSError, ValueError, KeyError, subprocess.TimeoutExpired) as error:
        parser.exit(1, f"error: {error}\n")
    print(f"Packaged {args.output}")


if __name__ == "__main__":
    main()
