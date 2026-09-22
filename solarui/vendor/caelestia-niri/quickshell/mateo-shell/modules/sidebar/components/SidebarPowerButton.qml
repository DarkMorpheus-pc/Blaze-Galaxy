import QtQuick
import "../../../config"
import "../../../services"

Rectangle {
    id: root

    width: 56
    height: 48
    radius: Theme.radiusMedium
    color: Theme.background
    border.width: 1
    border.color: Theme.outline
    scale: hoverArea.containsMouse ? 1.05 : 1.0

    Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

    Text {
        anchors.centerIn: parent
        text: "⏻"
        color: "#c0392b"
        font.pixelSize: 18
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: ShellState.togglePowerMenu()
    }
}
