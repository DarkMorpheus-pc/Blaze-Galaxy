import QtQuick
import QtQuick.Effects
import Quickshell
import "../../config"

PanelWindow {
    id: bar
    screen: modelData   // injected by the Variants{} in shell.qml

    anchors { top: true; left: true; right: true }
    margins { top: Theme.barMargin; left: Theme.barMargin; right: Theme.barMargin }

    implicitHeight: Theme.barHeight + Theme.barMargin
    exclusiveZone: implicitHeight
    color: "transparent"

    Rectangle {
        id: background
        anchors.fill: parent
        anchors.topMargin: Theme.barMargin
        radius: Theme.radiusLarge
        color: Theme.surface
        opacity: 0.94
        border.width: 1
        border.color: Theme.outline

        layer.enabled: true
        layer.effect: MultiEffect {
            shadowEnabled: true
            shadowColor: "#33000000"
            shadowBlur: 0.5
            shadowVerticalOffset: 3
        }

        Row {
            anchors.left: parent.left
            anchors.leftMargin: 14
            anchors.verticalCenter: parent.verticalCenter
            spacing: 12
            LauncherButton {}
            Workspaces {}
        }

        Clock { anchors.centerIn: parent }

        Row {
            anchors.right: parent.right
            anchors.rightMargin: 14
            anchors.verticalCenter: parent.verticalCenter
            spacing: 16
            CpuIndicator {}
            MemoryIndicator {}
            BatteryIndicator {}
            VolumeIndicator {}
        }
    }
}
