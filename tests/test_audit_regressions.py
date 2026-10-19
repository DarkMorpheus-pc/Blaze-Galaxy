"""Isolated regressions; never execute privileged operations on the host."""
import ast
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
OVERLAY = ROOT / 'blazeos_custom_apps'


class AuditRegressions(unittest.TestCase):
    def test_installer_starts_with_locale_and_persists_usage_profile(self):
        configs = [
            OVERLAY / 'etc/anaconda/conf.d/99-blazeos.conf',
            OVERLAY / 'etc/anaconda/anaconda.conf',
            OVERLAY / 'etc/anaconda/profile.d/fedora-kde.conf',
            OVERLAY / 'etc/anaconda/profile.d/fedora-workstation.conf',
        ]
        for config in configs:
            self.assertNotIn('    anaconda-screen-language', config.read_text())
        webui = (OVERLAY / 'usr/share/cockpit/anaconda-webui/index.js').read_text()
        self.assertIn('var BZReview=', webui)
        self.assertIn('BZI18N=', webui)
        for language in ('tr', 'en', 'de', 'es', 'ru'):
            self.assertIn(f'{language}:{{title:', webui)
        finalizer = (OVERLAY / 'usr/lib64/python3.15/site-packages/pyanaconda/modules/boss/installation.py').read_text()
        self.assertIn('etc/blazeos-usage-profile', finalizer)

    def test_end4_has_brand_assets_and_runtime_payloads(self):
        vendor = ROOT / 'solarui/vendor/end4-pC'
        self.assertTrue((vendor / 'assets/icons/blazeos-symbolic.svg').is_file())
        self.assertIn('case "blazeos":', (vendor / 'services/SystemInfo.qml').read_text())
        self.assertIn('/usr/share/blazeos/wallpapers/galaxy.png', (vendor / 'services/FirstRunExperience.qml').read_text())
        thumbnail = (vendor / 'modules/common/widgets/ThumbnailImage.qml').read_text()
        self.assertIn('source = sourcePath', thumbnail)
        build = (ROOT / 'build_f45_final.sh').read_text()
        for package in ('matugen', 'grim', 'slurp', 'swappy', 'cava', 'brightnessctl', 'playerctl'):
            self.assertIn(package, build)

    def test_niri_publishes_wayland_environment_for_portals(self):
        config = (OVERLAY / 'etc/solarui/niri.kdl').read_text()
        skel_config = (OVERLAY / 'etc/skel/.config/solarui/niri.kdl').read_text()
        helper = (OVERLAY / 'usr/libexec/blazeos-wayland-session-ready').read_text()
        self.assertIn('spawn-at-startup "/usr/libexec/blazeos-wayland-session-ready"', config)
        self.assertIn('spawn-at-startup "/usr/libexec/blazeos-wayland-session-ready"', skel_config)
        self.assertIn('import-environment WAYLAND_DISPLAY NIRI_SOCKET', helper)
        self.assertIn('restart xdg-desktop-portal.service', helper)

    def test_installer_choices_run_before_oobe_and_display_manager(self):
        service = (OVERLAY / 'etc/systemd/system/blazeos-apply-install-choices.service').read_text()
        helper = (OVERLAY / 'usr/local/bin/blazeos-apply-install-choices').read_text()
        provision_service = (OVERLAY / 'etc/systemd/system/blazeos-desktop-provision.service').read_text()
        provision = (OVERLAY / 'usr/local/bin/blazeos-desktop-provision').read_text()
        self.assertIn('Before=display-manager.service sddm.service blazeos-postinstall.service', service)
        self.assertNotIn('environment install', helper)
        self.assertIn('desktop-provision-pending', helper)
        self.assertIn('blazeos-apply-install-choices.service', provision_service)
        self.assertIn('After=network-online.target local-fs.target display-manager.service', provision_service)
        self.assertNotIn('Before=display-manager.service', provision_service)
        self.assertIn('kde:minimal)', provision)
        self.assertIn('plasma-systemsettings', provision)
        self.assertNotIn('plasma-workspace-wayland', provision)
        self.assertNotIn('kwin-wayland', provision)
        self.assertIn('kde:full)', provision)
        self.assertIn('install_environment kde-desktop-environment', provision)
        self.assertIn('gnome:full)', provision)
        self.assertIn('install_environment workstation-product-environment', provision)
        self.assertIn('gnome-session-wayland-session', provision)
        self.assertIn('upgrade_required gsettings-desktop-schemas gnome-desktop3 gnome-desktop4', provision)
        self.assertIn('XSession=', provision)
        self.assertIn('/var/lib/sddm/state.conf', provision)
        self.assertIn('sway:full) install_environment sway-desktop-environment', provision)
        self.assertIn('/etc/blazeos-selected-session', helper)
        self.assertIn('DONE="$STATE_DIR/install-choices-applied"', helper)

    def test_installer_exposes_profile_and_safe_solarui_removal(self):
        webui = (OVERLAY / 'usr/share/cockpit/anaconda-webui/index.js').read_text()
        provision = (OVERLAY / 'usr/local/bin/blazeos-desktop-provision').read_text()
        remover = (OVERLAY / 'usr/local/bin/blazeos-remove-solarui').read_text()
        self.assertIn('Yalnızca masaüstü ortamı', webui)
        self.assertIn('Tüm masaüstü uygulamaları', webui)
        self.assertIn('SolarUI kaldırılsın mı?', webui)
        self.assertIn('/tmp/anaconda_desktop_profile', webui)
        self.assertIn('/tmp/anaconda_remove_solarui', webui)
        self.assertIn('/tmp/anaconda_usage_profile', webui)
        for profile in ('light', 'daily', 'office', 'gaming', 'standard', 'developer'):
            self.assertIn(f'"{profile}"', webui)
        self.assertNotIn('{id:"hyprland",name:"Hyprland"', webui)
        self.assertIn('/usr/local/bin/blazeos-remove-solarui', provision)
        self.assertIn('Refusing to remove SolarUI without a validated replacement session.', remover)
        self.assertNotIn('/usr/bin/blaze-recovery', remover)

    def test_oobe_uses_anaconda_selected_session(self):
        source = (OVERLAY / 'usr/bin/blaze-setup').read_text()
        self.assertIn('/etc/blazeos-selected-session', source)
        self.assertIn('Session={selected_desktop}', source)
        self.assertIn('Session={selected_session}', source)

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
        block = source.split('# 4. Synchronize', 1)[1].split('# 5.', 1)[0]
        self.assertIn('/etc/blazeos-selected-session', block)
        self.assertIn('s/^Session=.*/Session=${SESSION_NAME}/', block)
        self.assertIn('s/^XSession=.*/XSession=${SESSION_NAME}/', block)
        self.assertIn('s/^SessionType=.*/SessionType=${SESSION_TYPE}/', block)

    def test_blazehouse_update_uses_fixed_offline_transaction(self):
        helper = (OVERLAY / 'usr/libexec/blazeos-update').read_text()
        wrapper = (OVERLAY / 'usr/libexec/blazeos-offline-update').read_text()
        dropin = (OVERLAY / 'etc/systemd/system/dnf5-offline-transaction.service.d/10-blazeos-ui.conf').read_text()
        theme = (OVERLAY / 'usr/share/plymouth/themes/blazeos/blazeos.script').read_text()
        self.assertIn('upgrade --offline -y', helper)
        self.assertIn('offline reboot', helper)
        self.assertNotIn('eval ', helper)
        self.assertNotIn('sh -c', helper)
        self.assertIn('/usr/bin/dnf5 offline _execute', wrapper)
        self.assertIn('dnf_status=${PIPESTATUS[0]}', wrapper)
        self.assertIn('ExecStart=/usr/libexec/blazeos-offline-update', dropin)
        self.assertIn('Plymouth.SetSystemUpdateFunction', theme)
        self.assertIn('Evren güncelleniyor, daha da genişliyor…', theme)
        self.assertIn('Lütfen bilgisayarınızı kapatmayın', theme)
        self.assertIn('update_progress + "%"', theme)

    def test_graphical_polkit_agent_is_pinned_and_started_after_wayland(self):
        helper = (OVERLAY / 'usr/libexec/blazeos-wayland-session-ready').read_text()
        self.assertIn('/usr/libexec/polkit-mate-authentication-agent-1', helper)
        expected = {
            'mate-polkit-1.28.1-9.fc45.x86_64.rpm': '6e16a21f6c0c755a106a009fd84e5388709a9f02f8ea5fb7d424f6a558ddde28',
            'libappindicator-gtk3-12.10.1-11.fc45.x86_64.rpm': '7527f45d311ddadfe23c70567716104d517d55fc61a1240fada633a43564a8a7',
            'libdbusmenu-16.04.0-32.fc45.x86_64.rpm': '53a5856835e3da24812420b2849632a291a6b102362e5aa91f750af75d96a382',
            'libdbusmenu-gtk3-16.04.0-32.fc45.x86_64.rpm': 'fcaf227fbc3c212f1c1b203d504a6fe585eb65740d16f25a0c4087f67b0d0eba',
        }
        for name, digest in expected.items():
            payload = (ROOT / 'packages' / name).read_bytes()
            self.assertEqual(hashlib.sha256(payload).hexdigest(), digest)

    def test_style_center_is_only_offered_after_gnome_or_kde_selection(self):
        provision = (OVERLAY / 'usr/local/bin/blazeos-desktop-provision').read_text()
        launcher = (OVERLAY / 'usr/libexec/blaze-style-center-first-run').read_text()
        autostart = (OVERLAY / 'etc/xdg/autostart/blaze-style-center-first-run.desktop').read_text()
        source = (OVERLAY / 'usr/bin/blaze-style-center').read_text()
        compile(source, 'blaze-style-center', 'exec')
        self.assertIn('kde|gnome) touch "$STATE_DIR/style-center-pending"', provision)
        self.assertNotIn('solarui) touch "$STATE_DIR/style-center-pending"', provision)
        self.assertIn('[ -f /etc/blazeos-setup-completed ]', launcher)
        self.assertIn('X-KDE-autostart-after=panel', autostart)
        self.assertIn('/etc/blazeos-selected-session', launcher)
        self.assertIn('flock -n', launcher)
        self.assertIn('backup_once', source)
        self.assertIn('def restore(', source)
        self.assertIn('plasma-apply-lookandfeel', source)
        self.assertIn('dash-to-dock@micxgx.gmail.com', source)

    def test_appearance_is_integrated_and_dock_uses_session_helper(self):
        control = (OVERLAY / 'usr/local/bin/blazeos-control').read_text()
        style = (OVERLAY / 'usr/bin/blaze-style-center').read_text()
        desktop = (OVERLAY / 'usr/share/applications/blaze-style-center.desktop').read_text()
        license_notice = OVERLAY / 'usr/share/licenses/blazeos-control/PARDUS-GNOME-GREETER-LICENSE'
        compile(control, 'blazeos-control', 'exec')
        compile(style, 'blaze-style-center', 'exec')
        self.assertIn('self.build_appearance_page()', control)
        self.assertIn('BLAZEOS_CONTROL_PAGE', control)
        self.assertIn('def apply_appearance(', control)
        self.assertIn('"set-dock", action', control)
        self.assertNotIn('dash-to-dock@vswitch.com', control)
        self.assertIn('def command_line(', style)
        self.assertIn('NoDisplay=true', desktop)
        launcher = (OVERLAY / 'usr/libexec/blaze-style-center-first-run').read_text()
        self.assertIn('exec /usr/local/bin/blazeos-control', launcher)
        self.assertIn('pardus/pardus-gnome-greeter', license_notice.read_text())

    def test_core_image_trim_keeps_profiles_explicit(self):
        build = (ROOT / 'build_f45_final.sh').read_text()
        provision = (OVERLAY / 'usr/local/bin/blazeos-desktop-provision').read_text()
        self.assertIn('Trimming Workstation host payload', build)
        self.assertIn('libreoffice($|-)', build)
        self.assertIn('qemu-guest-agent', build)  # retained by the documented exclusion
        standard = provision.split('    standard)', 1)[1].split('    developer)', 1)[0]
        office = provision.split('    office)', 1)[1].split('    gaming)', 1)[0]
        self.assertNotIn('libreoffice', standard)
        self.assertIn('libreoffice-writer', office)
        preset = (OVERLAY / 'etc/systemd/system-preset/00-blazeos.preset').read_text()
        self.assertIn('disable NetworkManager-wait-online.service', preset)
        self.assertIn('disable avahi-daemon.service', preset)

    def test_desktop_provision_and_end4_use_fedora45_abi(self):
        provision = (OVERLAY / 'usr/local/bin/blazeos-desktop-provision').read_text()
        provider = (ROOT / 'solarui/crates/solar-shell/src/providers/end4_pc.rs').read_text()
        final_build = (ROOT / 'build_f45_final.sh').read_text()
        offline_build = (ROOT / 'build_f45_offline.sh').read_text()
        self.assertNotIn('--setopt=updates-testing-modular', provision)
        self.assertIn('DNF_STABLE=(--setopt=updates-testing.enabled=0)', provision)
        self.assertIn('/usr/bin/quickshell-fedora', provider)
        for build in (final_build, offline_build):
            self.assertNotIn('cp -a --no-preserve=ownership /usr/lib64/qt6/qml/Qt5Compat', build)
            self.assertIn('quickshell-fedora', build)
            self.assertIn('Qt_6.11_PRIVATE_API', build)
        expected = {
            'qt6-qt5compat-6.11.2-1.fc45.x86_64.rpm': '36677b8a807c93e12b0fb1ee9036f925bede904938ad7d7e8d9bb241b945e068',
            'quickshell-0.2.1_git20260209.dacfa9d-5.fc45.x86_64.rpm': '3f926860e9495029953a47b193428b98ccb2b339428ee18c37a4450edb0e3536',
            'jemalloc-5.3.0-14.fc44.x86_64.rpm': '3159390bdbcf46c3269fd499855e95421f92b3b6a155b68d5b641fb6cc7f7a88',
        }
        for name, digest in expected.items():
            payload = (ROOT / 'packages' / name).read_bytes()
            self.assertEqual(hashlib.sha256(payload).hexdigest(), digest)


if __name__ == '__main__':
    unittest.main()
