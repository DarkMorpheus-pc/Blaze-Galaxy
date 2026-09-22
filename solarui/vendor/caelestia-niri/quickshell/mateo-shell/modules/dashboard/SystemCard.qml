import QtQuick
import QtQuick.Layouts
import "../../config"

DashboardCard {
    id: root
    title: "System"

    Column {
        width: parent.width
        spacing: 6

        Repeater {
            model: [
                { label: "OS", value: "CachyOS" },
                { label: "WM", value: "Niri" },
                { label: "USER", value: "minx" },
                { label: "UPTIME", value: "Loading..." }
            ]

            delegate: RowLayout {
                required property var modelData
                width: parent.width
                spacing: 8

                Text {
                    text: modelData.label
                    color: Theme.textSecondary
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                    Layout.preferredWidth: 60
                }

                Text {
                    text: modelData.value
                    color: Theme.textPrimary
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                    Layout.fillWidth: true
                }
            }
        }

        Item {
            width: 1
            height: 6
        }

        Row {
            spacing: 10

            Repeater {
                model: ["⏻", "⟲", "⎋", "⚿"]

                delegate: Rectangle {
                    required property string modelData

                    width: 34
                    height: 34
                    radius: Theme.radiusSmall

                    color: Theme.surface
                    border.width: 1
                    border.color: Theme.outline

                    Text {
                        anchors.centerIn: parent
                        text: modelData
                        color: Theme.textPrimary
                        font.pixelSize: 15
                    }
                }
            }
        }
    }
}
