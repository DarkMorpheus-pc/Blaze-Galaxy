import QtQuick
import "../../config"

Rectangle {
    id: root

    property string name: ""
    property string comment: ""
    property string icon: ""
    signal activated()

    height: 56
    radius: Theme.radiusSmall
    color: hoverArea.containsMouse ? Theme.surfaceVariant : "transparent"
    scale: hoverArea.containsMouse ? 1.01 : 1.0

    Behavior on color { ColorAnimation { duration: 120 } }
    Behavior on scale { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }

    Row {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 12

        Rectangle {
            width: 40
            height: 40
            radius: Theme.radiusSmall
            color: Theme.surfaceVariant
            border.width: 1
            border.color: Theme.outline
            anchors.verticalCenter: parent.verticalCenter

            Image {
                anchors.fill: parent
                anchors.margins: 4
                visible: root.icon.length > 0 && root.icon.charAt(0) === "/"
                source: visible ? "file://" + root.icon : ""
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }

            Text {
                anchors.centerIn: parent
                visible: !(root.icon.length > 0 && root.icon.charAt(0) === "/")
                text: root.name.length > 0 ? root.name.charAt(0).toUpperCase() : "?"
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: 16
                font.bold: true
            }
        }

        Column {
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2
            width: parent.width - 52

            Text {
                width: parent.width
                text: root.name
                color: Theme.textPrimary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                font.bold: true
                elide: Text.ElideRight
            }

            Text {
                width: parent.width
                visible: root.comment.length > 0
                text: root.comment
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                elide: Text.ElideRight
            }
        }
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.activated()
    }
}
