"""Check that generated update metadata refers to the actual release assets."""
import pathlib
import sys
import urllib.parse
import xml.etree.ElementTree as ET


def verify(platform, directory, version, repository, tag):
    directory = pathlib.Path(directory)
    if platform == "macos":
        feeds = list(directory.glob("appcast-*.xml"))
        assert len(feeds) == 1, "Expected one architecture-specific appcast"
        root = ET.parse(feeds[0]).getroot()
        items = root.findall("./channel/item")
        assert len(items) == 1, "Expected one current release"
        item = items[0]
        sparkle = "{http://www.andymatuschak.org/xml-namespaces/sparkle}"
        enclosure = item.find("enclosure")
        assert enclosure is not None, "Missing enclosure"
        assert item.findtext(sparkle + "shortVersionString") == version or enclosure.get(sparkle + "shortVersionString") == version, "App version mismatch"
        assert enclosure.get(sparkle + "edSignature"), "Unsigned Sparkle update"
        url = enclosure.attrib["url"]
        assert url.startswith(f"https://github.com/{repository}/releases/download/{tag}/"), "Unexpected archive URL"
        asset = directory / urllib.parse.unquote(urllib.parse.urlparse(url).path.rsplit("/", 1)[-1])
        assert asset.is_file() and asset.stat().st_size == int(enclosure.attrib["length"]), "Archive size mismatch"
    elif platform == "windows":
        feeds = list(directory.glob("*.appinstaller"))
        assert len(feeds) == 1, "Expected one App Installer file"
        root = ET.parse(feeds[0]).getroot()
        ns = "{http://schemas.microsoft.com/appx/appinstaller/2018}"
        package = root.find(ns + "MainPackage")
        assert package is not None and package.attrib["Version"] == version, "MSIX version mismatch"
        base = f"https://github.com/{repository}/releases/latest/download/"
        assert root.attrib["Uri"] == base + feeds[0].name, "Unexpected App Installer URL"
        assert package.attrib["Uri"].startswith(base), "Unexpected MSIX URL"
        asset = directory / urllib.parse.unquote(package.attrib["Uri"].rsplit("/", 1)[-1])
        assert asset.is_file() and asset.suffix == ".msix", "Missing MSIX payload"
    else:
        raise ValueError("Unknown platform")


if __name__ == "__main__":
    verify(*sys.argv[1:])
