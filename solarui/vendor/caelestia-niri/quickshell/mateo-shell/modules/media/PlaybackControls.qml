import QtQuick
import QtQuick.Layouts
import "../../config"
import "../../services"

Row {
    id: root
    spacing: 14

    Repeater {
        model: [
            { icon: "⏮", isMain: false, action: "previous" },
            { icon: "", isMain: true, action: "playPause" },
            { icon: "⏭", isMain: false, action: "next" }
        ]
        delegate: Rectangle {
            id: btn
            required property var modelData
            required property int index

            width: modelData.isMain ? 44 : 36
            height: width
            radius: width / 2
            color: hoverArea.containsMouse ? Theme.surfaceVariant : Theme.background
            border.width: 1
            border.color: Theme.outline
            scale: hoverArea.containsMouse ? 1.08 : 1.0

            Behavior on color { ColorAnimation { duration: 120 } }
            Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

            Text {
                anchors.centerIn: parent
                text: btn.modelData.isMain ? (MediaService.playing ? "⏸" : "▶") : btn.modelData.icon
                color: Theme.textPrimary
                font.pixelSize: btn.modelData.isMain ? 16 : 14
            }

            MouseArea {
                id: hoverArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    switch (btn.modelData.action) {
                        case "playPause": MediaService.playPause(); break;
                        case "next": MediaService.next(); break;
                        case "previous": MediaService.previous(); break;
                    }
                }
            }
        }
    }
}
