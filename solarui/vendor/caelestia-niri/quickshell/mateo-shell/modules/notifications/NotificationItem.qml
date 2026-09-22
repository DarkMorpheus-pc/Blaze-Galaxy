import QtQuick
import "../../config"

Rectangle {
    id: root

    property string appName: ""
    property string title: ""
    property string body: ""
    property string relativeTime: ""

    signal dismissed()

    width: parent ? parent.width : 0
    height: contentColumn.implicitHeight + 20
    radius: Theme.radiusSmall
    color: Theme.surfaceVariant
    border.width: 1
    border.color: Theme.outline

    scale: hoverArea.containsMouse ? 1.02 : 1.0
    Behavior on scale {
        NumberAnimation { duration: 150; easing.type: Easing.OutCubic }
    }

    MouseArea {
        id: hoverArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.NoButton
    }

    Column {
        id: contentColumn
        anchors.left: parent.left
        anchors.right: dismissButton.left
        anchors.verticalCenter: parent.verticalCenter
        anchors.margins: 10
        anchors.rightMargin: 8
        spacing: 3

        Row {
            spacing: 8
            width: parent.width

            Text {
                text: root.appName
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                font.bold: true
            }

            Text {
                text: root.relativeTime
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
            }
        }

        Text {
            width: parent.width
            text: root.title
            color: Theme.textPrimary
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeNormal
            font.bold: true
            elide: Text.ElideRight
        }

        Text {
            width: parent.width
            text: root.body
            color: Theme.textSecondary
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
            wrapMode: Text.WordWrap
            maximumLineCount: 2
            elide: Text.ElideRight
        }
    }

    Rectangle {
        id: dismissButton
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: 8
        width: 22
        height: 22
        radius: 11
        color: dismissHover.containsMouse ? Theme.outline : "transparent"

        Behavior on color { ColorAnimation { duration: 120 } }

        Text {
            anchors.centerIn: parent
            text: "×"
            color: Theme.textSecondary
            font.pixelSize: 14
        }

        MouseArea {
            id: dismissHover
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: root.dismissed()
        }
    }
}
