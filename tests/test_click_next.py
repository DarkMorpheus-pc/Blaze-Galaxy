import os
import sys
import time
import socket
import subprocess

ISO_PATH = "/home/darkmorpheus/BlazeFedora/Blaze-SolarEvolution-5-x86_64.iso"
SOCK_PATH = "/tmp/qemu-monitor-click.sock"
SCREEN_PPM = "/tmp/qemu_screen_click.ppm"
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

    print("Launching QEMU for Next button click...")
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
    send_qemu_cmd("sendkey ret") # boot

    time.sleep(45)

    # Close welcome with Esc
    send_qemu_cmd("sendkey esc")
    time.sleep(2)
    take_screenshot("click_1_welcome_closed")

    # In QEMU, mouse coordinates can be moved or sent with mouse_move dx dy or absolute
    # But even better, in Anaconda WebUI: Next button has key shortcut or mouse click.
    # Let's test sending mouse click by positioning or clicking directly.
    # Note: QEMU PS/2 mouse uses relative motion. To reset mouse to (0,0), move -1000 -1000 several times:
    print("Calibrating mouse cursor to top-left (0,0)...")
    for _ in range(5):
        send_qemu_cmd("mouse_move -1000 -1000")
        time.sleep(0.05)

    # Now move from (0,0) to Sonraki button (x=322, y=745)
    print("Moving mouse to Sonraki button (322, 745)...")
    send_qemu_cmd("mouse_move 322 745")
    time.sleep(0.5)
    take_screenshot("click_2_mouse_on_sonraki")

    print("Clicking left mouse button on Sonraki...")
    send_qemu_cmd("mouse_button 1")
    time.sleep(0.2)
    send_qemu_cmd("mouse_button 0")
    time.sleep(3)
    take_screenshot("click_3_after_sonraki_clicked")

    send_qemu_cmd("quit")
    proc.wait(timeout=10)
    print("Done.")

if __name__ == "__main__":
    main()
