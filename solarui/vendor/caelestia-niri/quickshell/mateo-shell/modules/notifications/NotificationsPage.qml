import QtQuick
import QtQuick.Layouts
import "../../config"
import "../../services"

NotificationCard {
    id: root
    anchors.fill: parent

    function _formatRelativeTime(timestamp) {
        const diffMs = Date.now() - timestamp;
        const diffSec = Math.floor(diffMs / 1000);

        if (diffSec < 60) return "just now";
        const diffMin = Math.floor(diffSec / 60);
        if (diffMin < 60) return diffMin + "m ago";
        const diffHour = Math.floor(diffMin / 60);
        if (diffHour < 24) return diffHour + "h ago";
        const diffDay = Math.floor(diffHour / 24);
        return diffDay + "d ago";
    }

    RowLayout {
        width: parent.width

        Text {
            text: "Notifications"
            color: Theme.textPrimary
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeNormal
            font.bold: true
        }

        Item { Layout.fillWidth: true }

        Rectangle {
            id: clearAllButton
            width: clearAllLabel.implicitWidth + 20
            height: 28
            radius: Theme.radiusSmall
            color: clearAllHover.containsMouse ? Theme.surfaceVariant : "transparent"
            border.width: 1
            border.color: Theme.outline

            Behavior on color { ColorAnimation { duration: 120 } }

            Text {
                id: clearAllLabel
                anchors.centerIn: parent
                text: "Clear All"
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
            }

            MouseArea {
                id: clearAllHover
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: NotificationService.clearAll()
            }
        }
    }

    Column {
        width: parent.width
        spacing: 10

        Text {
            visible: NotificationService.notifications.length === 0
            text: "No notifications"
            anchors.horizontalCenter: parent.horizontalCenter
            color: Theme.textSecondary
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
        }

        Repeater {
            model: NotificationService.notifications
            delegate: NotificationItem {
                required property var modelData

                appName: modelData.appName
                title: modelData.summary
                body: modelData.body
                relativeTime: root._formatRelativeTime(modelData.timestamp)

                onDismissed: NotificationService.dismiss(modelData.id)
            }
        }
    }
}
