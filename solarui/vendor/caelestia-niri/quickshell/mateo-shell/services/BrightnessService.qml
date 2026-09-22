pragma Singleton
import QtQuick

Item {
    id: root

    property real brightness: 70   // 0-100, local state only for now

    function setBrightness(value) {
        // TODO: wire to real brightness backend (e.g. brightnessctl) in a later step
        root.brightness = Math.max(0, Math.min(100, value))
    }
}
