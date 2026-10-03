import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import qs.components
import qs.services
import qs.modules.nexus.common
import Caelestia.Config

PageBase {
    id: root
    
    title: qsTr("SolarUI Engine")
    
    // We can use a Process to check current engine status
    property string currentEngine: "hybrid"
    property bool taskbarEnabled: false
    
    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraLarge
        
        Process {
            id: engineStatus
            running: true
            command: ["solar-shell", "engine", "get"]
            stdout: StdioCollector {
                onStreamFinished: {
                    root.currentEngine = text.trim();
                }
            }
        }
        
        Process {
            id: taskbarStatus
            running: true
            command: ["solar-shell", "taskbar", "get"]
            stdout: StdioCollector {
                onStreamFinished: {
                    root.taskbarEnabled = (text.trim() === "true");
                }
            }
        }
        
        ColumnLayout {
            spacing: 0
            Layout.fillWidth: true
            
            SectionHeader {
                text: qsTr("Active Engine")
            }
            
            RowButton {
                text: qsTr("Noctalia")
                subtext: qsTr("Pure CachyOS C++23 Native Shell. Best for raw performance and gaming.")
                icon: "speed"
                trailingIcon: root.currentEngine === "noctalia" ? "check_circle" : ""
                onClicked: {
                    Quickshell.execDetached(["niri", "msg", "action", "spawn", "--", "solar-shell", "switch", "noctalia"]);
                    engineStatus.running = true;
                }
            }
            RowButton {
                text: qsTr("Caelestia")
                subtext: qsTr("Pure Quickshell QML interface. Elegant, modern, and highly customizable.")
                icon: "auto_awesome"
                trailingIcon: root.currentEngine === "caelestia" ? "check_circle" : ""
                onClicked: {
                    Quickshell.execDetached(["niri", "msg", "action", "spawn", "--", "solar-shell", "switch", "caelestia"]);
                    engineStatus.running = true;
                }
            }
            RowButton {
                text: qsTr("Hybrid (Recommended)")
                subtext: qsTr("Noctalia top bar & widgets + Caelestia dashboard & launcher.")
                icon: "join_inner"
                trailingIcon: root.currentEngine === "hybrid" ? "check_circle" : ""
                onClicked: {
                    Quickshell.execDetached(["niri", "msg", "action", "spawn", "--", "solar-shell", "switch", "hybrid"]);
                    engineStatus.running = true;
                }
            }
        }
        
        ColumnLayout {
            spacing: 0
            Layout.fillWidth: true
            
            SectionHeader {
                text: qsTr("SolarUI Taskbar")
            }
            
            ToggleRow {
                text: qsTr("Enable KDE-style Taskbar")
                subtext: qsTr("Shows active window icons at the bottom of the screen.")
                checked: root.taskbarEnabled
                onClicked: {
                    root.taskbarEnabled = !root.taskbarEnabled;
                    Quickshell.execDetached(["solar-shell", "taskbar", "set", root.taskbarEnabled ? "true" : "false"]);
                }
            }
        }
    }
}
