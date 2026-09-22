import QtQuick
import QtQuick.Layouts
import QtQuick.Effects
import Quickshell
import Quickshell.Io
import "../../config"
import "../../services"
import "../performance"
import "../media"
import "../weather"
import "../notifications"

PanelWindow {
    id: dashboardWindow
    screen: Quickshell.screens[0]

    anchors { top: true; left: true; right: true }

    implicitHeight: 480
    exclusiveZone: 0
    color: "transparent"

    property bool showWindow: false
    visible: showWindow

    Connections {
        target: ShellState
        function onDashboardOpenChanged() {
            if (ShellState.dashboardOpen) {
                showWindow = true;
            }
        }
    }

    IpcHandler {
        target: "dashboard"
        function toggle() { ShellState.toggleDashboard() }
        function open() { ShellState.openDashboard() }
        function close() { ShellState.closeDashboard() }
    }

    Rectangle {
        id: panel
        width: 640
        height: 460
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: Theme.barMargin

        radius: Theme.radiusLarge
        color: Theme.surface
        opacity: ShellState.dashboardOpen ? 0.96 : 0
        scale: ShellState.dashboardOpen ? 1.0 : 0.96
        border.width: 1
        border.color: Theme.outline

        Behavior on opacity {
            NumberAnimation {
                duration: 180
                easing.type: Easing.OutCubic
                onFinished: if (!ShellState.dashboardOpen) dashboardWindow.showWindow = false
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
            spacing: 12

            DashboardTabs {
                Layout.alignment: Qt.AlignHCenter
            }

            Loader {
                id: pageLoader
                Layout.fillWidth: true
                Layout.fillHeight: true

                opacity: item ? 1 : 0
                scale: item ? 1 : 0.98

                Behavior on opacity {
                    NumberAnimation { duration: 140; easing.type: Easing.OutCubic }
                }
                Behavior on scale {
                    NumberAnimation { duration: 140; easing.type: Easing.OutCubic }
                }

                sourceComponent: {
                    switch (ShellState.activeTab) {
                        case "Dashboard":     return dashboardPageComponent;
                        case "Performance":   return performancePageComponent;
                        case "Media":         return mediaPageComponent;
                        case "Weather":       return weatherPageComponent;
                        case "Notifications": return notificationsPageComponent;
                        default:              return null;
                    }
                }
            }
        }
    }

    Component { id: dashboardPageComponent; DashboardPage {} }
    Component { id: performancePageComponent; PerformancePage {} }
    Component { id: mediaPageComponent; MediaPage {} }
    Component { id: weatherPageComponent; WeatherPage {} }
    Component { id: notificationsPageComponent; NotificationsPage {} }
}
