import QtQuick
import "../../config"

Text {
	id: root
	property var now: new Date()

	color: Theme.textPrimary
	font.family: Theme.fontFamily
	font.pixelSize: Theme.fontSizeNormal
	font.bold: true
	text: Qt.formatDateTime(now, "hh:mm")

	Timer {
		interval: 1000
		running: true
		repeat: true
		onTriggered: root.now = new Date()
	}
}
