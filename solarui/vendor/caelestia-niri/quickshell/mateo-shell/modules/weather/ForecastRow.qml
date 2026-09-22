import QtQuick
import QtQuick.Layouts
import "../../config"

RowLayout {
    id: root
    spacing: 10

    property var days: []

    Repeater {
        model: root.days
        delegate: Rectangle {
            id: dayCell
            required property var modelData

            Layout.fillWidth: true
            Layout.preferredHeight: 78
            radius: Theme.radiusSmall
            color: Theme.surfaceVariant
            border.width: 1
            border.color: Theme.outline

            Column {
                anchors.centerIn: parent
                spacing: 6

                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: dayCell.modelData.day
                    color: Theme.textSecondary
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                    font.bold: true
                }

                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: dayCell.modelData.icon
                    color: Theme.textSecondary
                    font.pixelSize: 18
                }

                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: dayCell.modelData.temp + "°"
                    color: Theme.textPrimary
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                }
            }
        }
    }
}
