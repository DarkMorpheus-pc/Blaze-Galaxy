import QtQuick
import QtQuick.Layouts
import QtQuick.Effects
import Quickshell
import Quickshell.Io
import "../../config"
import "../../services"

PanelWindow {
    id: powerWindow
    screen: Quickshell.screens[0]

    anchors { top: true; left: true; right: true; bottom: true }
    exclusiveZone: 0
    color: "transparent"

    property bool showWindow: false
    visible: showWindow

    Connections {
        target: ShellState
        function onPowerMenuOpenChanged() {
            if (ShellState.powerMenuOpen) showWindow = true;
        }
    }

    IpcHandler {
        target: "power"
        function toggle() { ShellState.togglePowerMenu() }
        function open() { ShellState.openPowerMenu() }
        function close() { ShellState.closePowerMenu() }
    }

    MouseArea {
        anchors.fill: parent
        onClicked: ShellState.closePowerMenu()
    }

    Rectangle {
        id: panel
        width: 420
        height: 140
        anchors.centerIn: parent

        radius: Theme.radiusLarge
        color: Theme.surface
        opacity: ShellState.powerMenuOpen ? 0.96 : 0
        scale: ShellState.powerMenuOpen ? 1.0 : 0.96
        border.width: 1
        border.color: Theme.outline

        Behavior on opacity {
            NumberAnimation {
                duration: 160
                easing.type: Easing.OutCubic
                onFinished: if (!ShellState.powerMenuOpen) powerWindow.showWindow = false
            }
        }
        Behavior on scale {
            NumberAnimation { duration: 160; easing.type: Easing.OutCubic }
        }

        layer.enabled: true
        layer.effect: MultiEffect {
            shadowEnabled: true
            shadowColor: "#33000000"
            shadowBlur: 0.5
            shadowVerticalOffset: 4
        }

        MouseArea {
            anchors.fill: parent
            onClicked: {}
        }

        Row {
            anchors.centerIn: parent
            spacing: 16

            PowerButton {
                iconText: "⎋"
                label: "Logout"
                onClicked: PowerService.logout()
            }
            PowerButton {
                iconText: "⚿"
                label: "Lock"
                onClicked: PowerService.lock()
            }
            PowerButton {
                iconText: "☾"
                label: "Suspend"
                onClicked: PowerService.suspend()
            }
            PowerButton {
                iconText: "⟲"
                label: "Reboot"
                onClicked: PowerService.reboot()
            }
            PowerButton {
                iconText: "⏻"
                label: "Shutdown"
                onClicked: PowerService.shutdown()
            }
        }
    }
}
