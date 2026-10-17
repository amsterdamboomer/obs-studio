"""Coalescing must retain work from superseded master pushes."""
import subprocess
import unittest
from unittest.mock import Mock
from ci.master_scope_base import scope_base

class MasterScopeBaseTest(unittest.TestCase):
    def test_uses_successful_ancestor_instead_of_immediate_push_parent(self):
        sha = 'a' * 40
        run = Mock(side_effect=[subprocess.CompletedProcess([], 0, '[{"headSha":"'+sha+'"}]'), subprocess.CompletedProcess([], 0)])
        self.assertEqual(scope_base(run), sha)
        self.assertIn('--status', run.call_args_list[0].args[0])
        self.assertIn('success', run.call_args_list[0].args[0])

    def test_nonancestor_is_ignored_and_empty_tree_is_safe_fallback(self):
        run = Mock(side_effect=[subprocess.CompletedProcess([], 0, '[{"headSha":"'+'b'*40+'"}]'), subprocess.CompletedProcess([], 1), subprocess.CompletedProcess([], 0, 'empty-tree\n')])
        self.assertEqual(scope_base(run), 'empty-tree')
        self.assertIn('hash-object', run.call_args.args[0])

    def test_api_failure_uses_entire_tree(self):
        run = Mock(side_effect=[subprocess.CalledProcessError(1, 'gh'), subprocess.CompletedProcess([], 0, 'empty-tree\n')])
        self.assertEqual(scope_base(run), 'empty-tree')

    def test_master_workflow_uses_catchup_scope(self):
        from pathlib import Path
        text = Path('.github/workflows/build-project.yaml').read_text()
        self.assertIn('base="$(uv run ci/master_scope_base.py)"', text)
        self.assertIn('GH_TOKEN: ${{ github.token }}', text)

if __name__ == '__main__':
    unittest.main()
