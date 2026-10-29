"""Stage an unsigned macOS ChessVault.app without building, signing, or archiving."""

import argparse
import plistlib
import re
import shutil
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUNDLE_ID = "app.chessvault.ChessVault"


def numeric_version(value: str) -> str:
    """Accept one to three numeric components, not a Git tag or SemVer suffix."""
    if not re.fullmatch(r"[0-9]+(?:\.[0-9]+){0,2}", value):
        raise ValueError(f"expected one to three numeric version components: {value!r}")
    return value


def prepare(
    binary: Path,
    icon: Path,
    output: Path,
    *,
    version: str,
    build_version: str,
    minimum_macos: str,
) -> None:
    """Atomically stage a fresh bundle; callers supply a native binary and ICNS.

    This is a layout tool, not a Mach-O/linkage validator. It can run on Linux
    for tests. The deployment target must match the target used to build binary.
    """
    for path in (binary, icon):
        if not path.is_file():
            raise ValueError(f"input is not a regular file: {path}")
    if output.suffix != ".app":
        raise ValueError("output must have an .app suffix")
    if output.exists() or output.is_symlink():
        raise ValueError(f"output already exists: {output}")
    metadata = {
        "CFBundleDevelopmentRegion": "en",
        "CFBundleDisplayName": "ChessVault",
        "CFBundleExecutable": "chessvault",
        "CFBundleIconFile": "chessvault.icns",
        "CFBundleIdentifier": BUNDLE_ID,
        "CFBundleInfoDictionaryVersion": "6.0",
        "CFBundleName": "ChessVault",
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": numeric_version(version),
        "CFBundleVersion": numeric_version(build_version),
        "LSMinimumSystemVersion": numeric_version(minimum_macos),
        "NSHighResolutionCapable": True,
        "LSApplicationCategoryType": "public.app-category.board-games",
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix=".macos-prepare-", dir=output.parent
    ) as temp:
        bundle = Path(temp) / "ChessVault.app"
        contents = bundle / "Contents"
        executable = contents / "MacOS/chessvault"
        executable.parent.mkdir(parents=True)
        resources = contents / "Resources"
        resources.mkdir()
        shutil.copyfile(binary, executable)
        executable.chmod(0o755)
        for source, destination in (
            (icon, resources / "chessvault.icns"),
            (ROOT / "LICENSE", resources / "LICENSE"),
        ):
            shutil.copyfile(source, destination)
            destination.chmod(0o644)
        plist = contents / "Info.plist"
        plist.write_bytes(plistlib.dumps(metadata))
        plist.chmod(0o644)
        # Do not inherit restrictive umask permissions for bundle directories.
        for directory in (bundle, contents, executable.parent, resources):
            directory.chmod(0o755)
        bundle.rename(output)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--icon", type=Path, required=True, help="prepared .icns icon")
    parser.add_argument("--output", type=Path, default=ROOT / "dist/ChessVault.app")
    parser.add_argument(
        "--version", required=True, help="numeric display version, e.g. 0.1.0"
    )
    parser.add_argument(
        "--build-version", required=True, help="numeric bundle build version"
    )
    parser.add_argument(
        "--minimum-macos", required=True, help="binary deployment target"
    )
    args = parser.parse_args()
    try:
        prepare(
            args.binary,
            args.icon,
            args.output,
            version=args.version,
            build_version=args.build_version,
            minimum_macos=args.minimum_macos,
        )
    except (OSError, ValueError) as error:
        parser.exit(1, f"error: {error}\n")
    print(f"Prepared {args.output}")


if __name__ == "__main__":
    main()
