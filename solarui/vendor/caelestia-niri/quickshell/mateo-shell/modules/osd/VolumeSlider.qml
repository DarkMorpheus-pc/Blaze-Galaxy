import QtQuick
import "../../config"
import "../../services"

Column {
    width: parent ? parent.width : 200
    spacing: 6

    Row {
        width: parent.width
        Text {
            text: "🔊 Volume"
            color: Theme.textSecondary
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
        }
    }

    Rectangle {
        width: parent.width
        height: 6
        radius: 3
        color: Theme.surfaceVariant

        Rectangle {
            width: parent.width * (AudioService.volume / 100)
            height: parent.height
            radius: 3
            color: Theme.accent
        }

        MouseArea {
            anchors.fill: parent
            onPressed: mouse => AudioService.setVolume((mouse.x / width) * 100)
            onPositionChanged: mouse => {
                if (pressed) AudioService.setVolume((mouse.x / width) * 100)
            }
        }
    }
}
