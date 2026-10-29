"""Validate, stage, ad-hoc sign, and ZIP an Apple Silicon macOS app."""

import argparse
import re
import subprocess
import sys
import tempfile
from pathlib import Path

import macos


def run(*command: str) -> str:
    result = subprocess.run(
        command, check=True, capture_output=True, text=True, timeout=120
    )
    return result.stdout


def validate_binary(binary: Path, minimum_macos: str) -> None:
    """Reject wrong architectures, deployment targets, and non-system libraries."""
    architectures = run("/usr/bin/lipo", "-archs", str(binary)).split()
    if architectures != ["arm64"]:
        raise ValueError(f"expected an arm64-only executable, found {architectures}")
    load_commands = run("/usr/bin/otool", "-l", str(binary))
    # Modern arm64 binaries use LC_BUILD_VERSION. Also recognize the older
    # LC_VERSION_MIN_MACOSX spelling rather than trusting Info.plist alone.
    targets = re.findall(
        r"\bcmd LC_BUILD_VERSION\s+cmdsize \d+\s+platform (?:1|macos)\s+minos ([0-9.]+)",
        load_commands,
    )
    targets += re.findall(
        r"\bcmd LC_VERSION_MIN_MACOSX\s+cmdsize \d+\s+version ([0-9.]+)",
        load_commands,
    )

    def normalized(value: str) -> tuple[int, ...]:
        parts = [int(part) for part in value.split(".")]
        return tuple([*parts, 0, 0][:3])

    if len(targets) != 1 or normalized(targets[0]) != normalized(minimum_macos):
        raise ValueError(
            f"binary deployment target {targets} does not match {minimum_macos}"
        )
    libraries = run("/usr/bin/otool", "-L", str(binary)).splitlines()[1:]
    if not libraries:
        raise ValueError("otool did not report linked libraries")
    for line in libraries:
        library = line.strip().split(" (", 1)[0]
        if not library.startswith(("/System/Library/", "/usr/lib/")):
            raise ValueError(f"non-system linked library is not bundled: {library}")


def package(
    binary: Path,
    icon: Path,
    output: Path,
    *,
    version: str,
    build_version: str,
    minimum_macos: str,
) -> None:
    """Publish a ZIP only after validation, signing, and archiving succeed."""
    if sys.platform != "darwin":
        raise ValueError(
            "macOS packaging requires a macOS host with Xcode command-line tools"
        )
    for value in (version, build_version, minimum_macos):
        macos.numeric_version(value)
    binary, icon = binary.resolve(), icon.resolve()
    for path in (binary, icon):
        if not path.is_file():
            raise ValueError(f"input is not a regular file: {path}")
    if output.suffix != ".zip":
        raise ValueError("output must have a .zip suffix")
    if output.exists() or output.is_symlink():
        raise ValueError(f"output already exists: {output}")
    validate_binary(binary, minimum_macos)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix=".macos-package-", dir=output.parent
    ) as temp:
        staging = Path(temp)
        # Have Apple's own decoder validate the generated ICNS before shipping.
        run(
            "/usr/bin/iconutil",
            "--convert",
            "iconset",
            "--output",
            str(staging / "validated.iconset"),
            str(icon),
        )
        bundle = staging / "ChessVault.app"
        macos.prepare(
            binary,
            icon,
            bundle,
            version=version,
            build_version=build_version,
            minimum_macos=minimum_macos,
        )
        run("/usr/bin/plutil", "-lint", str(bundle / "Contents/Info.plist"))
        # Ad-hoc signing needs no Apple credentials and provides no publisher trust.
        # Sign only after staging; modifying the bundle afterward invalidates it.
        run(
            "/usr/bin/codesign",
            "--force",
            "--sign",
            "-",
            "--timestamp=none",
            str(bundle),
        )
        run(
            "/usr/bin/codesign",
            "--verify",
            "--deep",
            "--strict",
            "--verbose=2",
            str(bundle),
        )
        archive = staging / "ChessVault.zip"
        run(
            "/usr/bin/ditto",
            "-c",
            "-k",
            "--sequesterRsrc",
            "--keepParent",
            str(bundle),
            str(archive),
        )
        # Verify the archive round trip, not just the pre-archive app.
        extracted = staging / "extracted"
        run("/usr/bin/ditto", "-x", "-k", str(archive), str(extracted))
        run(
            "/usr/bin/codesign",
            "--verify",
            "--deep",
            "--strict",
            "--verbose=2",
            str(extracted / "ChessVault.app"),
        )
        archive.rename(output)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--icon", type=Path, required=True)
    parser.add_argument(
        "--output", type=Path, default=macos.ROOT / "dist/ChessVault-macos-aarch64.zip"
    )
    parser.add_argument("--version", required=True)
    parser.add_argument("--build-version", required=True)
    parser.add_argument("--minimum-macos", required=True)
    args = parser.parse_args()
    try:
        package(
            args.binary,
            args.icon,
            args.output,
            version=args.version,
            build_version=args.build_version,
            minimum_macos=args.minimum_macos,
        )
    except subprocess.CalledProcessError as error:
        parser.exit(1, f"error: {error}\n{error.stdout}{error.stderr}")
    except (OSError, ValueError, subprocess.TimeoutExpired) as error:
        parser.exit(1, f"error: {error}\n")
    print(f"Packaged {args.output}")


if __name__ == "__main__":
    main()
