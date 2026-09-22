import QtQuick
import "../../config"
import "../../services"

Rectangle {
    id: root

    height: 56
    radius: Theme.radiusMedium
    color: Theme.background
    border.width: 1
    border.color: Theme.outline

    Row {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 10

        Text {
            text: "☕"
            font.pixelSize: 18
            anchors.verticalCenter: parent.verticalCenter
        }

        Column {
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            Text {
                text: "Keep Awake"
                color: Theme.textPrimary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                font.bold: true
            }

            Text {
                text: SessionService.keepAwake ? "Preventing sleep mode" : "Normal power management"
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
            }
        }
    }

    Rectangle {
        id: toggleSwitch
        anchors.right: parent.right
        anchors.rightMargin: 12
        anchors.verticalCenter: parent.verticalCenter
        width: 40
        height: 22
        radius: 11
        color: SessionService.keepAwake ? Theme.accent : Theme.surfaceVariant

        Behavior on color { ColorAnimation { duration: 120 } }

        Rectangle {
            width: 18
            height: 18
            radius: 9
            color: Theme.surface
            y: 2
            x: SessionService.keepAwake ? parent.width - width - 2 : 2
            Behavior on x { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: SessionService.toggleKeepAwake()
        }
    }
}
