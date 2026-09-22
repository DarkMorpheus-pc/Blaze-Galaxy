import QtQuick
import "../../../config"

Rectangle {
    id: root

    property string iconText: ""
    signal clicked()

    width: 40
    height: 40
    radius: Theme.radiusSmall
    color: hoverArea.containsMouse ? Theme.surfaceVariant : "transparent"
    scale: hoverArea.containsMouse ? 1.08 : 1.0

    Behavior on color { ColorAnimation { duration: 120 } }
    Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

    Text {
        anchors.centerIn: parent
        text: root.iconText
        color: Theme.textPrimary
        font.pixelSize: 16
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
