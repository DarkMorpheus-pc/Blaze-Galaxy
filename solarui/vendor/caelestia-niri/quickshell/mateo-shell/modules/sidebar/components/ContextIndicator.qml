import QtQuick
import "../../../config"
import "../../../services"

Rectangle {
    id: root

    readonly property string _label: {
        if (!NiriService.focusedWindow) return "Desktop";
        const w = NiriService.focusedWindow;
        return w.app_id || w.title || "Desktop";
    }

    width: 56
    height: 64
    radius: Theme.radiusMedium
    color: Theme.background
    border.width: 1
    border.color: Theme.outline

    Column {
        anchors.centerIn: parent
        spacing: 4

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "🖥"
            font.pixelSize: 18
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: root._label
            color: Theme.textSecondary
            font.family: Theme.fontFamily
            font.pixelSize: 9
            width: 48
            horizontalAlignment: Text.AlignHCenter
            elide: Text.ElideRight
        }
    }
}
