#!/usr/bin/env python3
import os
import sys
import time
import socket
import subprocess

ISO_PATH = "/home/darkmorpheus/BlazeFedora/Blaze-SolarEvolution-5-x86_64.iso"
DISK_PATH = "/tmp/blaze_test_disk.qcow2"
SOCK_PATH = "/tmp/qemu-monitor-ag.sock"
SCREEN_PPM = "/tmp/qemu_screen_ag.ppm"
BRAIN_DIR = "/home/darkmorpheus/.gemini/antigravity/brain/cafdcfba-10e8-4d85-b938-c1121a5b7023"

def send_qemu_cmd(cmd):
    try:
        s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        s.connect(SOCK_PATH)
        time.sleep(0.05)
        s.recv(1024)
        s.sendall((cmd + "\n").encode('utf-8'))
        time.sleep(0.15)
        resp = s.recv(4096)
        s.close()
        return resp.decode('utf-8', errors='ignore')
    except Exception as e:
        print(f"Socket error for '{cmd}': {e}")
        return ""

def take_screenshot(name):
    if os.path.exists(SCREEN_PPM):
        try:
            os.remove(SCREEN_PPM)
        except OSError:
            pass
    send_qemu_cmd(f"screendump {SCREEN_PPM}")
    time.sleep(0.5)
    out_png = os.path.join(BRAIN_DIR, f"{name}.png")
    if os.path.exists(SCREEN_PPM):
        subprocess.run(["ffmpeg", "-y", "-i", SCREEN_PPM, out_png], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        print(f"Captured: {out_png}")
        return out_png
    else:
        print(f"Failed to capture {SCREEN_PPM}")
        return None

def click_coords(x, y, w=1280, h=800):
    norm_x = int(x / w * 32767)
    norm_y = int(y / h * 32767)
    send_qemu_cmd(f"mouse_move {norm_x} {norm_y}")
    time.sleep(0.1)
    send_qemu_cmd("mouse_button 1")
    time.sleep(0.1)
    send_qemu_cmd("mouse_button 0")
    time.sleep(0.2)

def main():
    print("=== BlazeOS Interactive Dogfooding & Anaconda Test ===")

    if not os.path.exists(DISK_PATH):
        subprocess.run(["qemu-img", "create", "-f", "qcow2", DISK_PATH, "30G"], check=True)

    if os.path.exists(SOCK_PATH):
        try:
            os.remove(SOCK_PATH)
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

    print("Booting QEMU...")
    time.sleep(2)
    send_qemu_cmd("sendkey ret")

    # Wait for desktop (~45s)
    time.sleep(45)
    for _ in range(5):
        take_screenshot("ag_boot_check")
        time.sleep(3)

    take_screenshot("ag_desktop_ready")

    # 1. Open GameZone and test scrolling into Katman 4 and Katman 5
    print("\n--- Testing GameZone UI & Scrolling ---")
    send_qemu_cmd("sendkey meta_l-g")
    time.sleep(4)
    take_screenshot("ag_gz_01_initial")

    # Click in the right content area (X: 700, Y: 300) to ensure focus on main scroll
    click_coords(700, 300)
    time.sleep(0.5)

    # Scroll down using Page Down
    send_qemu_cmd("sendkey pgdn")
    time.sleep(1)
    take_screenshot("ag_gz_02_scrolled_news")

    send_qemu_cmd("sendkey pgdn")
    time.sleep(1)
    take_screenshot("ag_gz_03_scrolled_library_showcase")

    send_qemu_cmd("sendkey pgdn")
    time.sleep(1)
    take_screenshot("ag_gz_04_bottom_showcase")

    # Exit GameZone with Esc
    send_qemu_cmd("sendkey esc")
    time.sleep(2)
    take_screenshot("ag_gz_05_closed")

    # 2. Test Installer Wizard Flow (Clicks through Steps 1 to 7)
    print("\n--- Testing BlazeOS Installer Wizard ---")
    # Take screenshot of Step 1
    take_screenshot("ag_wiz_step1_welcome")

    # Click 'Sonraki' on Step 1 (Center around 412, 757)
    print("Clicking 'Sonraki' on Step 1 (Welcome)...")
    click_coords(412, 757)
    time.sleep(2)
    take_screenshot("ag_wiz_step2_datetime")

    # Click 'Sonraki' on Step 2 (Date/Time)
    print("Clicking 'Sonraki' on Step 2 (DateTime)...")
    click_coords(412, 757)
    time.sleep(2)
    take_screenshot("ag_wiz_step3_system_prefs")

    # Click 'Sonraki' on Step 3 (Preferences / DE / Bootloader)
    print("Clicking 'Sonraki' on Step 3 (System Preferences)...")
    click_coords(412, 757)
    time.sleep(2)
    take_screenshot("ag_wiz_step4_install_method")

    # Click 'Sonraki' on Step 4 (Install Method)
    print("Clicking 'Sonraki' on Step 4 (Install Method)...")
    click_coords(412, 757)
    time.sleep(2)
    take_screenshot("ag_wiz_step5_storage")

    # On Step 5 (Storage Configuration): Click disk or Sonraki
    print("Clicking on Step 5 (Storage Configuration)...")
    click_coords(412, 757)
    time.sleep(2)
    take_screenshot("ag_wiz_step6_account")

    # Click 'Sonraki' on Step 6
    print("Clicking on Step 6 (Account)...")
    click_coords(412, 757)
    time.sleep(2)
    take_screenshot("ag_wiz_step7_review")

    print("\nAll interactive steps executed! Terminating VM...")
    proc.terminate()
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        proc.kill()

    print("Interactive dogfooding test completed successfully.")

if __name__ == "__main__":
    main()
