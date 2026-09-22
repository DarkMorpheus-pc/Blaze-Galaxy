pragma Singleton
import QtQuick
import Quickshell.Services.Notifications

Item {
    id: root

    property var notifications: []
    readonly property int _maxStored: 50

    function dismiss(id) {
        root.notifications = root.notifications.filter(n => n.id !== id);
    }

    function clearAll() {
        root.notifications = [];
    }

    function _urgencyToString(urgency) {
        switch (urgency) {
            case NotificationUrgency.Low: return "low";
            case NotificationUrgency.Critical: return "critical";
            default: return "normal";
        }
    }

    // The actual org.freedesktop.Notifications D-Bus provider — Quickshell-
    // native, not an external daemon. Only one process on the session bus
    // can own this name; stop dunst/mako/etc. before testing.
    NotificationServer {
        id: notifServer

        bodySupported: true
        actionsSupported: false
        imageSupported: false

        onNotification: notification => {
            notification.tracked = true;

            const entry = {
                id: notification.id,
                appName: notification.appName || "Unknown",
                summary: notification.summary || "",
                body: notification.body || "",
                urgency: root._urgencyToString(notification.urgency),
                timestamp: Date.now()
            };

            let updated = [entry].concat(root.notifications);
            if (updated.length > root._maxStored) {
                updated = updated.slice(0, root._maxStored);
            }
            root.notifications = updated;

            notification.closed.connect(function () {
                root.dismiss(notification.id);
            });
        }
    }
}
