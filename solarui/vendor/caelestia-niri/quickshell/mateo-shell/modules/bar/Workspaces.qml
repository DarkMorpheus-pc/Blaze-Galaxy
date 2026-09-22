import QtQuick
import "../../config"
import "../../services"

Row {
	spacing: 6

	Repeater {
		model: NiriService.workspaces
		delegate: Rectangle {
			width: modelData.id_focused ? 22 : 12
			height: 12
			radius: 6
			color: modelData.is_focused ? Theme.accent : Theme.outline

			Behavior on width { NumberAnimation { duration: 120; easing.type: Easing.OutCubic }}

			MouseArea {
				anchors.fill: parent
				onClicked: NiriService.focusWorkspace(modelData.idx)
			}
		}
	}
}
