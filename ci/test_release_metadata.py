import unittest
from release_metadata import versions


class ReleaseMetadataTests(unittest.TestCase):
    def test_platform_versions(self):
        self.assertEqual(versions("v1.2.34"), ("1.2.34", "1.2.34.0"))
        self.assertEqual(versions("v0.6.0"), ("0.6.0", "0.6.0.0"))

    def test_prereleases_and_noncanonical_versions_never_replace_stable_feed(self):
        for tag in ("v1.0.0-beta.1", "v1.0.0+dev", "1.0.0", "v01.0.0", "v1.2.3\n", "../v1.2.3"):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                versions(tag)

    def test_msix_version_limits(self):
        self.assertEqual(versions("v65535.65535.65535")[1], "65535.65535.65535.0")
        for tag in ("v65536.0.0", "v1.65536.0", "v1.0.65536"):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                versions(tag)
