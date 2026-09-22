import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import qs.components
import qs.components.controls
import qs.services
import qs.modules.nexus.common
import Caelestia.Config

PageBase {
    id: root
    
    title: qsTr("Noctalia Settings")
    
    property var settingsData: ({})
    property bool isLoading: true
    
    property string currentEngine: "hybrid"
    
    // Disable inputs if engine is pure caelestia
    readonly property bool isNoctaliaActive: currentEngine === "noctalia" || currentEngine === "hybrid"
    
    function setOption(key: string, val: var) {
        Quickshell.execDetached(["solar-shell", "noctalia-config", "set", key, val.toString()]);
        // Reload noctalia to apply settings
        Quickshell.execDetached(["solar-shell", "noctalia-config", "reload"]);
    }
    
    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraLarge
        visible: !root.isLoading
        opacity: isNoctaliaActive ? 1.0 : 0.5
        enabled: isNoctaliaActive
        
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
            id: loadProcess
            running: true
            command: ["solar-shell", "noctalia-config", "get"]
            stdout: StdioCollector {
                onStreamFinished: {
                    try {
                        root.settingsData = JSON.parse(text);
                    } catch(e) {
                        console.error("Failed to parse noctalia settings", e);
                    }
                    root.isLoading = false;
                }
            }
        }
        
        Item {
            visible: !isNoctaliaActive
            Layout.fillWidth: true
            implicitHeight: warningLabel.implicitHeight + Tokens.padding.large * 2
            
            StyledRect {
                anchors.fill: parent
                color: Colours.palette.m3errorContainer
                radius: Tokens.rounding.medium
            }
            
            StyledText {
                id: warningLabel
                anchors.centerIn: parent
                text: qsTr("Noctalia motoru şu an kapalı! Ayarları değiştirebilmek için 'SolarUI Engine' sekmesinden Noctalia veya Hibrit moduna geçmelisiniz.")
                color: Colours.palette.m3onErrorContainer
                font: Tokens.font.body.medium
                wrapMode: Text.WordWrap
                width: parent.width - Tokens.padding.large * 2
                horizontalAlignment: Text.AlignHCenter
            }
        }
        
        ColumnLayout {
            spacing: 0
            Layout.fillWidth: true
            
            SectionHeader {
                text: qsTr("Animations & Visuals")
            }
            
            RowButton {
                text: qsTr("Animation Speed")
                subtext: qsTr("Current multiplier: ") + (root.settingsData?.shell?.animation?.speed ?? 1.0)
                icon: "animation"
                onClicked: {
                    let current = root.settingsData?.shell?.animation?.speed ?? 1.0;
                    let next = current < 1.0 ? 1.0 : 0.5;
                    setOption("shell.animation.speed", next);
                    loadProcess.running = true;
                }
            }
            
            RowButton {
                text: qsTr("Corner Radius Scale")
                subtext: qsTr("Scale: ") + (root.settingsData?.shell?.corner_radius_scale ?? 1.0)
                icon: "rounded_corner"
                onClicked: {
                    let current = root.settingsData?.shell?.corner_radius_scale ?? 1.0;
                    let next = current > 1.0 ? 1.0 : 1.15;
                    setOption("shell.corner_radius_scale", next);
                    loadProcess.running = true;
                }
            }
            
            RowButton {
                text: qsTr("Panel Transparency Mode")
                subtext: qsTr("Mode: ") + (root.settingsData?.shell?.panel?.transparency_mode ?? "glass")
                icon: "opacity"
                onClicked: {
                    let current = root.settingsData?.shell?.panel?.transparency_mode ?? "glass";
                    let next = current === "glass" ? "solid" : "glass";
                    setOption("shell.panel.transparency_mode", next);
                    loadProcess.running = true;
                }
            }
        }
        
        ColumnLayout {
            spacing: 0
            Layout.fillWidth: true
            
            SectionHeader {
                text: qsTr("Accessibility")
            }
            
            RowButton {
                text: qsTr("UI Scale")
                subtext: qsTr("Global scaling factor: ") + (root.settingsData?.accessibility?.ui_scale ?? 1.0)
                icon: "format_size"
                onClicked: {
                    let current = root.settingsData?.accessibility?.ui_scale ?? 1.0;
                    let next = current < 1.0 ? 1.0 : 0.9;
                    setOption("accessibility.ui_scale", next);
                    loadProcess.running = true;
                }
            }
        }
        
        ColumnLayout {
            spacing: 0
            Layout.fillWidth: true
            
            SectionHeader {
                text: qsTr("Widgets & Features")
            }
            
            ToggleRow {
                text: qsTr("Screen Time Tracking")
                subtext: qsTr("Enabled: ") + (root.settingsData?.shell?.screen_time_enabled ?? true)
                checked: root.settingsData?.shell?.screen_time_enabled ?? true
                onClicked: {
                    let current = root.settingsData?.shell?.screen_time_enabled ?? true;
                    setOption("shell.screen_time_enabled", !current);
                    loadProcess.running = true;
                }
            }
            
            ToggleRow {
                text: qsTr("Desktop Grid Widgets")
                subtext: qsTr("Visible: ") + (root.settingsData?.desktop_widgets?.grid?.visible ?? true)
                checked: root.settingsData?.desktop_widgets?.grid?.visible ?? true
                onClicked: {
                    let current = root.settingsData?.desktop_widgets?.grid?.visible ?? true;
                    setOption("desktop_widgets.grid.visible", !current);
                    loadProcess.running = true;
                }
            }
        }
        
        ColumnLayout {
            spacing: 0
            Layout.fillWidth: true
            
            SectionHeader {
                text: qsTr("Advanced Config")
            }
            
            RowButton {
                text: qsTr("Open Native Noctalia Settings")
                subtext: qsTr("Launch the full C++ native configuration app for complex layouts.")
                icon: "settings_applications"
                onClicked: {
                    Quickshell.execDetached(["noctalia", "msg", "settings-toggle"]);
                }
            }
        }
    }
}
