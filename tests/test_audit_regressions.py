"""Isolated regressions; never execute privileged operations on the host."""
import ast
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
OVERLAY = ROOT / 'blazeos_custom_apps'


class AuditRegressions(unittest.TestCase):
    def test_live_boot_exits_before_account_or_gdm_mutations(self):
        source = (OVERLAY / 'usr/local/bin/blazeos-postinstall').read_text()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'cmdline').write_text('quiet root=live:CDLABEL=Blaze-SE-5 rd.live.image\n')
            # Only the read-only live-boot input is replaced. A successful early
            # exit must happen before the first log redirect or system mutation.
            source = source.split('LOG="/var/log/blazeos-postinstall.log"', 1)[0]
            source += '\necho "UNSAFE_SETUP_REACHED"; exit 99\n'
            source = source.replace('/proc/cmdline', str(root / 'cmdline'))
            result = subprocess.run(['bash', '-c', source], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('installed-system setup skipped', result.stdout)

    def test_missing_nvidia_module_does_not_retire_nouveau(self):
        source = (OVERLAY / 'usr/local/bin/blazeos-nvidia-setup').read_text()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            bindir = root / 'bin'
            bindir.mkdir()
            stubs = {
                'lspci': 'echo "VGA NVIDIA"', 'rpm': 'echo 45',
                'dnf': 'exit 0', 'seq': 'echo 1', 'sleep': 'exit 0',
                'modinfo': 'exit 1',
                'grubby': f'echo touched >> "{root}/mutations"',
                'dracut': f'echo touched >> "{root}/mutations"',
            }
            for name, body in stubs.items():
                p = bindir / name
                p.write_text('#!/bin/sh\n' + body + '\n')
                p.chmod(0o755)
            for old, new in [('/var/lib/blazeos', str(root / 'state')),
                             ('/var/log/blazeos-nvidia-setup.log', str(root / 'log')),
                             ('/etc/modprobe.d', str(root / 'modprobe'))]:
                source = source.replace(old, new)
            result = subprocess.run(['bash', '-c', source], env={**os.environ, 'PATH': f'{bindir}:/usr/bin:/bin'}, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1, result.stderr)
            self.assertFalse((root / 'state/nvidia-setup-done').exists())
            self.assertFalse((root / 'mutations').exists())
            self.assertFalse((root / 'modprobe/blazeos-nouveau-blacklist.conf').exists())

    def test_control_log_is_bounded(self):
        tree = ast.parse((OVERLAY / 'usr/local/bin/blazeos-control').read_text())
        cls = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == 'BlazeControlWindow')
        method = next(n for n in cls.body if isinstance(n, ast.FunctionDef) and n.name == 'append_log')
        ns = {}
        exec(compile(ast.Module(body=[method], type_ignores=[]), '<append_log>', 'exec'), ns)

        class Buffer:
            text = ''
            def insert(self, _, text): self.text += text
            def get_end_iter(self): return len(self.text)
            def get_start_iter(self): return 0
            def get_char_count(self): return len(self.text)
            def get_iter_at_offset(self, offset): return offset
            def delete(self, start, end): self.text = self.text[:start] + self.text[end:]

        obj = type('Window', (), {'log_buffer': Buffer()})()
        for i in range(100):
            ns['append_log'](obj, 'x' * 4096)
        ns['append_log'](obj, 'sonuç')
        self.assertEqual(len(obj.log_buffer.text), 200_000)
        self.assertTrue(obj.log_buffer.text.endswith('sonuç'))

    def test_session_sync_replaces_all_session_keys(self):
        source = (OVERLAY / 'usr/local/bin/blazeos-postinstall').read_text()
        block = source.split('# Synchronize GDM', 1)[1].split('# 5.', 1)[0]
        block = '# Synchronize GDM' + block
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            for name in ['etc/gdm', 'usr/share/wayland-sessions', 'var/lib/AccountsService/users']:
                (root / name).mkdir(parents=True)
            (root / 'etc/gdm/custom.conf').write_text('[daemon]\nDefaultSession=gnome.desktop\n')
            (root / 'usr/share/wayland-sessions/gnome.desktop').touch()
            (root / 'etc/passwd').write_text('demo:x:1000:1000::/home/demo:/bin/bash\n')
            account = root / 'var/lib/AccountsService/users/demo'
            account.write_text('[User]\nSession=solarui\nXSession=solarui\nSessionType=wayland\n')
            for path in ['/etc/gdm', '/usr/share/wayland-sessions', '/usr/share/xsessions', '/var/lib/AccountsService/users', '/etc/passwd']:
                block = block.replace(path, str(root) + path)
            block = block.replace('restorecon -F "$ACCOUNT"', 'true')
            result = subprocess.run(['bash', '-e', '-c', block], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(account.read_text(), '[User]\nSession=gnome\nXSession=gnome\nSessionType=wayland\n')


if __name__ == '__main__':
    unittest.main()
