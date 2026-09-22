import QtQuick
import Quickshell.Io
import "../../config"

Text {
    id: root
    property real usage: 0
    property real _prevIdle: 0
    property real _prevTotal: 0

    color: Theme.textPrimary
    font.family: Theme.fontFamily
    font.pixelSize: Theme.fontSizeSmall
    text: "CPU " + Math.round(usage) + "%"

    FileView {
        id: statFile
        path: "/proc/stat"
        onLoaded: root._parse(text())
    }

    Timer {
        interval: 2000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: statFile.reload()
    }

    function _parse(content) {
        const line = content.split("\n")[0];
        const parts = line.trim().split(/\s+/).slice(1).map(Number);
        const idle = parts[3] + parts[4];
        const total = parts.reduce((a, b) => a + b, 0);
        const dIdle = idle - _prevIdle;
        const dTotal = total - _prevTotal;
        if (_prevTotal > 0 && dTotal > 0) {
            usage = Math.max(0, Math.min(100, 100 * (1 - dIdle / dTotal)));
        }
        _prevIdle = idle;
        _prevTotal = total;
    }
}
