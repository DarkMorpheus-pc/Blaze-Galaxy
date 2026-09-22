import QtQuick
import QtQuick.Layouts
import QtQuick.Effects
import Quickshell
import Quickshell.Io
import "../../config"
import "../../services"

PanelWindow {
    id: sessionWindow
    screen: Quickshell.screens[0]   // injected by the Variants{} in shell.qml

    anchors { top: true; right: true; bottom: true }
    exclusiveZone: 0
    color: "transparent"

    property bool showWindow: false
    visible: showWindow

    Connections {
        target: ShellState
        function onSessionOpenChanged() {
            if (ShellState.sessionOpen) showWindow = true;
        }
    }

    IpcHandler {
        target: "session"
        function toggle() { ShellState.toggleSession() }
        function open() { ShellState.openSession() }
        function close() { ShellState.closeSession() }
    }

    Rectangle {
        id: panel
        width: 300
        height: parent.height - 2 * Theme.frameMargin
        anchors.right: parent.right
        anchors.verticalCenter: parent.verticalCenter
        anchors.rightMargin: Theme.frameMargin

        radius: Theme.radiusLarge
        color: Theme.surface
        opacity: ShellState.sessionOpen ? 0.96 : 0
        scale: ShellState.sessionOpen ? 1.0 : 0.96
        border.width: 1
        border.color: Theme.outline

        Behavior on opacity {
            NumberAnimation {
                duration: 180
                easing.type: Easing.OutCubic
                onFinished: if (!ShellState.sessionOpen) sessionWindow.showWindow = false
            }
        }
        Behavior on scale {
            NumberAnimation { duration: 180; easing.type: Easing.OutCubic }
        }

        layer.enabled: true
        layer.effect: MultiEffect {
            shadowEnabled: true
            shadowColor: "#33000000"
            shadowBlur: 0.5
            shadowVerticalOffset: 4
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 14

            Text {
                text: "Notifications"
                color: Theme.textPrimary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                font.bold: true
            }

            // Placeholder — real NotificationService wiring is a follow-up step
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 100
                radius: Theme.radiusMedium
                color: Theme.background
                border.width: 1
                border.color: Theme.outline

                Text {
                    anchors.centerIn: parent
                    text: "No Notifications"
                    color: Theme.textSecondary
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                }
            }

            KeepAwakeCard {
                Layout.fillWidth: true
            }

            Item { Layout.fillHeight: true }

            Text {
                text: "Quick Toggles"
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                font.bold: true
            }

            GridLayout {
                Layout.fillWidth: true
                columns: 3
                rowSpacing: 8
                columnSpacing: 8

                QuickToggle {
                    iconText: "📶"
                    active: SessionService.wifiEnabled
                    onClicked: SessionService.toggleWifi()
                }
                QuickToggle {
                    iconText: "🔷"
                    active: SessionService.bluetoothEnabled
                    onClicked: SessionService.toggleBluetooth()
                }
                QuickToggle {
                    iconText: "🌙"
                    active: SessionService.doNotDisturb
                    onClicked: SessionService.toggleDoNotDisturb()
                }
                QuickToggle {
                    iconText: "⚙"
                    active: false
                    onClicked: SessionService.openSettings()
                }
                QuickToggle {
                    iconText: "✈"
                    active: SessionService.airplaneMode
                    onClicked: SessionService.toggleAirplaneMode()
                }
                QuickToggle {
                    iconText: "🎙"
                    active: SessionService.micMuted
                    onClicked: SessionService.toggleMicMute()
                }
            }
        }
    }
}
