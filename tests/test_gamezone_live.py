import os
import sys
import time
import socket
import subprocess

ISO_PATH = "/home/darkmorpheus/BlazeFedora/Blaze-SolarEvolution-5-x86_64.iso"
SOCK_PATH = "/tmp/qemu-monitor-gz.sock"
SCREEN_PPM = "/tmp/qemu_screen_gz.ppm"
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

def main():
    if os.path.exists(SOCK_PATH):
        os.remove(SOCK_PATH)

    print("Starting QEMU VM for GameZone test...")
    qemu_cmd = [
        "qemu-system-x86_64",
        "-enable-kvm",
        "-m", "4G",
        "-smp", "4",
        "-cpu", "host",
        "-vga", "virtio",
        "-netdev", "user,id=net0",
        "-device", "virtio-net-pci,netdev=net0",
        "-display", "none",
        "-cdrom", ISO_PATH,
        "-monitor", f"unix:{SOCK_PATH},server,nowait"
    ]
    proc = subprocess.Popen(qemu_cmd)

    for _ in range(30):
        if os.path.exists(SOCK_PATH):
            break
        time.sleep(0.5)

    if not os.path.exists(SOCK_PATH):
        print("QEMU failed to initialize monitor socket.")
        proc.kill()
        sys.exit(1)

    print("QEMU running. Booting...")
    time.sleep(2)
    send_qemu_cmd("sendkey ret")

    # Wait for desktop (~45s)
    time.sleep(45)
    for _ in range(6):
        snap = take_screenshot("qemu_gz_boot_wait")
        time.sleep(5)

    take_screenshot("qemu_gz_desktop_ready")

    # Close welcome dialog by clicking or pressing Alt+F4 / Escape
    print("Closing welcome dialog...")
    send_qemu_cmd("sendkey alt-f4")
    time.sleep(2)
    take_screenshot("qemu_gz_welcome_closed")

    # Launch GameZone using Super+G
    print("Sending Super+G to launch GameZone...")
    send_qemu_cmd("sendkey meta_l-g")
    time.sleep(4)
    take_screenshot("qemu_gamezone_full_opened")

    # Test right arrow to navigate game cards
    print("Navigating cards with Right arrow...")
    send_qemu_cmd("sendkey right")
    time.sleep(2)
    take_screenshot("qemu_gamezone_card_nav")

    # Press Enter on selected card
    print("Pressing Enter on selected card...")
    send_qemu_cmd("sendkey ret")
    time.sleep(2)
    take_screenshot("qemu_gamezone_action_enter")

    print("Shutting down QEMU...")
    send_qemu_cmd("quit")
    proc.wait(timeout=10)
    print("QEMU test completed.")

if __name__ == "__main__":
    main()
