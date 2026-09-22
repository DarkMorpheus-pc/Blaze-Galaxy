pragma Singleton
import QtQuick

Item {
    id: root
    property bool enabled: true
    function toggle() { enabled = !enabled }
}
