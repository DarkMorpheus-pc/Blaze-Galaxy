#!/usr/bin/env python3
import sys
import subprocess
import time

KEY_MAP = {
    'a': 'KEY_A', 'b': 'KEY_B', 'c': 'KEY_C', 'd': 'KEY_D', 'e': 'KEY_E',
    'f': 'KEY_F', 'g': 'KEY_G', 'h': 'KEY_H', 'i': 'KEY_I', 'j': 'KEY_J',
    'k': 'KEY_K', 'l': 'KEY_L', 'm': 'KEY_M', 'n': 'KEY_N', 'o': 'KEY_O',
    'p': 'KEY_P', 'q': 'KEY_Q', 'r': 'KEY_R', 's': 'KEY_S', 't': 'KEY_T',
    'u': 'KEY_U', 'v': 'KEY_V', 'w': 'KEY_W', 'x': 'KEY_X', 'y': 'KEY_Y',
    'z': 'KEY_Z',
    '0': 'KEY_0', '1': 'KEY_1', '2': 'KEY_2', '3': 'KEY_3', '4': 'KEY_4',
    '5': 'KEY_5', '6': 'KEY_6', '7': 'KEY_7', '8': 'KEY_8', '9': 'KEY_9',
    ' ': 'KEY_SPACE', '-': 'KEY_MINUS', '_': ['KEY_LEFTSHIFT', 'KEY_MINUS'],
    '.': 'KEY_DOT', '/': 'KEY_SLASH', '=': 'KEY_EQUAL', ':': ['KEY_LEFTSHIFT', 'KEY_SEMICOLON'],
    '\n': 'KEY_ENTER'
}

def send_text(vm_name, text):
    for ch in text:
        mapping = KEY_MAP.get(ch.lower())
        if not mapping:
            continue
        keys = []
        if ch.isupper() and isinstance(mapping, str):
            keys = ['KEY_LEFTSHIFT', mapping]
        elif isinstance(mapping, list):
            keys = mapping
        else:
            keys = [mapping]
        
        cmd = ["virsh", "-c", "qemu:///system", "send-key", vm_name] + keys
        subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(0.04)

if __name__ == "__main__":
    vm = sys.argv[1]
    msg = " ".join(sys.argv[2:])
    send_text(vm, msg + "\n")
