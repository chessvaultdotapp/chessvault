"""Stage the ChessVault binary, desktop entry, and icon without building or archiving."""

import argparse
import shutil
import tempfile
from pathlib import Path

CHESSVAULT_ROOT = Path(__file__).resolve().parents[2]
DESKTOP_ENTRY = CHESSVAULT_ROOT / "apps/chessvault/data/chessvault.desktop"


def prepare(binary: Path, output: Path) -> None:
    """Stage the binary (0755), desktop entry and sibling chessvault.png (0644).

    Existing output is rejected rather than mixing stale files into a release.
    Copy into a temporary sibling first so copy failures leave no partial package.
    """
    if not binary.is_file():
        raise ValueError(f"binary is not a regular file: {binary}")
    icon = binary.with_name("chessvault.png")
    if not icon.is_file():
        raise ValueError(f"icon is not a regular file: {icon}")
    if output.exists() or output.is_symlink():
        raise ValueError(f"output already exists: {output}")

    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(
        prefix=".dist-prepare-", dir=output.parent
    ) as temp:
        package = Path(temp) / "package"
        destination = package / "bin" / "chessvault"
        destination.parent.mkdir(parents=True)
        shutil.copyfile(binary, destination)
        destination.chmod(0o755)
        desktop = package / "share/applications/chessvault.desktop"
        desktop.parent.mkdir(parents=True)
        shutil.copyfile(DESKTOP_ENTRY, desktop)
        desktop.chmod(0o644)
        staged_icon = package / "share/icons/hicolor/256x256/apps/chessvault.png"
        staged_icon.parent.mkdir(parents=True)
        shutil.copyfile(icon, staged_icon)
        staged_icon.chmod(0o644)
        package.rename(output)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "binary",
        type=Path,
        help="path to the built ChessVault binary (with chessvault.png beside it)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=CHESSVAULT_ROOT / "dist" / "package",
        help="new staging directory (default: <repository>/dist/package)",
    )
    args = parser.parse_args()
    try:
        prepare(args.binary, args.output)
    except (OSError, ValueError) as error:
        parser.exit(1, f"error: {error}\n")
    print(f"Prepared {args.output}")


if __name__ == "__main__":
    main()
