"""Publish complete platform feeds together; never overwrite published releases."""
import json
import os
from pathlib import Path
import subprocess
from release_metadata import versions


def gh(*arguments):
    return subprocess.run(["gh", *arguments], text=True, capture_output=True, check=True).stdout


def main():
    tag = os.environ["RELEASE_TAG"]
    version, _ = versions(tag)
    directory = Path(os.environ["RELEASE_ASSETS"])
    assets = sorted(directory.iterdir())
    expected = {
        f"BibCiTeX-{version}-macos-arm64.zip",
        f"BibCiTeX-{version}-macos-x86_64.zip",
        "appcast-arm64.xml", "appcast-x86_64.xml",
        "BibCiTeX-x64.appinstaller", "BibCiTeX-arm64.appinstaller",
    }
    names = {path.name for path in assets}
    assert expected <= names and len(assets) == 8, "Missing or unexpected release artifacts"
    assert len([path for path in assets if path.suffix == ".msix"]) == 2, "Missing MSIX architectures"
    assert all(path.is_file() and path.stat().st_size for path in assets), "Empty release artifact"
    # Do not advance /latest/download backwards when an old tag is rebuilt.
    repository = os.environ["GITHUB_REPOSITORY"]
    try:
        latest = json.loads(gh("api", f"repos/{repository}/releases/latest"))
    except subprocess.CalledProcessError as error:
        if "404" not in error.stderr:
            raise
    else:
        try:
            latest_version, _ = versions(latest["tag_name"])
        except ValueError:
            # Historical releases may use another tag convention.
            latest_version = None
        if latest_version is not None and tuple(map(int, version.split('.'))) <= tuple(map(int, latest_version.split('.'))):
            raise ValueError("The stable release must be newer than the currently published stable version")
    try:
        release = json.loads(gh("release", "view", tag, "--json", "isDraft,isPrerelease"))
    except subprocess.CalledProcessError as error:
        if "release not found" not in error.stderr.lower() and "404" not in error.stderr:
            raise
        gh("release", "create", tag, "--verify-tag", "--draft", "--title", tag, "--generate-notes")
    else:
        if not release["isDraft"] or release["isPrerelease"]:
            raise ValueError("Refusing to replace an already published or prerelease release")
    # A failed upload leaves a draft, so all existing stable feeds stay intact.
    gh("release", "upload", tag, *map(str, assets), "--clobber")
    gh("release", "edit", tag, "--draft=false", "--prerelease=false", "--latest")


if __name__ == "__main__":
    main()
