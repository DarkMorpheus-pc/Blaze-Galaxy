import QtQuick
import Quickshell.Io
import "../../config"

Text {
    id: root
    property int capacity: 0
    property bool charging: false

    color: Theme.textPrimary
    font.family: Theme.fontFamily
    font.pixelSize: Theme.fontSizeSmall
    text: "BAT " + capacity + "%" + (charging ? " +" : "")
    visible: capacity > 0   // hides cleanly on desktops with no battery

    FileView {
        id: capacityFile
        path: "/sys/class/power_supply/BAT0/capacity"
        onLoaded: root.capacity = parseInt(text().trim())
    }

    FileView {
        id: statusFile
        path: "/sys/class/power_supply/BAT0/status"
        onLoaded: root.charging = text().trim() === "Charging"
    }

    Timer {
        interval: 5000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: { capacityFile.reload(); statusFile.reload(); }
    }
}
