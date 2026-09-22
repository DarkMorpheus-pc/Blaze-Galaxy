import QtQuick
import QtQuick.Layouts
import "../../config"

RowLayout {
    id: root
    anchors.fill: parent
    spacing: 16

    ColumnLayout {
        Layout.preferredWidth: parent.width * 0.5
        Layout.fillHeight: true
        spacing: 16

        ClockCard {
            Layout.fillWidth: true
            Layout.preferredHeight: 140
        }

        CalendarCard {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }

    ColumnLayout {
        Layout.preferredWidth: parent.width * 0.5
        Layout.fillHeight: true
        spacing: 16

        SystemCard {
            Layout.fillWidth: true
            Layout.fillHeight: true
        }
    }
}
