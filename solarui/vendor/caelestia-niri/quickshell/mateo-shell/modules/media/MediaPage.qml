import QtQuick
import QtQuick.Layouts
import "../../config"
import "../../services"

MediaCard {
    id: root
    anchors.fill: parent

    function _formatTime(seconds) {
        const s = Math.max(0, Math.floor(seconds));
        const m = Math.floor(s / 60);
        const r = s % 60;
        return m + ":" + (r < 10 ? "0" : "") + r;
    }

    RowLayout {
        width: parent.width
        spacing: 16

        AlbumArt {
            size: 96
            artUrl: MediaService.artUrl
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4

            Text {
                Layout.fillWidth: true
                text: MediaService.title
                color: Theme.textPrimary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                font.bold: true
                elide: Text.ElideRight
            }

            Text {
                Layout.fillWidth: true
                text: MediaService.artist
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                elide: Text.ElideRight
            }
        }
    }

    ColumnLayout {
        width: parent.width
        spacing: 6

        Rectangle {
            Layout.fillWidth: true
            height: 6
            radius: 3
            color: Theme.surfaceVariant

            Rectangle {
                height: parent.height
                radius: 3
                color: Theme.accent
                width: parent.width * (MediaService.length > 0
                    ? Math.min(1, MediaService.position / MediaService.length)
                    : 0)

                Behavior on width { NumberAnimation { duration: 200 } }
            }
        }

        RowLayout {
            width: parent.width

            Text {
                text: root._formatTime(MediaService.position)
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
            }

            Item { Layout.fillWidth: true }

            Text {
                text: root._formatTime(MediaService.length)
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
            }
        }
    }

    PlaybackControls {
        Layout.alignment: Qt.AlignHCenter
    }
}
