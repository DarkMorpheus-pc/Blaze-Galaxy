import QtQuick
import QtQuick.Layouts
import QtQuick.Effects
import Quickshell
import "../../config"
import "../../services"

PanelWindow {
    id: osdWindow
    screen: Quickshell.screens[0]

    readonly property int triggerZoneWidth: 12
    readonly property int panelWidth: 220

    anchors { top: true; right: true; bottom: true }
    exclusiveZone: 0
    implicitWidth: triggerZoneWidth + panelWidth + Theme.frameMargin
    color: "transparent"

    // Mouse-proximity trigger — hovering the thin strip at the screen's
    // right edge reveals the panel. Pure UI mechanics, no system polling.
    MouseArea {
        id: triggerZone
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: osdWindow.triggerZoneWidth
        hoverEnabled: true
        onEntered: ShellState.osdVisible = true
    }

    MouseArea {
        id: panelHoverZone
        anchors.fill: panel
        hoverEnabled: true
        onExited: ShellState.osdVisible = false
    }

    Rectangle {
        id: panel
        width: osdWindow.panelWidth
        height: 160
        anchors.verticalCenter: parent.verticalCenter
        anchors.right: parent.right
        anchors.rightMargin: ShellState.osdVisible ? Theme.frameMargin : -width

        radius: Theme.radiusLarge
        color: Theme.surface
        opacity: 0.96
        border.width: 1
        border.color: Theme.outline

        Behavior on anchors.rightMargin {
            NumberAnimation { duration: 200; easing.type: Easing.OutCubic }
        }

        layer.enabled: true
        layer.effect: MultiEffect {
            shadowEnabled: true
            shadowColor: "#33000000"
            shadowBlur: 0.5
            shadowVerticalOffset: 3
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 18

            VolumeSlider { Layout.fillWidth: true }
            BrightnessSlider { Layout.fillWidth: true }
        }
    }
}
