import QtQuick
import QtQuick.Layouts
import "../../config"

DashboardCard {
    id: root
    title: Qt.formatDateTime(new Date(), "MMMM yyyy")

    readonly property var _today: new Date()
    readonly property int _year: _today.getFullYear()
    readonly property int _month: _today.getMonth()
    readonly property var _dayNames: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
    readonly property var _days: _buildDays()

    function _buildDays() {
        const first = new Date(_year, _month, 1);
        const firstIndex = (first.getDay() + 6) % 7; // Monday-first index
        const daysInMonth = new Date(_year, _month + 1, 0).getDate();

        let cells = [];
        for (let i = 0; i < firstIndex; i++) cells.push(null);
        for (let d = 1; d <= daysInMonth; d++) cells.push(d);
        while (cells.length % 7 !== 0) cells.push(null);
        return cells;
    }

    GridLayout {
        width: parent.width
        columns: 7
        rowSpacing: 6
        columnSpacing: 6

        Repeater {
            model: root._dayNames
            delegate: Text {
                required property string modelData
                Layout.alignment: Qt.AlignHCenter
                text: modelData
                color: Theme.textSecondary
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                font.bold: true
            }
        }

        Repeater {
            model: root._days
            delegate: Rectangle {
                required property var modelData
                Layout.preferredWidth: 26
                Layout.preferredHeight: 26
                radius: 13
                color: modelData === root._today.getDate() ? Theme.accent : "transparent"

                Text {
                    anchors.centerIn: parent
                    visible: modelData !== null
                    text: modelData ?? ""
                    color: modelData === root._today.getDate() ? Theme.background : Theme.textPrimary
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                }
            }
        }
    }
}
