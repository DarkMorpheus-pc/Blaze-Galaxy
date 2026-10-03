#!/usr/bin/env python3
import sys
import subprocess
import json
import base64
import time

def vm_exec(cmd_str, domain="blaze45", timeout=600):
    payload = json.dumps({
        "execute": "guest-exec",
        "arguments": {
            "path": "/bin/bash",
            "arg": ["-c", cmd_str],
            "capture-output": True
        }
    })
    res = subprocess.check_output(["virsh", "-c", "qemu:///system", "qemu-agent-command", domain, payload])
    pid = json.loads(res)["return"]["pid"]
    start = time.time()
    while time.time() - start < timeout:
        status_payload = json.dumps({"execute": "guest-exec-status", "arguments": {"pid": pid}})
        s_res = subprocess.check_output(["virsh", "-c", "qemu:///system", "qemu-agent-command", domain, status_payload])
        s_data = json.loads(s_res)["return"]
        if s_data.get("exited"):
            out = base64.b64decode(s_data.get("out-data", "")).decode("utf-8", errors="replace")
            err = base64.b64decode(s_data.get("err-data", "")).decode("utf-8", errors="replace")
            return s_data.get("exitcode", 0), out, err
        time.sleep(0.3)
    raise TimeoutError(f"Command timed out after {timeout}s: {cmd_str}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: vm_exec.py <command>")
        sys.exit(1)
    cmd = " ".join(sys.argv[1:])
    code, out, err = vm_exec(cmd)
    if out:
        sys.stdout.write(out)
    if err:
        sys.stderr.write(err)
    sys.exit(code)
