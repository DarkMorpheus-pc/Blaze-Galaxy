import os
import sys
import time
import socket
import subprocess

ISO_PATH = "/home/darkmorpheus/BlazeFedora/Blaze-SolarEvolution-5-x86_64.iso"
SOCK_PATH = "/tmp/qemu-monitor-2.sock"
SCREEN_PPM = "/tmp/qemu_screen_2.ppm"
BRAIN_DIR = "/home/darkmorpheus/.gemini/antigravity/brain/cafdcfba-10e8-4d85-b938-c1121a5b7023"

def send_qemu_cmd(cmd):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(SOCK_PATH)
    time.sleep(0.1)
    s.recv(1024)
    s.sendall((cmd + "\n").encode('utf-8'))
    time.sleep(0.2)
    resp = s.recv(4096)
    s.close()
    return resp.decode('utf-8', errors='ignore')

def take_screenshot(name):
    if os.path.exists(SCREEN_PPM):
        os.remove(SCREEN_PPM)
    send_qemu_cmd(f"screendump {SCREEN_PPM}")
    time.sleep(0.5)
    out_png = os.path.join(BRAIN_DIR, f"{name}.png")
    if os.path.exists(SCREEN_PPM):
        subprocess.run(["ffmpeg", "-y", "-i", SCREEN_PPM, out_png], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        print(f"Captured screenshot: {out_png}")
        return out_png
    else:
        print(f"Failed to generate {SCREEN_PPM}")
        return None

def test_anaconda_next_button():
    if os.path.exists(SOCK_PATH):
        os.remove(SOCK_PATH)

    print("\n--- TEST 1: ANACONDA NEXT BUTTON CLICK TEST ---")
    proc = subprocess.Popen([
        "qemu-system-x86_64",
        "-enable-kvm",
        "-m", "4G",
        "-smp", "4",
        "-cpu", "host",
        "-vga", "virtio",
        "-display", "none",
        "-cdrom", ISO_PATH,
        "-monitor", f"unix:{SOCK_PATH},server,nowait"
    ])

    for _ in range(30):
        if os.path.exists(SOCK_PATH):
            break
        time.sleep(0.5)

    time.sleep(2)
    send_qemu_cmd("sendkey ret") # boot first entry

    # Wait for desktop (50s)
    time.sleep(45)

    # Dismiss welcome dialog by sending Enter (Masaüstüne Başla) or Esc
    print("Dismissing welcome dialog...")
    send_qemu_cmd("sendkey esc")
    time.sleep(2)
    take_screenshot("qemu_anaconda_focused")

    # In Anaconda, the Next button is at bottom right, or we can send Alt+I (İleri) or Tab/Enter
    # Let's test clicking the Next button via mouse or sending Tab -> Enter / Alt+i
    print("Testing Anaconda 'İleri' button...")
    send_qemu_cmd("sendkey alt-i")
    time.sleep(3)
    take_screenshot("qemu_anaconda_step2_alti")

    # Let's also test mouse click at bottom right: (X=900, Y=730 in 1024x768 resolution)
    # Or sending Tab/Enter
    send_qemu_cmd("sendkey ret")
    time.sleep(2)
    take_screenshot("qemu_anaconda_step2_ret")

    send_qemu_cmd("quit")
    proc.wait(timeout=10)
    print("Test 1 completed.")

def test_berp_boot():
    if os.path.exists(SOCK_PATH):
        os.remove(SOCK_PATH)

    print("\n--- TEST 2: BERP (BLAZE EMERGENCY RECOVERY PROTOCOL) BOOT TEST ---")
    proc = subprocess.Popen([
        "qemu-system-x86_64",
        "-enable-kvm",
        "-m", "4G",
        "-smp", "4",
        "-cpu", "host",
        "-vga", "virtio",
        "-display", "none",
        "-cdrom", ISO_PATH,
        "-monitor", f"unix:{SOCK_PATH},server,nowait"
    ])

    for _ in range(30):
        if os.path.exists(SOCK_PATH):
            break
        time.sleep(0.5)

    time.sleep(2)
    print("Selecting BERP in GRUB menu (Down -> Enter)...")
    send_qemu_cmd("sendkey down")
    time.sleep(0.5)
    take_screenshot("qemu_grub_berp_selected")
    send_qemu_cmd("sendkey ret")
    print("Booting BERP...")

    # Wait 25s for BERP console TUI to load
    for i in range(1, 6):
        time.sleep(5)
        take_screenshot(f"qemu_berp_boot_{i*5}s")

    send_qemu_cmd("quit")
    proc.wait(timeout=10)
    print("Test 2 completed.")

if __name__ == "__main__":
    test_anaconda_next_button()
    test_berp_boot()
