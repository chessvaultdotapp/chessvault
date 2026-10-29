"""Reference subprocess: its environment belongs exclusively to one test case."""

import json
import os
import sys

import platformdirs
from platformdirs.api import PlatformDirsABC
from platformdirs.macos import MacOS
from platformdirs.unix import Unix


def main() -> None:
    request = json.load(sys.stdin)

    def configured(opinion: bool) -> PlatformDirsABC:
        implementation = MacOS if sys.platform == "darwin" else Unix
        return implementation(
            appname=request["app"],
            appauthor=False,
            version=None,
            multipath=True,
            ensure_exists=False,
            use_site_for_root=False,
            opinion=opinion,
        )

    try:
        dirs = configured(True)
        unopinionated = configured(False)
    except ValueError as error:
        print(
            json.dumps(
                {
                    "version": platformdirs.__version__,
                    "outputs": {
                        key: {"error": str(error)}
                        for key in (
                            "state",
                            "data",
                            "site",
                            "log_default",
                            "log_true",
                            "log_false",
                        )
                    },
                }
            )
        )
        return
    outputs = {
        "state": [dirs.user_state_dir],
        "data": [dirs.user_data_dir],
        "site": dirs.site_data_dir.split(os.pathsep),
        "log_default": [dirs.user_log_dir],
        "log_true": [dirs.user_log_dir],
        "log_false": [unopinionated.user_log_dir],
    }
    print(
        json.dumps(
            {
                "version": platformdirs.__version__,
                "outputs": {key: {"paths": value} for key, value in outputs.items()},
            }
        )
    )


if __name__ == "__main__":
    main()
