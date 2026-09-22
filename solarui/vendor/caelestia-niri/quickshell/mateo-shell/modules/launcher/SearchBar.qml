import QtQuick
import "../../config"
import "../../services"

Rectangle {
    id: root

    signal accepted()
    signal cancelled()

    property alias input: textInput

    height: 44
    radius: Theme.radiusSmall
    color: Theme.surfaceVariant
    border.width: 1
    border.color: Theme.outline

    Row {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 8

        Text {
            text: "⌕"
            color: Theme.textSecondary
            font.pixelSize: 16
            anchors.verticalCenter: parent.verticalCenter
        }

        Item {
            width: parent.width - 30
            height: parent.height
            anchors.verticalCenter: parent.verticalCenter

            Text {
                visible: textInput.text.length === 0
                text: "Search applications..."
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                anchors.verticalCenter: parent.verticalCenter
            }

            TextInput {
                id: textInput
                anchors.fill: parent
                verticalAlignment: TextInput.AlignVCenter
                color: Theme.textPrimary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                clip: true

                onTextChanged: AppLauncherService.searchText = text

                Keys.onEscapePressed: root.cancelled()
                Keys.onReturnPressed: root.accepted()
                Keys.onEnterPressed: root.accepted()
            }
        }
    }
}
