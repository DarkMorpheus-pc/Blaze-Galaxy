import QtQuick
import "../../../config"
import "../../../services"

Rectangle {
    id: root

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

        SystemToggle {
            iconText: "📶"
            enabled: WifiService.enabled
            onClicked: WifiService.toggle()
        }
        SystemToggle {
            iconText: "🔷"
            enabled: BluetoothService.enabled
            onClicked: BluetoothService.toggle()
        }
        SystemToggle {
            iconText: "🎮"
            enabled: GameModeService.enabled
            onClicked: GameModeService.toggle()
        }
    }
}
