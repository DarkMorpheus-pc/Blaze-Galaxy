pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

Item {
    id: root

    property string title: "No media"
    property string artist: ""
    property string album: ""
    property bool playing: false
    property real position: 0   // seconds
    property real length: 0     // seconds
    property string artUrl: ""

    function playPause() { Quickshell.execDetached(["playerctl", "play-pause"]) }
    function next() { Quickshell.execDetached(["playerctl", "next"]) }
    function previous() { Quickshell.execDetached(["playerctl", "previous"]) }

    readonly property string _sep: "\u001f"

    // Live updates: playerctl re-emits this format string whenever the
    // player's PropertiesChanged signal fires (metadata OR status),
    // same long-lived Process + SplitParser shape as NiriService's
    // event-stream listener.
    Process {
        id: metadataStream
        command: [
            "playerctl", "--follow", "metadata",
            "--format",
            "{{status}}" + root._sep + "{{xesam:title}}" + root._sep +
            "{{xesam:artist}}" + root._sep + "{{xesam:album}}" + root._sep +
            "{{mpris:length}}" + root._sep + "{{mpris:artUrl}}"
        ]
        running: true

        stdout: SplitParser {
            splitMarker: "\n"
            onRead: line => root._parseMetadata(line)
        }

        onExited: metadataRestart.start()
    }

    Timer {
        id: metadataRestart
        interval: 1500
        onTriggered: metadataStream.running = true
    }

    function _parseMetadata(line) {
        if (!line) return;
        const parts = line.split(root._sep);
        if (parts.length < 6) return;

        root.playing = parts[0] === "Playing";
        root.title = parts[1] || "No media";
        root.artist = parts[2] || "";
        root.album = parts[3] || "";
        root.length = parts[4] ? parseInt(parts[4]) / 1000000 : 0;
        root.artUrl = parts[5] || "";
    }

    // Position isn't pushed by --follow (no signal fires per-second),
    // so it's polled lightly — and only while actually playing, same
    // "avoid unnecessary processes" discipline as SystemService's disk poll.
    Process {
        id: positionProc
        command: ["playerctl", "position"]
        stdout: StdioCollector {
            onStreamFinished: {
                const v = parseFloat(text);
                if (!isNaN(v)) root.position = v;
            }
        }
    }

    Timer {
        interval: 1000
        running: root.playing
        repeat: true
        triggeredOnStart: true
        onTriggered: positionProc.running = true
    }
}
