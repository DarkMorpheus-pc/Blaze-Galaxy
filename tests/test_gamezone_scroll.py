import os
import sys
import time
import socket
import subprocess

ISO_PATH = "/home/darkmorpheus/BlazeFedora/Blaze-SolarEvolution-5-x86_64.iso"
SOCK_PATH = "/tmp/qemu-monitor-gz-scroll.sock"
SCREEN_PPM = "/tmp/qemu_screen_gz_scroll.ppm"
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

    print("Starting QEMU VM...")
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

    time.sleep(2)
    send_qemu_cmd("sendkey ret")

    # Wait for desktop (~45s)
    time.sleep(45)

    # Close welcome dialog
    send_qemu_cmd("sendkey alt-f4")
    time.sleep(2)

    # Open GameZone
    send_qemu_cmd("sendkey meta_l-g")
    time.sleep(3)

    # Move mouse to center and scroll down (Page_Down)
    send_qemu_cmd("sendkey pgdn")
    time.sleep(1)
    take_screenshot("qemu_gamezone_scrolled_news")

    send_qemu_cmd("sendkey pgdn")
    time.sleep(1)
    take_screenshot("qemu_gamezone_scrolled_grid")

    send_qemu_cmd("quit")
    proc.wait(timeout=10)
    print("Scroll test completed.")

if __name__ == "__main__":
    main()
