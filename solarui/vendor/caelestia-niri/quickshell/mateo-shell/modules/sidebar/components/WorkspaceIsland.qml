import QtQuick
import "../../../config"
import "../../../services"

Rectangle {
    id: root

    width: 56
    height: column.implicitHeight + 16
    radius: Theme.radiusMedium
    color: Theme.background
    border.width: 1
    border.color: Theme.outline

    Column {
        id: column
        anchors.centerIn: parent
        spacing: 6

        Repeater {
            model: NiriService.workspaces
            delegate: WorkspaceButton {
                required property var modelData
                idx: modelData.idx
                active: modelData.is_focused
                occupied: NiriService.isWorkspaceOccupied(modelData.id)
                appId: NiriService.firstWindowAppId(modelData.id)
                onClicked: NiriService.focusWorkspace(modelData.idx)
            }
        }
    }
}
