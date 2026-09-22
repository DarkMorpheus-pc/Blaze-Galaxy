import QtQuick
import "../../config"

Rectangle {
    id: root

    property string iconText: ""
    property bool active: false
    signal clicked()

    width: 46
    height: 46
    radius: Theme.radiusSmall
    color: active ? Theme.accent : (hoverArea.containsMouse ? Theme.surfaceVariant : Theme.background)
    border.width: 1
    border.color: Theme.outline

    Behavior on color { ColorAnimation { duration: 120 } }

    Text {
        anchors.centerIn: parent
        text: root.iconText
        color: root.active ? Theme.background : Theme.textPrimary
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
