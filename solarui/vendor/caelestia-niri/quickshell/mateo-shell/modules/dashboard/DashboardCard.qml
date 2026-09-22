import QtQuick
import QtQuick.Effects
import "../../config"

Rectangle {
    id: root

    default property alias content: contentColumn.children
    property string title: ""

    radius: Theme.radiusMedium
    color: Theme.background
    border.width: 1
    border.color: Theme.outline

    scale: hoverArea.containsMouse ? 1.02 : 1.0
    Behavior on scale {
        NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
    }

    layer.enabled: true
    layer.effect: MultiEffect {
        shadowEnabled: true
        shadowColor: "#26000000"
        shadowBlur: 0.4
        shadowVerticalOffset: 2
    }

    // Hover tracking only — declared first so it sits beneath SystemCard's
    // session buttons and any other interactive children in `content`.
    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.NoButton
    }

    Column {
        id: contentColumn
        anchors.fill: parent
        anchors.margins: 14
        spacing: 8

        Text {
            visible: root.title.length > 0
            text: root.title
            color: Theme.textSecondary
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
            font.bold: true
        }
    }
}
