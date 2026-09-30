#!/usr/bin/env python3
import os
import sys
import time
import socket
import subprocess
import json

ISO_PATH = "/home/darkmorpheus/BlazeFedora/Blaze-SolarEvolution-5-x86_64.iso"
DISK_PATH = "/tmp/blaze_test_disk.qcow2"
QMP_PATH = "/tmp/qemu-qmp-dogfood.sock"
SCREEN_PPM = "/tmp/qemu_screen_qmp.ppm"
BRAIN_DIR = "/home/darkmorpheus/.gemini/antigravity/brain/cafdcfba-10e8-4d85-b938-c1121a5b7023"

class QmpClient:
    def __init__(self, sock_path):
        self.sock_path = sock_path
        self.sock = None

    def connect(self):
        for _ in range(40):
            if os.path.exists(self.sock_path):
                break
            time.sleep(0.5)
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(self.sock_path)
        time.sleep(0.1)
        self.sock.recv(4096) # banner
        self.send({"execute": "qmp_capabilities"})

    def send(self, obj):
        data = json.dumps(obj) + "\n"
        self.sock.sendall(data.encode('utf-8'))
        time.sleep(0.05)
        res = b""
        while True:
            chunk = self.sock.recv(4096)
            res += chunk
            if b"\n" in chunk:
                break
        return res.decode('utf-8', errors='ignore')

    def move_abs(self, x, y, w=1280, h=800):
        nx = int(x / w * 32767)
        ny = int(y / h * 32767)
        self.send({
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    {"type": "abs", "data": {"axis": "x", "value": nx}},
                    {"type": "abs", "data": {"axis": "y", "value": ny}}
                ]
            }
        })
        time.sleep(0.1)

    def click(self, x, y, w=1280, h=800):
        self.move_abs(x, y, w, h)
        self.send({
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    {"type": "btn", "data": {"button": "left", "down": True}}
                ]
            }
        })
        time.sleep(0.1)
        self.send({
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    {"type": "btn", "data": {"button": "left", "down": False}}
                ]
            }
        })
        time.sleep(0.3)

    def send_key(self, qcode):
        self.send({
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    {"type": "key", "data": {"key": {"type": "qcode", "data": qcode}, "down": True}},
                    {"type": "key", "data": {"key": {"type": "qcode", "data": qcode}, "down": False}}
                ]
            }
        })
        time.sleep(0.2)

    def type_text(self, text):
        for ch in text:
            if ch == ' ':
                self.send_key('spc')
            elif ch == '\n':
                self.send_key('ret')
            elif ch == '\t':
                self.send_key('tab')
            else:
                self.send_key(ch.lower())
            time.sleep(0.08)

    def send_combo(self, mod, key):
        self.send({
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    {"type": "key", "data": {"key": {"type": "qcode", "data": mod}, "down": True}},
                    {"type": "key", "data": {"key": {"type": "qcode", "data": key}, "down": True}},
                    {"type": "key", "data": {"key": {"type": "qcode", "data": key}, "down": False}},
                    {"type": "key", "data": {"key": {"type": "qcode", "data": mod}, "down": False}}
                ]
            }
        })
        time.sleep(0.3)

    def screenshot(self, name):
        if os.path.exists(SCREEN_PPM):
            try:
                os.remove(SCREEN_PPM)
            except OSError:
                pass
        self.send({
            "execute": "screendump",
            "arguments": {"filename": SCREEN_PPM}
        })
        time.sleep(0.5)
        out_png = os.path.join(BRAIN_DIR, f"{name}.png")
        if os.path.exists(SCREEN_PPM):
            subprocess.run(["ffmpeg", "-y", "-i", SCREEN_PPM, out_png], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            print(f"[Captured Screenshot] {out_png}")
            return out_png
        else:
            print(f"[Error] Failed to capture {SCREEN_PPM}")
            return None

    def close(self):
        try:
            self.send({"execute": "quit"})
            self.sock.close()
        except Exception:
            pass

def main():
    print("=== BlazeOS Production Dogfooding & Anaconda Test with QMP ===")

    if not os.path.exists(DISK_PATH):
        subprocess.run(["qemu-img", "create", "-f", "qcow2", DISK_PATH, "30G"], check=True)

    if os.path.exists(QMP_PATH):
        try:
            os.remove(QMP_PATH)
        except OSError:
            pass

    qemu_cmd = [
        "qemu-system-x86_64",
        "-enable-kvm",
        "-m", "6G",
        "-smp", "4",
        "-cpu", "host",
        "-vga", "virtio",
        "-device", "virtio-tablet-pci",
        "-netdev", "user,id=net0",
        "-device", "virtio-net-pci,netdev=net0",
        "-drive", f"file={DISK_PATH},if=virtio,format=qcow2",
        "-cdrom", ISO_PATH,
        "-display", "none",
        "-qmp", f"unix:{QMP_PATH},server,nowait"
    ]
    proc = subprocess.Popen(qemu_cmd)

    client = QmpClient(QMP_PATH)
    client.connect()

    print("Booting ISO...")
    time.sleep(2)
    client.send_key("ret")

    # Wait 45s for desktop load
    print("Waiting 45s for Live Desktop & Welcome HUD...")
    time.sleep(45)

    client.screenshot("qmp_01_desktop_ready")

    # 1. Dismiss Welcome HUD by pressing Return (start_btn was focused!)
    print("\n--- 1. Dismissing Welcome HUD ---")
    client.send_key("ret")
    time.sleep(2)
    client.screenshot("qmp_02_welcome_closed_anaconda")

    # If welcome is still there for any reason, click Masaüstüne Başla (868, 720)
    client.click(868, 720)
    time.sleep(1)

    # 2. Test Anaconda Installer 7 Steps
    print("\n--- 2. Testing Anaconda Installer Steps ---")
    # Step 1: Click Sonraki (412, 745)
    print("Step 1 (Language/Welcome) -> Clicking Sonraki (412, 745)...")
    client.click(412, 745)
    time.sleep(3)
    client.screenshot("qmp_03_anaconda_step2_datetime")

    # Step 2: Click Sonraki (412, 745)
    print("Step 2 (Date/Time) -> Clicking Sonraki (412, 745)...")
    client.click(412, 745)
    time.sleep(3)
    client.screenshot("qmp_04_anaconda_step3_de_bootloader")

    # Step 3: System Preferences (DE & Bootloader) -> Click Sonraki (412, 745)
    print("Step 3 (DE & Bootloader) -> Clicking Sonraki (412, 745)...")
    client.click(412, 745)
    time.sleep(3)
    client.screenshot("qmp_05_anaconda_step4_install_method")

    # Step 4: Installation Method -> Click Sonraki (412, 745)
    print("Step 4 (Install Method) -> Clicking Sonraki (412, 745)...")
    client.click(412, 745)
    time.sleep(3)
    client.screenshot("qmp_06_anaconda_step5_storage")

    # Step 5: Storage Configuration -> Select 30GB disk card, then click Sonraki
    print("Step 5 (Storage Configuration) -> Selecting disk & clicking Sonraki...")
    client.click(500, 350)
    time.sleep(1)
    client.click(412, 745)
    time.sleep(3)
    client.screenshot("qmp_07_anaconda_step6_account")

    # Step 6: User Account -> Enter credentials
    print("Step 6 (Account) -> Entering user credentials...")
    client.click(450, 350)
    time.sleep(0.3)
    client.type_text("blaze")
    client.click(450, 428)
    time.sleep(0.3)
    client.type_text("blaze")
    client.click(450, 512)
    time.sleep(0.3)
    client.type_text("blaze123")
    client.click(450, 645)
    time.sleep(0.3)
    client.type_text("blaze123")
    time.sleep(1)
    client.screenshot("qmp_07b_anaconda_step6_account_filled")

    print("Step 6 (Account) -> Clicking Sonraki (412, 745)...")
    client.click(412, 745)
    time.sleep(4)
    client.screenshot("qmp_08_anaconda_step7_review")

    # Close Anaconda installer window
    print("Closing Anaconda installer (Mod + Q)...")
    client.send_combo("meta_l", "q")
    time.sleep(2)

    # 3. Test GameZone Fullscreen Shell & Sidebar Navigation
    print("\n--- 3. Testing Blaze GameZone ---")
    client.send_combo("meta_l", "g")
    time.sleep(3)
    client.screenshot("qmp_09_gamezone_home")

    # Click Steam Kütüphanesi (110, 290)
    print("Clicking Steam Kütüphanesi in sidebar...")
    client.click(110, 290)
    time.sleep(2)
    client.screenshot("qmp_10_gamezone_steam_filtered")

    # Click Epic Games (Heroic) (110, 340)
    print("Clicking Epic Games (Heroic)...")
    client.click(110, 340)
    time.sleep(2)
    client.screenshot("qmp_11_gamezone_epic_filtered")

    # Click GOG Galaxy (110, 390)
    print("Clicking GOG Galaxy...")
    client.click(110, 390)
    time.sleep(2)
    client.screenshot("qmp_12_gamezone_gog_filtered")

    # Click Retro Konsol (110, 440)
    print("Clicking Retro Konsol...")
    client.click(110, 440)
    time.sleep(2)
    client.screenshot("qmp_13_gamezone_retro_filtered")

    # Click Tüm Oyunlar (110, 240)
    print("Clicking Tüm Oyunlar (reset to top)...")
    client.click(110, 240)
    time.sleep(2)
    client.screenshot("qmp_14_gamezone_all_games")

    # Close GameZone
    client.send_key("esc")
    time.sleep(2)

    # 4. Test Omnibar (Mod + Space)
    print("\n--- 4. Testing Solar Omnibar ---")
    client.send_combo("meta_l", "spc")
    time.sleep(2)
    client.screenshot("qmp_15_omnibar_open")
    client.send_key("esc")
    time.sleep(1)

    # 5. Test Everyday Desktop Applications
    # 5.1 Terminal (Mod + Return) & Network Verification
    print("\n--- 5. Testing Terminal & Network Connectivity ---")
    client.send_combo("meta_l", "ret")
    time.sleep(3)
    client.type_text("ip a\n")
    time.sleep(1)
    client.type_text("ping -c 3 1.1.1.1\n")
    time.sleep(4)
    client.screenshot("qmp_18_terminal_network")
    client.send_combo("meta_l", "q")
    time.sleep(2)

    # 5.2 Settings (Mod + I)
    print("\n--- 6. Testing SolarUI Settings ---")
    client.send_combo("meta_l", "i")
    time.sleep(3)
    client.screenshot("qmp_19_settings_open")
    client.send_combo("meta_l", "q")
    time.sleep(2)

    # 5.3 File Manager (Mod + E)
    print("\n--- 7. Testing File Manager ---")
    client.send_combo("meta_l", "e")
    time.sleep(3)
    client.screenshot("qmp_20_file_manager")
    client.send_combo("meta_l", "q")
    time.sleep(2)

    # 5.4 Control Center (Mod + S)
    print("\n--- 8. Testing Control Center ---")
    client.send_combo("meta_l", "s")
    time.sleep(3)
    client.screenshot("qmp_21_control_center")
    client.send_key("esc")
    time.sleep(1)

    # 6. Test Caelestia Edge Triggers
    print("\n--- 9. Testing Caelestia Edge Triggers ---")
    print("Hovering top edge (640, 2)...")
    client.move_abs(640, 2)
    time.sleep(2)
    client.screenshot("qmp_16_caelestia_top_hover")

    print("Hovering right edge (1278, 150)...")
    client.move_abs(1278, 150)
    time.sleep(2)
    client.screenshot("qmp_17_caelestia_right_hover")

    client.move_abs(640, 400)
    time.sleep(1)

    print("\nAll dogfooding tests complete! Terminating VM...")
    client.close()
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()

    print("=== Complete! ===")

if __name__ == "__main__":
    main()
