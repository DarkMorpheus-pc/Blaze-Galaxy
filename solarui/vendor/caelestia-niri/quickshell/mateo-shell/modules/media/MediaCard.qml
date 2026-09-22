import QtQuick
import "../../config"

Item {
    id: root

    default property alias content: contentColumn.children

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
            spacing: 14
        }
    }
}
