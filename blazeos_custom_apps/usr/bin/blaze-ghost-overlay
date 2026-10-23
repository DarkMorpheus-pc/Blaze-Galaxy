#!/usr/bin/env python3
"""
Blaze Continuum — Ghost Surface Overlay (BCDL: Kinetic Glass Edition)
Renders the ephemeral Wayland Ghost Surface with 1px specular refraction,
unblurred captured window buffer, and bottom-right Swiss telemetry pill HUD.
Intercepts pointer/keyboard input into the BCTP Input Replay Queue.
"""

import sys
import os
import signal
import json
import time
import argparse
import gi

gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, Gdk, GLib, Gio

BCDL_CSS = """
window.blz-ghost-window {
    background-color: #0d0f12;
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.45);
}

.blz-scrim-layer {
    background-color: rgba(8, 9, 11, 0.18);
}

.blz-telemetry-pill {
    background-color: rgba(32, 37, 43, 0.88);
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 9999px;
    padding: 4px 14px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
    margin: 16px;
}

.blz-status-dot {
    font-size: 10px;
    margin-right: 6px;
}

.blz-status-dot.syncing {
    color: #e5be6b; /* Mineral Amber */
}

.blz-status-dot.stable {
    color: #85d79e; /* Celadon Green */
}

.blz-telemetry-text {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 11px;
    font-weight: 500;
    color: #e5e9f0;
}

.blz-telemetry-dim {
    color: #9ba3af;
    margin: 0 4px;
}
"""

