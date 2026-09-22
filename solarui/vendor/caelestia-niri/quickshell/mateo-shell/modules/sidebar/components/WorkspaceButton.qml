import QtQuick
import "../../../config"

Rectangle {
    id: root

    property int idx: 0
    property bool active: false
    property bool occupied: false
    property string appId: ""
    signal clicked()

    width: 40
    height: 40
    radius: Theme.radiusSmall
    color: active ? Theme.accent : (hoverArea.containsMouse ? Theme.surfaceVariant : "transparent")
    scale: hoverArea.containsMouse ? 1.06 : 1.0

    Behavior on color { ColorAnimation { duration: 120 } }
    Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

    Text {
        anchors.centerIn: parent
        visible: !root.occupied
        text: (root.idx + 1).toString()
        color: root.active ? Theme.background : Theme.textPrimary
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSizeNormal
        font.bold: root.active
    }

    // Occupied indicator: initial-letter badge, not a real app icon —
    // resolving app_id to a themed icon needs xdg icon-theme lookup,
    // out of scope (same limitation as AppEntry's fallback in the Launcher).
    Rectangle {
        visible: root.occupied
        anchors.centerIn: parent
        width: 22
        height: 22
        radius: 6
        color: root.active ? Theme.background : Theme.surfaceVariant

        Text {
            anchors.centerIn: parent
            text: root.appId.length > 0 ? root.appId.charAt(0).toUpperCase() : "•"
            color: root.active ? Theme.accent : Theme.textPrimary
            font.family: Theme.fontFamily
            font.pixelSize: 11
            font.bold: true
        }
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
