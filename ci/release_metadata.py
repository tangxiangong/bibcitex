"""Validate release identity before any signing key is loaded."""
import os
import re
import sys


def versions(tag):
    match = re.fullmatch(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", tag)
    if not match:
        raise ValueError("Stable updates require a vMAJOR.MINOR.PATCH tag; prereleases must not replace the stable feeds")
    parts = tuple(map(int, match.groups()))
    if any(number > 65535 for number in parts):
        raise ValueError("MSIX version components must be <= 65535")
    version = ".".join(map(str, parts))
    return version, version + ".0"


if __name__ == "__main__":
    try:
        version, windows_version = versions(os.environ["RELEASE_TAG"])
        with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
            output.write(f"version={version}\nwindows_version={windows_version}\n")
    except (KeyError, ValueError) as error:
        print(f"::error::{error}", file=sys.stderr)
        sys.exit(1)
