import QtQuick
import qs.services
import qs.modules.common

Image {
    id: root

    property real implicitSize: 26
    property bool animated: true
    implicitWidth: implicitSize
    implicitHeight: implicitSize
    fillMode: Image.PreserveAspectFit
    asynchronous: true
    cache: true
    smooth: true
}
