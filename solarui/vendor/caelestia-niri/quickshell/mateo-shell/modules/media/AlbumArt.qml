import QtQuick
import "../../config"

Rectangle {
    id: root

    property int size: 96
    property string artUrl: ""

    width: size
    height: size
    radius: Theme.radiusMedium
    color: Theme.surfaceVariant
    border.width: 1
    border.color: Theme.outline
    clip: true

    Image {
        anchors.fill: parent
        visible: root.artUrl.length > 0
        source: root.artUrl
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
    }

    Text {
        anchors.centerIn: parent
        visible: root.artUrl.length === 0
        text: "♪"
        color: Theme.textSecondary
        font.pixelSize: root.size * 0.35
    }
}
