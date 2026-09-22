pragma Singleton
import QtQuick

Item {
    id: root
    property bool enabled: false
    function toggle() { enabled = !enabled }
}
