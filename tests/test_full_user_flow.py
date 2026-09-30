#!/usr/bin/env python3
import os
import sys
import time
import socket
import subprocess

ISO_PATH = "/home/darkmorpheus/BlazeFedora/Blaze-SolarEvolution-5-x86_64.iso"
DISK_PATH = "/tmp/blaze_test_disk.qcow2"
SOCK_PATH = "/tmp/qemu-monitor-full.sock"
SCREEN_PPM = "/tmp/qemu_screen_full.ppm"
BRAIN_DIR = "/home/darkmorpheus/.gemini/antigravity/brain/cafdcfba-10e8-4d85-b938-c1121a5b7023"

def send_qemu_cmd(cmd):
    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.connect(SOCK_PATH)
        time.sleep(0.1)
        s.recv(1024)
        s.sendall((cmd + "\n").encode('utf-8'))
        time.sleep(0.2)
        resp = s.recv(4096)
        s.close()
        return resp.decode('utf-8', errors='ignore')
    except Exception as e:
        print(f"Socket error for cmd '{cmd}': {e}")
        return ""

def take_screenshot(name):
    if os.path.exists(SCREEN_PPM):
        try:
            os.remove(SCREEN_PPM)
        except OSError:
            pass
    send_qemu_cmd(f"screendump {SCREEN_PPM}")
    time.sleep(0.6)
    out_png = os.path.join(BRAIN_DIR, f"{name}.png")
    if os.path.exists(SCREEN_PPM):
        subprocess.run(["ffmpeg", "-y", "-i", SCREEN_PPM, out_png], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        print(f"Captured: {out_png}")
        return out_png
    else:
        print(f"Failed to capture {SCREEN_PPM}")
        return None

def type_string(text):
    for ch in text:
        if ch == ' ':
            send_qemu_cmd("sendkey spc")
        elif ch == '\n':
            send_qemu_cmd("sendkey ret")
        elif ch == '-':
            send_qemu_cmd("sendkey minus")
        elif ch == '.':
            send_qemu_cmd("sendkey dot")
        elif ch == '/':
            send_qemu_cmd("sendkey slash")
        elif ch == ':':
            send_qemu_cmd("sendkey shift-semicolon")
        elif ch == '_':
            send_qemu_cmd("sendkey shift-minus")
        elif ch == ';':
            send_qemu_cmd("sendkey semicolon")
        elif ch.isupper():
            send_qemu_cmd(f"sendkey shift-{ch.lower()}")
        else:
            send_qemu_cmd(f"sendkey {ch}")
        time.sleep(0.08)

def main():
    print("=== BlazeOS Automated End-to-End Dogfooding Test ===")

    # 1. Create or ensure 30GB test disk
    if not os.path.exists(DISK_PATH):
        print(f"Creating 30GB test disk at {DISK_PATH}...")
        subprocess.run(["qemu-img", "create", "-f", "qcow2", DISK_PATH, "30G"], check=True)
    else:
        print(f"Test disk {DISK_PATH} already exists.")

    if os.path.exists(SOCK_PATH):
        try:
            os.remove(SOCK_PATH)
        except OSError:
            pass

    # 2. Launch QEMU VM
    print("Launching QEMU VM with ISO and virtio disk...")
    qemu_cmd = [
        "qemu-system-x86_64",
        "-enable-kvm",
        "-m", "6G",
        "-smp", "4",
        "-cpu", "host",
        "-vga", "virtio",
        "-netdev", "user,id=net0",
        "-device", "virtio-net-pci,netdev=net0",
        "-drive", f"file={DISK_PATH},if=virtio,format=qcow2",
        "-cdrom", ISO_PATH,
        "-display", "none",
        "-monitor", f"unix:{SOCK_PATH},server,nowait"
    ]
    proc = subprocess.Popen(qemu_cmd)

    for _ in range(30):
        if os.path.exists(SOCK_PATH):
            break
        time.sleep(0.5)

    if not os.path.exists(SOCK_PATH):
        print("ERROR: QEMU monitor socket did not appear.")
        proc.kill()
        sys.exit(1)

    print("QEMU initialized. Selecting boot menu...")
    time.sleep(2)
    send_qemu_cmd("sendkey ret")

    # Wait for desktop (~45 seconds)
    print("Waiting 45s for desktop environment to load...")
    time.sleep(45)

    for i in range(8):
        take_screenshot(f"full_01_boot_wait_{i}")
        time.sleep(4)

    take_screenshot("full_01_desktop_ready")

    # Close welcome app
    print("Closing welcome screen...")
    send_qemu_cmd("sendkey alt-f4")
    time.sleep(2)
    take_screenshot("full_02_welcome_closed")

    # ── Test 1: GameZone Full Audit ──
    print("\n[Test 1] Opening GameZone (Super+G)...")
    send_qemu_cmd("sendkey meta_l-g")
    time.sleep(4)
    take_screenshot("full_03_gamezone_opened")

    # Scroll down to view News, Empty Library, and Store Showcase
    print("Scrolling down in GameZone to view Store Showcase...")
    for _ in range(5):
        send_qemu_cmd("sendkey pgdn")
        time.sleep(0.5)
    time.sleep(2)
    take_screenshot("full_04_gamezone_scrolled_showcase")

    for _ in range(3):
        send_qemu_cmd("sendkey down")
        time.sleep(0.3)
    time.sleep(1)
    take_screenshot("full_05_gamezone_bottom_view")

    # Scroll back up to Hero Banner
    for _ in range(6):
        send_qemu_cmd("sendkey pgup")
        time.sleep(0.4)
    time.sleep(1)
    take_screenshot("full_06_gamezone_back_top")

    # Exit GameZone with Escape
    print("Exiting GameZone (Esc)...")
    send_qemu_cmd("sendkey esc")
    time.sleep(2)
    take_screenshot("full_07_desktop_after_gamezone")

    # ── Test 2: Omnibar & Terminal ──
    print("\n[Test 2] Testing Omnibar (Super+Space)...")
    send_qemu_cmd("sendkey meta_l-spc")
    time.sleep(2)
    take_screenshot("full_08_omnibar_opened")

    send_qemu_cmd("sendkey esc")
    time.sleep(1)

    print("Opening Kitty Terminal (Super+Enter)...")
    send_qemu_cmd("sendkey meta_l-ret")
    time.sleep(3)
    take_screenshot("full_09_terminal_opened")

    # Test Network inside guest terminal
    print("Testing guest internet & DNS in terminal...")
    type_string("ip a; ping -c 3 1.1.1.1\n")
    time.sleep(5)
    take_screenshot("full_10_terminal_ping_tested")

    type_string("curl -I https://www.google.com\n")
    time.sleep(4)
    take_screenshot("full_11_terminal_curl_tested")

    # ── Test 3: Anaconda Installer ──
    print("\n[Test 3] Launching Anaconda Installer (liveinst)...")
    type_string("sudo liveinst\n")
    time.sleep(12)
    take_screenshot("full_12_anaconda_started")

    # Wait for Anaconda GUI
    time.sleep(10)
    take_screenshot("full_13_anaconda_gui_loaded")

    print("All tests completed. Keeping VM running 10s for final inspection.")
    time.sleep(10)

    take_screenshot("full_14_final_state")

    print("Terminating test VM...")
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()

    print("Test run finished successfully.")

if __name__ == "__main__":
    main()
