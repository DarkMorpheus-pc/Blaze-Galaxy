import QtQuick
import "../../config"

Item {
    id: root

    default property alias content: contentColumn.children

    scale: hoverArea.containsMouse ? 1.02 : 1.0
    Behavior on scale {
        NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
    }

    Rectangle {
        anchors.fill: parent
        anchors.topMargin: 3
        radius: Theme.radiusMedium
        color: "#14000000"
    }

    Rectangle {
        id: card
        anchors.fill: parent
        radius: Theme.radiusMedium
        color: Theme.background
        border.width: 1
        border.color: Theme.outline

        Column {
            id: contentColumn
            anchors.fill: parent
            anchors.margins: 16
            spacing: 16
        }
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.NoButton
    }
}
