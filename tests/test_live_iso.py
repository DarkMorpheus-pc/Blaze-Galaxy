import os
import sys
import time
import socket
import subprocess

ISO_PATH = "/home/darkmorpheus/BlazeFedora/Blaze-SolarEvolution-5-x86_64.iso"
SOCK_PATH = "/tmp/qemu-monitor.sock"
SCREEN_PPM = "/tmp/qemu_screen.ppm"
BRAIN_DIR = "/home/darkmorpheus/.gemini/antigravity/brain/cafdcfba-10e8-4d85-b938-c1121a5b7023"

def send_qemu_cmd(cmd):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(SOCK_PATH)
    time.sleep(0.1)
    s.recv(1024) # read prompt
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

    # Wait for socket
    for _ in range(30):
        if os.path.exists(SOCK_PATH):
            break
        time.sleep(0.5)

    if not os.path.exists(SOCK_PATH):
        print("QEMU failed to initialize monitor socket.")
        proc.kill()
        sys.exit(1)

    print("QEMU running. Monitoring boot...")
    # GRUB timeout is 10s. Press enter to boot immediately
    time.sleep(2)
    send_qemu_cmd("sendkey ret")
    print("Sent ENTER to GRUB.")

    # Boot loop: wait for desktop
    for i in range(1, 15):
        time.sleep(5)
        snap = take_screenshot(f"qemu_boot_step_{i*5}s")
        print(f"Boot check {i*5}s...")

    print("Taking desktop screenshot...")
    take_screenshot("qemu_desktop_live")

    # Test Super + Space (Omnibar)
    print("Sending Super+Space for Omnibar...")
    send_qemu_cmd("sendkey meta_l-spc")
    time.sleep(2)
    take_screenshot("qemu_omnibar_live")

    # Dismiss Omnibar
    send_qemu_cmd("sendkey esc")
    time.sleep(1)

    # Test Super + G (GameZone)
    print("Sending Super+G for GameZone...")
    send_qemu_cmd("sendkey meta_l-g")
    time.sleep(3)
    take_screenshot("qemu_gamezone_live")

    print("Shutting down QEMU...")
    send_qemu_cmd("quit")
    proc.wait(timeout=10)
    print("QEMU finished cleanly.")

if __name__ == "__main__":
    main()
