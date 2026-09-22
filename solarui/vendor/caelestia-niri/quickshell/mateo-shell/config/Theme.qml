pragma Singleton
import QtQuick

QtObject {
	readonly property color background: "#e8e8e8"
	readonly property color surface: "#f2f2f2"
	readonly property color surfaceVariant: "#dcdcdc"
	readonly property color outline: "#c9c9c9"
	readonly property color textPrimary: "#1a1a1a"
	readonly property color textSecondary: "#6a6a6a"
	readonly property color accent: "#2b2b2b"

	readonly property int radiusLarge: 22
	readonly property int radiusSmall: 10
	readonly property int radiusMedium: 16

	readonly property int barHeight: 38
	readonly property int barMargin: 8

	readonly property string fontFamily: "JetBrainsMono Nerd Font"
	readonly property int fontSizeSmall: 11
	readonly property int fontSizeNormal: 13
}
