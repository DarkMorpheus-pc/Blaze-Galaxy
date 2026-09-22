pragma Singleton
import QtQuick
import Quickshell.Io

Item {
    id: root

    // ---- CPU (unchanged from previous step) ----
    property real cpuUsage: 0
    property string cpuModel: "Unknown CPU"

    property real _prevIdle: 0
    property real _prevTotal: 0

    FileView {
        id: cpuInfoFile
        path: "/proc/cpuinfo"
        onLoaded: {
            const m = text().match(/^model name\s*:\s*(.+)$/m);
            if (m) root.cpuModel = m[1].trim();
        }
    }

    FileView {
        id: statFile
        path: "/proc/stat"
        onLoaded: root._parseStat(text())
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: statFile.reload()
    }

    function _parseStat(content) {
        const line = content.split("\n")[0];
        const parts = line.trim().split(/\s+/).slice(1).map(Number);
        const idle = parts[3] + parts[4];
        const total = parts.reduce((a, b) => a + b, 0);
        const dIdle = idle - root._prevIdle;
        const dTotal = total - root._prevTotal;

        if (root._prevTotal > 0 && dTotal > 0) {
            root.cpuUsage = Math.max(0, Math.min(100, 100 * (1 - dIdle / dTotal)));
        }

        root._prevIdle = idle;
        root._prevTotal = total;
    }

    // ---- RAM (new) ----
    property real ramUsage: 0
    property real ramUsed: 0    // MB
    property real ramTotal: 0   // MB

    FileView {
        id: memInfoFile
        path: "/proc/meminfo"
        onLoaded: root._parseMemInfo(text())
    }

    Timer {
        interval: 1000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: memInfoFile.reload()
    }

    function _parseMemInfo(content) {
        const totalKb = parseInt(content.match(/MemTotal:\s+(\d+)/)[1]);
        const availKb = parseInt(content.match(/MemAvailable:\s+(\d+)/)[1]);
        const usedKb = totalKb - availKb;

        root.ramTotal = totalKb / 1024;
        root.ramUsed = usedKb / 1024;
        root.ramUsage = totalKb > 0 ? (usedKb / totalKb) * 100 : 0;
    }

    // ---- Disk (new) ----
    property real diskUsage: 0
    property real diskUsed: 0    // GB
    property real diskTotal: 0   // GB

    // No sysfs equivalent exposes free/used space for a mounted filesystem
    // the way /proc does for CPU/RAM, so this is Option B: a lightweight
    // one-shot `df` call, same Process + StdioCollector shape already used
    // in modules/bar/VolumeIndicator.qml.
    Process {
        id: diskProc
        command: ["df", "-k", "/"]
        stdout: StdioCollector {
            onStreamFinished: root._parseDf(text)
        }
    }

    Timer {
        interval: 12000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: diskProc.running = true
    }

    function _parseDf(content) {
        const lines = content.trim().split("\n");
        if (lines.length < 2) return;

        const parts = lines[1].trim().split(/\s+/);
        // df -k /: Filesystem 1K-blocks Used Available Use% Mounted-on
        const totalKb = parseInt(parts[1]);
        const usedKb = parseInt(parts[2]);

        root.diskTotal = totalKb / (1024 * 1024);
        root.diskUsed = usedKb / (1024 * 1024);
        root.diskUsage = totalKb > 0 ? (usedKb / totalKb) * 100 : 0;
    }
}
