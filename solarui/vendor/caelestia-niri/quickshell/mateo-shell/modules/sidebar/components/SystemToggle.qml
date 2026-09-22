import QtQuick
import "../../../config"

Rectangle {
    id: root

    property string iconText: ""
    property bool enabled: true
    signal clicked()

    width: 40
    height: 40
    radius: Theme.radiusSmall
    color: hoverArea.containsMouse ? Theme.surfaceVariant : "transparent"
    scale: hoverArea.containsMouse ? 1.06 : 1.0

    Behavior on color { ColorAnimation { duration: 120 } }
    Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

    Text {
        anchors.centerIn: parent
        text: root.iconText
        color: root.enabled ? Theme.textPrimary : Theme.textSecondary
        font.pixelSize: 15
    }

    // Disabled state: diagonal strike-through overlay, no separate icon glyph needed
    Rectangle {
        visible: !root.enabled
        anchors.centerIn: parent
        width: 26
        height: 1.5
        color: Theme.textSecondary
        rotation: 45
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
