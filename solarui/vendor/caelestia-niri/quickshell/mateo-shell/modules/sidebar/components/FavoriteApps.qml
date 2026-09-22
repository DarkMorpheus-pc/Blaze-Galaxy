import QtQuick
import "../../../config"
import "../../../services"

Rectangle {
    id: root

    readonly property var favorites: [
        { icon: "♪", exec: "spotify" },
        { icon: "◔", exec: "discord" },
        { icon: "🦊", exec: "firefox" },
        { icon: "▢", exec: "kitty" }
    ]

    width: 56
    height: column.implicitHeight + 16
    radius: Theme.radiusMedium
    color: Theme.background
    border.width: 1
    border.color: Theme.outline

    Column {
        id: column
        anchors.centerIn: parent
        spacing: 6

        Repeater {
            model: root.favorites
            delegate: FavoriteButton {
                required property var modelData
                iconText: modelData.icon
                onClicked: AppLauncherService.launch(modelData.exec)
            }
        }
    }
}
