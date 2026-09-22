pragma Singleton
import QtQuick

Item {
    id: root

    property real volume: 50   // 0-100, local state only for now
    property bool muted: false

    function setVolume(value) {
        // TODO: wire to real audio backend (e.g. wpctl) in a later step
        root.volume = Math.max(0, Math.min(100, value))
    }

    function toggleMute() {
        root.muted = !root.muted
    }
}
