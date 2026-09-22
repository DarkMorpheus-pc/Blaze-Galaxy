import QtQuick
import Quickshell.Io
import "../../config"

Text {
    id: root
    property real usage: 0

    color: Theme.textPrimary
    font.family: Theme.fontFamily
    font.pixelSize: Theme.fontSizeSmall
    text: "RAM " + Math.round(usage) + "%"

    FileView {
        id: memFile
        path: "/proc/meminfo"
        onLoaded: root._parse(text())
    }

    Timer {
        interval: 3000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: memFile.reload()
    }

    function _parse(content) {
        const total = parseInt(content.match(/MemTotal:\s+(\d+)/)[1]);
        const avail = parseInt(content.match(/MemAvailable:\s+(\d+)/)[1]);
        usage = 100 * (total - avail) / total;
    }
}
