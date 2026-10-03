import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import publish_release


class PublishTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        directory = Path(self.temp.name)
        for name in ('BibCiTeX-0.6.0-macos-arm64.zip', 'BibCiTeX-0.6.0-macos-x86_64.zip',
                     'appcast-arm64.xml', 'appcast-x86_64.xml', 'BibCiTeX-x64.appinstaller',
                     'BibCiTeX-arm64.appinstaller', 'BibCiTeX-x64.msix', 'BibCiTeX-arm64.msix'):
            (directory / name).write_text('asset')
        self.environment = patch.dict(os.environ, RELEASE_TAG='v0.6.0', RELEASE_ASSETS=self.temp.name, GITHUB_REPOSITORY='owner/repo')
        self.environment.start()
        self.addCleanup(self.environment.stop)

    def test_retry_draft_only_publishes_after_upload(self):
        with patch.object(publish_release, 'gh', side_effect=[json.dumps({'tag_name': 'v0.5.0'}), json.dumps({'isDraft': True, 'isPrerelease': False}), '', '']) as call:
            publish_release.main()
            self.assertEqual(call.call_args_list[-2].args[:2], ('release', 'upload'))
            self.assertEqual(call.call_args_list[-1].args[:2], ('release', 'edit'))

    def test_upload_failure_preserves_existing_stable_release(self):
        with patch.object(publish_release, 'gh', side_effect=[json.dumps({'tag_name': 'v0.5.0'}), json.dumps({'isDraft': True, 'isPrerelease': False}), subprocess.CalledProcessError(1, 'gh', stderr='upload failed')]) as call:
            with self.assertRaises(subprocess.CalledProcessError):
                publish_release.main()
            self.assertFalse(any(item.args[:2] == ('release', 'edit') for item in call.call_args_list))

    def test_downgrade_never_creates_or_uploads_a_release(self):
        with patch.object(publish_release, 'gh', return_value=json.dumps({'tag_name': 'v0.7.0'})) as call:
            with self.assertRaisesRegex(ValueError, 'newer'):
                publish_release.main()
            self.assertEqual(call.call_count, 1)

    def test_published_release_cannot_be_overwritten(self):
        with patch.object(publish_release, 'gh', side_effect=[json.dumps({'tag_name': 'v0.5.0'}), json.dumps({'isDraft': False, 'isPrerelease': False})]) as call:
            with self.assertRaisesRegex(ValueError, 'already published'):
                publish_release.main()
            self.assertEqual(call.call_count, 2)
