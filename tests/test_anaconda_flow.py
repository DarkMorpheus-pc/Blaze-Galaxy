import os
import sys
import time
import socket
import subprocess

ISO_PATH = "/home/darkmorpheus/BlazeFedora/Blaze-SolarEvolution-5-x86_64.iso"
SOCK_PATH = "/tmp/qemu-monitor-flow.sock"
SCREEN_PPM = "/tmp/qemu_screen_flow.ppm"
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

    print("Launching QEMU to test Anaconda flow...")
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

    # Wait for desktop
    time.sleep(45)
    take_screenshot("flow_1_welcome_open")

    # Send Escape to close welcome dialog
    print("Sending Escape to close welcome...")
    send_qemu_cmd("sendkey esc")
    time.sleep(2)
    take_screenshot("flow_2_welcome_closed_anaconda_clean")

    # Click on the Anaconda window to ensure focus (e.g. x=500, y=400)
    print("Focusing Anaconda window...")
    send_qemu_cmd("mouse_move 500 400")
    send_qemu_cmd("mouse_button 1")
    time.sleep(0.2)
    send_qemu_cmd("mouse_button 0")
    time.sleep(1)

    # Click on Next / İleri button:
    # In Anaconda webui/cockpit or gtk, the Next button is usually bottom-right or middle.
    # Let's send Tab a few times then Enter, or Alt+i / Alt+n
    print("Sending Alt+i (İleri)...")
    send_qemu_cmd("sendkey alt-i")
    time.sleep(2)
    take_screenshot("flow_3_anaconda_next_alti")

    print("Sending Tab and Enter...")
    send_qemu_cmd("sendkey tab")
    time.sleep(0.5)
    send_qemu_cmd("sendkey ret")
    time.sleep(2)
    take_screenshot("flow_4_anaconda_tab_ret")

    send_qemu_cmd("quit")
    proc.wait(timeout=10)
    print("Done.")

if __name__ == "__main__":
    main()
