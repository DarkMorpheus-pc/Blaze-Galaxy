import QtQuick
import QtQuick.Layouts
import "../../config"

RowLayout {
    anchors.fill: parent
    spacing: 16

    CpuCard {
        Layout.fillWidth: true
        Layout.fillHeight: true
    }

    MemoryCard {
        Layout.fillWidth: true
        Layout.fillHeight: true
    }

    DiskCard {
        Layout.fillWidth: true
        Layout.fillHeight: true
    }
}
