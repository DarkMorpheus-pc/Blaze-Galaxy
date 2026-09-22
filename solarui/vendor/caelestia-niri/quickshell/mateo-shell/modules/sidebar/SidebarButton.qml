import QtQuick
import "../../config"

Rectangle {
    id: root

    property string iconText: ""
    signal clicked()

    width: 40
    height: 40
    radius: Theme.radiusSmall
    color: mouseArea.containsMouse ? Theme.surfaceVariant : "transparent"
    scale: mouseArea.containsMouse ? 1.08 : 1.0

    // Hover-only animation — no idle/looping motion
    Behavior on color { ColorAnimation { duration: 120 } }
    Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

    Text {
        anchors.centerIn: parent
        text: root.iconText
        color: Theme.textPrimary
        font.family: Theme.fontFamily
        font.pixelSize: 16
    }

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
