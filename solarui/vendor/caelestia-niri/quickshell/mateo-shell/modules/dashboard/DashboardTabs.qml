import QtQuick
import QtQuick.Layouts
import "../../config"
import "../../services"

Row {
    id: root
    spacing: 4

    readonly property var tabNames: ["Dashboard", "Media", "Performance", "Weather", "Notifications"]

    Repeater {
        model: root.tabNames
        delegate: Rectangle {
            id: tabButton
            required property string modelData

            readonly property bool active: ShellState.activeTab === modelData

            width: label.implicitWidth + 24
            height: 30
            radius: Theme.radiusSmall
            color: active ? Theme.surfaceVariant
                 : mouseArea.containsMouse ? Theme.background
                 : "transparent"

            Behavior on color { ColorAnimation { duration: 120 } }

            Text {
                id: label
                anchors.centerIn: parent
                text: tabButton.modelData
                color: tabButton.active ? Theme.textPrimary : Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                font.bold: tabButton.active
            }

            MouseArea {
                id: mouseArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: ShellState.setTab(tabButton.modelData)
            }
        }
    }
}
