import QtQuick
import Quickshell
import "../../config"

Rectangle {
	id: root
	width: 30
	height: 30
	radius: Theme.radiusSmall
	color: mouseArea.containsMouse ? Theme.surfaceVariant : "transparent"

	Text {
		anchors.centerIn: parent
		text: "^_^"
		color: Theme.textPrimary
		font.pixelSize: 16
	}

	MouseArea {
		id: mouseArea
		anchors.fill: parent
		hoverEnabled: true
		cursorShape: Qt.PointingHandCursor
		// Placeholder - swap for whatever launcher you pick (fuzzel/anyrun/etc.)
		onClicked: Quickshell.execDetached(["fuzzel"])
	}
}
