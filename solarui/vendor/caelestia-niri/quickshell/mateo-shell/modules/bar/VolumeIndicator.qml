import QtQuick
import Quickshell.Io
import "../../config"

Text {
    id: root
    property int volume: 0
    property bool muted: false

    color: Theme.textPrimary
    font.family: Theme.fontFamily
    font.pixelSize: Theme.fontSizeSmall
    text: "VOL " + (muted ? "muted" : volume + "%")

    Process {
        id: proc
        command: ["wpctl", "get-volume", "@DEFAULT_AUDIO_SINK@"]
        stdout: StdioCollector {
            onStreamFinished: {
                const m = text.match(/Volume:\s*([0-9.]+)/);
                if (m) root.volume = Math.round(parseFloat(m[1]) * 100);
                root.muted = text.includes("MUTED");
            }
        }
    }

    Timer {
        interval: 2000
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: proc.running = true
    }
}
