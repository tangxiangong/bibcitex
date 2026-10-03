import tempfile
import unittest
from pathlib import Path
from verify_release import verify


class UpdateFeedTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)

    def mac_feed(self, signature='signed', size='3', version='0.6.0'):
        (self.directory / 'BibCiTeX.zip').write_bytes(b'zip')
        (self.directory / 'appcast-arm64.xml').write_text(f'''<rss xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle"><channel><item><sparkle:shortVersionString>{version}</sparkle:shortVersionString><enclosure url="https://github.com/owner/repo/releases/download/v0.6.0/BibCiTeX.zip" length="{size}" sparkle:edSignature="{signature}" /></item></channel></rss>''')

    def test_signed_archive_metadata(self):
        self.mac_feed()
        verify('macos', self.directory, '0.6.0', 'owner/repo', 'v0.6.0')

    def test_unsigned_archive_rejected(self):
        self.mac_feed(signature='')
        with self.assertRaisesRegex(AssertionError, 'Unsigned'):
            verify('macos', self.directory, '0.6.0', 'owner/repo', 'v0.6.0')

    def test_wrong_archive_size_rejected(self):
        self.mac_feed(size='99')
        with self.assertRaisesRegex(AssertionError, 'size mismatch'):
            verify('macos', self.directory, '0.6.0', 'owner/repo', 'v0.6.0')

    def test_wrong_version_rejected(self):
        self.mac_feed(version='0.5.0')
        with self.assertRaisesRegex(AssertionError, 'version mismatch'):
            verify('macos', self.directory, '0.6.0', 'owner/repo', 'v0.6.0')

    def windows_feed(self):
        (self.directory / 'BibCiTeX-x64.msix').write_bytes(b'msix')
        (self.directory / 'BibCiTeX-x64.appinstaller').write_text('''<AppInstaller xmlns="http://schemas.microsoft.com/appx/appinstaller/2018" Uri="https://github.com/owner/repo/releases/latest/download/BibCiTeX-x64.appinstaller"><MainPackage Version="0.6.0.0" Uri="https://github.com/owner/repo/releases/latest/download/BibCiTeX-x64.msix" /></AppInstaller>''')

    def test_app_installer_resolves_payload(self):
        self.windows_feed()
        verify('windows', self.directory, '0.6.0.0', 'owner/repo', 'v0.6.0')

    def test_app_installer_missing_payload_rejected(self):
        self.windows_feed()
        (self.directory / 'BibCiTeX-x64.msix').unlink()
        with self.assertRaisesRegex(AssertionError, 'Missing MSIX'):
            verify('windows', self.directory, '0.6.0.0', 'owner/repo', 'v0.6.0')
