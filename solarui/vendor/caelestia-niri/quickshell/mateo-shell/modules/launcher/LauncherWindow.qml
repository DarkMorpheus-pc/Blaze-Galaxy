import QtQuick
import QtQuick.Layouts
import QtQuick.Effects
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import "../../config"
import "../../services"

PanelWindow {
    id: launcherWindow
    screen: Quickshell.screens[0]   // injected by the Variants{} in shell.qml

    anchors { top: true; left: true; right: true; bottom: true }
    exclusiveZone: 0
    color: "transparent"

    property bool showWindow: false
    visible: showWindow

    // Grants real keyboard input to the search box while open. Verify this
    // attached-property/enum name against your Quickshell version — if it
    // doesn't compile, check `quickshell -c mateo-shell` output for the
    // correct API and this is a one-line fix.
    WlrLayershell.keyboardFocus: ShellState.launcherOpen ? WlrKeyboardFocus.OnDemand : WlrKeyboardFocus.None

    Connections {
        target: ShellState
        function onLauncherOpenChanged() {
            if (ShellState.launcherOpen) {
                showWindow = true;
                searchBar.input.text = "";
                AppLauncherService.searchText = "";
                Qt.callLater(() => searchBar.input.forceActiveFocus());
            }
        }
    }

    IpcHandler {
        target: "launcher"
        function toggle() { ShellState.toggleLauncher() }
        function open() { ShellState.openLauncher() }
        function close() { ShellState.closeLauncher() }
    }

    // Click outside the panel closes the launcher.
    MouseArea {
        anchors.fill: parent
        onClicked: ShellState.closeLauncher()
    }

    Rectangle {
        id: panel
        width: 480
        height: 420
        anchors.centerIn: parent

        radius: Theme.radiusLarge
        color: Theme.surface
        opacity: ShellState.launcherOpen ? 0.97 : 0
        scale: ShellState.launcherOpen ? 1.0 : 0.96
        border.width: 1
        border.color: Theme.outline

        Behavior on opacity {
            NumberAnimation {
                duration: 160
                easing.type: Easing.OutCubic
                onFinished: if (!ShellState.launcherOpen) launcherWindow.showWindow = false
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

        // Absorb clicks so they don't fall through to the outside-click MouseArea.
        MouseArea {
            anchors.fill: parent
            onClicked: {}
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 12

            SearchBar {
                id: searchBar
                Layout.fillWidth: true
                onAccepted: {
                    AppLauncherService.launchFirstMatch();
                    ShellState.closeLauncher();
                }
                onCancelled: ShellState.closeLauncher()
            }

            LauncherList {
                Layout.fillWidth: true
                Layout.fillHeight: true
            }
        }
    }
}
