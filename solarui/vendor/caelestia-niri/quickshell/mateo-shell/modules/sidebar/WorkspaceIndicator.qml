import QtQuick
import "../../config"
import "../../services"

Column {
    id: root
    spacing: 6

    Repeater {
        model: NiriService.workspaces
        delegate: Rectangle {
            id: dot
            required property var modelData

            width: 12
            height: modelData.is_focused ? 22 : 12
            radius: 6
            color: modelData.is_focused ? Theme.accent : Theme.outline
            opacity: hoverArea.containsMouse ? 0.8 : 1.0

            // State-driven size change (focus) + hover-only opacity dim.
            Behavior on height { NumberAnimation { duration: 120; easing.type: Easing.OutCubic } }
            Behavior on color { ColorAnimation { duration: 120 } }
            Behavior on opacity { NumberAnimation { duration: 100 } }

            MouseArea {
                id: hoverArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: NiriService.focusWorkspace(modelData.idx)
            }
        }
    }
}
