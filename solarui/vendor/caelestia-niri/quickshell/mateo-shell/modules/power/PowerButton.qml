import QtQuick
import "../../config"

Item {
    id: root

    property string iconText: ""
    property string label: ""
    signal clicked()

    width: 64
    height: 80

    Rectangle {
        id: circle
        width: 56
        height: 56
        radius: 28
        anchors.horizontalCenter: parent.horizontalCenter
        color: hoverArea.containsMouse ? Theme.accent : Theme.background
        border.width: 1
        border.color: Theme.outline
        scale: hoverArea.containsMouse ? 1.08 : 1.0

        Behavior on color { ColorAnimation { duration: 120 } }
        Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

        Text {
            anchors.centerIn: parent
            text: root.iconText
            font.pixelSize: 20
            color: hoverArea.containsMouse ? Theme.background : Theme.textPrimary
        }

        MouseArea {
            id: hoverArea
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.clicked()
        }
    }

    Text {
        anchors.top: circle.bottom
        anchors.topMargin: 6
        anchors.horizontalCenter: parent.horizontalCenter
        text: root.label
        color: Theme.textSecondary
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSizeSmall
    }
}