class GhostOverlayApp(Gtk.Application):
    def __init__(self, image_path, epoch, latency, inputs_count, status, auto_dismiss_ms, event_log):
        super().__init__(application_id="org.blazeos.continuum.ghost",
                         flags=Gio.ApplicationFlags.FLAGS_NONE)
        self.image_path = image_path
        self.epoch = epoch
        self.latency = latency
        self.inputs_count = inputs_count
        self.status = status
        self.auto_dismiss_ms = auto_dismiss_ms
        self.event_log = event_log
        self.window = None

    def do_activate(self):
        # 1. Apply BCDL CSS
        css_provider = Gtk.CssProvider()
        css_provider.load_from_data(BCDL_CSS.encode('utf-8'))
        Gtk.StyleContext.add_provider_for_display(
            Gdk.Display.get_default(),
            css_provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )

        # 2. Main Window
        self.window = Gtk.ApplicationWindow(application=self, title="Blaze Ghost Surface")
        self.window.add_css_class("blz-ghost-window")
        self.window.set_default_size(960, 540)

        # 3. Stack layout (Image + Scrim + HUD overlay)
        overlay = Gtk.Overlay()

        # Content Layer: Captured Window Image
        picture = Gtk.Picture.new_for_filename(self.image_path)
        picture.set_can_shrink(True)
        picture.set_content_fit(Gtk.ContentFit.CONTAIN)
        overlay.set_child(picture)

        # Scrim Layer (Dimming)
        scrim = Gtk.Box()
        scrim.add_css_class("blz-scrim-layer")
        scrim.set_hexpand(True)
        scrim.set_vexpand(True)
        overlay.add_overlay(scrim)

        # HUD Layer: Floating Telemetry Pill in Bottom-Right
        pill_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        pill_box.add_css_class("blz-telemetry-pill")
        pill_box.set_halign(Gtk.Align.END)
        pill_box.set_valign(Gtk.Align.END)

        dot_label = Gtk.Label(label="●")
        dot_label.add_css_class("blz-status-dot")
        dot_label.add_css_class(self.status.lower())
        pill_box.append(dot_label)

        status_text = Gtk.Label(label=self.status.upper())
        status_text.add_css_class("blz-telemetry-text")
        pill_box.append(status_text)

        sep1 = Gtk.Label(label="·")
        sep1.add_css_class("blz-telemetry-dim")
        pill_box.append(sep1)

        epoch_text = Gtk.Label(label=f"EPOCH #{self.epoch:02d}")
        epoch_text.add_css_class("blz-telemetry-text")
        pill_box.append(epoch_text)

        sep2 = Gtk.Label(label="·")
        sep2.add_css_class("blz-telemetry-dim")
        pill_box.append(sep2)

        lat_text = Gtk.Label(label=f"{self.latency:.1f}ms")
        lat_text.add_css_class("blz-telemetry-text")
        pill_box.append(lat_text)

        sep3 = Gtk.Label(label="·")
        sep3.add_css_class("blz-telemetry-dim")
        pill_box.append(sep3)

        self.queue_label = Gtk.Label(label=f"+{self.inputs_count} INP")
        self.queue_label.add_css_class("blz-telemetry-text")
        pill_box.append(self.queue_label)

        overlay.add_overlay(pill_box)
        self.window.set_child(overlay)

        # 4. Input Interception (Clicks & Keystrokes)
        click_gesture = Gtk.GestureClick.new()
        click_gesture.connect("pressed", self.on_click_pressed)
        self.window.add_controller(click_gesture)

        key_controller = Gtk.EventControllerKey.new()
        key_controller.connect("key-pressed", self.on_key_pressed)
        self.window.add_controller(key_controller)

        # 5. Signal handling for graceful dismiss
        try:
            from gi.repository import GLibUnix
            GLibUnix.signal_add(GLib.PRIORITY_DEFAULT, signal.SIGTERM, self.dismiss_overlay)
            GLibUnix.signal_add(GLib.PRIORITY_DEFAULT, signal.SIGUSR1, self.dismiss_overlay)
        except Exception:
            pass

        if self.auto_dismiss_ms > 0:
            GLib.timeout_add(self.auto_dismiss_ms, self.dismiss_overlay)

        self.window.present()
        print(f"[Ghost Surface]: BCDL Kinetic Glass Overlay ekranda! (Epoch: {self.epoch}, Gecikme: {self.latency}ms)")

    def record_input(self, event_data):
        self.inputs_count += 1
        self.queue_label.set_text(f"+{self.inputs_count} INP")
        entry = {
            "epoch": self.epoch,
            "timestamp_ms": int(time.time() * 1000),
            "event": event_data
        }
        print(f"[Input Intercept]: {json.dumps(entry)}")
        if self.event_log:
            with open(self.event_log, "a") as f:
                f.write(json.dumps(entry) + "\n")

    def on_click_pressed(self, gesture, n_press, x, y):
        button = gesture.get_current_button()
        self.record_input({
            "type": "PointerClick",
            "x": x,
            "y": y,
            "button": button
        })

    def on_key_pressed(self, controller, keyval, keycode, state):
        self.record_input({
            "type": "KeyPress",
            "keycode": keycode,
            "pressed": True
        })
        return True

    def dismiss_overlay(self):
        print("[Ghost Surface]: Commit alındı, 120ms içinde gerçek sürece devrediliyor...")
        if self.window:
            self.window.close()
        self.quit()
        return GLib.SOURCE_REMOVE

def main():
    parser = argparse.ArgumentParser(description="Blaze Ghost Surface Overlay")
    parser.add_argument("--image", default="/tmp/bctp_ghost_surface.jpg", help="Captured JPEG path")
    parser.add_argument("--epoch", type=int, default=2, help="Fencing Epoch")
    parser.add_argument("--latency", type=float, default=1.2, help="T_ghost latency in ms")
    parser.add_argument("--inputs", type=int, default=0, help="Initial input queue count")
    parser.add_argument("--status", default="syncing", choices=["syncing", "stable", "error"], help="HUD status")
    parser.add_argument("--dismiss-after-ms", type=int, default=0, help="Auto dismiss timer in ms (0 for persistent)")
    parser.add_argument("--event-log", default="/tmp/bctp_input_events.jsonl", help="Input replay JSON log")
    args = parser.parse_args()

    app = GhostOverlayApp(
        image_path=args.image,
        epoch=args.epoch,
        latency=args.latency,
        inputs_count=args.inputs,
        status=args.status,
        auto_dismiss_ms=args.dismiss_after_ms,
        event_log=args.event_log
    )
    chmod_x = os.chmod(sys.argv[0], 0o755)
    return app.run(None)

if __name__ == "__main__":
    main()
