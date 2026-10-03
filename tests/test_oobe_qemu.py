#!/usr/bin/env python3
import os
import sys
import time
import socket
import subprocess
import json

ISO_PATH = "/home/darkmorpheus/BlazeFedora/Blaze-SolarEvolution-5-x86_64.iso"
DISK_PATH = "/tmp/blaze_test_disk.qcow2"
QMP_PATH = "/tmp/qemu-qmp-oobe.sock"
SCREEN_PPM = "/tmp/qemu_screen_oobe.ppm"
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
        self.sock.recv(4096)
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
        if qcode == 'return':
            qcode = 'ret'
        self.send({
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    {"type": "key", "data": {"key": {"type": "qcode", "data": qcode}, "down": True}}
                ]
            }
        })
        time.sleep(0.04)
        self.send({
            "execute": "input-send-event",
            "arguments": {
                "events": [
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
            elif ch == '-':
                self.send_key('minus')
            elif ch == '.':
                self.send_key('dot')
            elif ch == '/':
                self.send_key('slash')
            else:
                self.send_key(ch.lower())
            time.sleep(0.08)

    def send_combo(self, mod, key):
        self.send({
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    {"type": "key", "data": {"key": {"type": "qcode", "data": mod}, "down": True}}
                ]
            }
        })
        time.sleep(0.05)
        self.send({
            "execute": "input-send-event",
            "arguments": {
                "events": [
                    {"type": "key", "data": {"key": {"type": "qcode", "data": key}, "down": True}},
                    {"type": "key", "data": {"key": {"type": "qcode", "data": key}, "down": False}}
                ]
            }
        })
        time.sleep(0.05)
        self.send({
            "execute": "input-send-event",
            "arguments": {
                "events": [
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
    print("=== BlazeOS Apple-Style OOBE Setup Assistant Live Verification ===")

    if os.path.exists(DISK_PATH):
        try:
            os.remove(DISK_PATH)
        except OSError:
            pass
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
        "-boot", "order=d",
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

    # Dismiss Welcome HUD
    print("Dismissing Welcome HUD (click Masaüstüne Başla / close)...")
    client.click(845, 682)
    time.sleep(1)
    client.send_key("esc")
    time.sleep(1)

    # Close Anaconda window to reveal clean Live Desktop
    print("Closing Anaconda installer (Mod + Q)...")
    client.send_combo("meta_l", "q")
    time.sleep(2)
    client.screenshot("qmp_02_desktop_clear")

    # 1. Open Omnibar and search for oobe
    print("\n--- 1. Testing Omnibar OOBE Action ---")
    client.send_combo("meta_l", "spc")
    time.sleep(2)
    client.type_text("oobe")
    time.sleep(1.5)
    client.screenshot("qmp_22_omnibar_oobe")

    # 2. Launch OOBE assistant via Omnibar
    print("Launching OOBE assistant via Omnibar (ret)...")
    client.send_key("ret")
    time.sleep(5)
    client.screenshot("qmp_23_oobe_hello_animation")

    # 3. Transition to Setup Wizard (click and space)
    print("\n--- 2. Transitioning from Hello to Setup Wizard ---")
    client.click(640, 400)
    time.sleep(0.5)
    client.send_key("spc")
    time.sleep(3)
    client.screenshot("qmp_24_oobe_wizard_step1_lang")

    # 4. Step 1: Language & Keymap -> Next
    print("\n--- 3. Wizard Step 1: Language & Keymap ---")
    print("Clicking Next (980, 700)...")
    client.click(980, 700)
    time.sleep(2)
    client.screenshot("qmp_25_oobe_wizard_step2_tz")

    # 5. Step 2: Timezone & Region -> Next
    print("\n--- 4. Wizard Step 2: Timezone & Region ---")
    print("Clicking Next (980, 700)...")
    client.click(980, 700)
    time.sleep(2)
    client.screenshot("qmp_26_oobe_wizard_step3_account")

    # 6. Step 3: Account & Password -> Next
    print("\n--- 5. Wizard Step 3: User Account & Password ---")
    print("Clicking Next (980, 700)...")
    client.click(980, 700)
    time.sleep(2)
    client.screenshot("qmp_27_oobe_wizard_step4_desktop")

    # 7. Step 4: Desktop Engine & Theme Accents -> Next
    print("\n--- 6. Wizard Step 4: Desktop Engine & Theme Accents ---")
    print("Clicking Next (980, 700)...")
    client.click(980, 700)
    time.sleep(2)
    client.screenshot("qmp_28_oobe_wizard_step5_acoustic")

    # 8. Step 5: Acoustic Sensory & Gaming -> Complete Setup
    print("\n--- 7. Wizard Step 5: Acoustic Sensory & Gaming ---")
    print("Clicking Complete Setup (980, 700)...")
    client.click(980, 700)
    time.sleep(3)
    client.screenshot("qmp_29_oobe_wizard_step6_checklist")

    # 9. Step 6: Finalization Checklist
    print("\n--- 8. Wizard Step 6: Finalization Checklist ---")
    time.sleep(3)
    client.screenshot("qmp_30_oobe_finalized")

    # Close OOBE window
    print("Closing OOBE window (Mod + Q)...")
    client.send_combo("meta_l", "q")
    time.sleep(2)

    print("\nVerification complete! Terminating VM...")
    client.close()
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()

    print("=== Complete! ===")

if __name__ == "__main__":
    main()
