pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell
import Caelestia.Config
import Caelestia.Models
import qs.components
import qs.components.controls
import qs.components.effects
import qs.components.filedialog
import qs.components.images
import qs.services
import qs.utils

Item {
    id: root

    required property var dialog
    readonly property FileEntry currentItem: view.currentItem as FileEntry

    StyledRect {
        anchors.fill: parent
        color: Colours.tPalette.m3surfaceContainer

        layer.enabled: true
        layer.effect: Mask {
            maskSource: mask
            maskInverted: true
        }
    }

    Item {
        id: mask

        anchors.fill: parent
        layer.enabled: true
        visible: false

        Rectangle {
            anchors.fill: parent
            anchors.margins: Tokens.padding.extraSmall
            radius: Tokens.rounding.medium
        }
    }

    Loader {
        asynchronous: true
        anchors.centerIn: parent

        opacity: view.count === 0 ? 1 : 0
        active: opacity > 0

        sourceComponent: ColumnLayout {
            MaterialIcon {
                Layout.alignment: Qt.AlignHCenter
                text: "scan_delete"
                color: Colours.palette.m3outline
                fontStyle: Tokens.font.icon.builders.extraLarge.scale(2).weight(Font.Medium).build()
            }

            StyledText {
                text: qsTr("This folder is empty")
                color: Colours.palette.m3outline
                font: Tokens.font.body.builders.large.weight(Font.Medium).build()
            }
        }

        Behavior on opacity {
            Anim {
                type: Anim.DefaultEffects
            }
        }
    }

    GridView {
        id: view

        anchors.fill: parent
        anchors.margins: Tokens.padding.extraSmall + Tokens.padding.medium

        cellWidth: Sizes.itemWidth + Tokens.spacing.small
        cellHeight: Sizes.itemWidth + Tokens.spacing.large + Tokens.padding.medium * 2 + 1

        clip: true
        focus: true
        currentIndex: -1
        Keys.onEscapePressed: currentIndex = -1

        Keys.onReturnPressed: {
            if (root.dialog.selectionValid)
                root.dialog.accepted((currentItem as FileEntry).modelData.path);
        }
        Keys.onEnterPressed: {
            if (root.dialog.selectionValid)
                root.dialog.accepted((currentItem as FileEntry).modelData.path);
        }

        StyledScrollBar.vertical: StyledScrollBar {
            flickable: view
        }

        model: FileSystemModel {
            path: {
                if (root.dialog.cwd[0] === "Home")
                    return Paths.home + `/${root.dialog.cwd.slice(1).join("/")}`;
                else
                    return root.dialog.cwd.join("/");
            }
            onPathChanged: view.currentIndex = -1
        }

        delegate: FileEntry {}

        add: Transition {
            Anim {
                properties: "opacity,scale"
                from: 0
                to: 1
            }
        }

        remove: Transition {
            Anim {
                type: Anim.DefaultEffects
                property: "opacity"
                to: 0
            }
            Anim {
                property: "scale"
                to: 0.5
            }
        }

        displaced: Transition {
            Anim {
                type: Anim.DefaultEffects
                properties: "opacity,scale"
                to: 1
                easing: Tokens.anim.standardDecel
            }
            Anim {
                properties: "x,y"
            }
        }
    }

    CurrentItem {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Tokens.padding.extraSmall

        currentItem: view.currentItem
    }

    component FileEntry: StyledRect {
        id: item

        required property int index
        required property FileSystemEntry modelData

        readonly property real nonAnimHeight: iconArea.implicitHeight + name.anchors.topMargin + name.implicitHeight + Tokens.padding.medium * 2

        implicitWidth: Sizes.itemWidth
        implicitHeight: nonAnimHeight

        radius: Tokens.rounding.large
        color: Qt.alpha(Colours.tPalette.m3surfaceContainerHighest, GridView.isCurrentItem ? Colours.tPalette.m3surfaceContainerHighest.a : 0)
        z: GridView.isCurrentItem || implicitHeight !== nonAnimHeight ? 1 : 0
        clip: true

        StateLayer {
            onClicked: view.currentIndex = item.index
            onDoubleClicked: {
                if (item.modelData.isDir)
                    root.dialog.cwd.push(item.modelData.name);
                else if (root.dialog.selectionValid)
                    root.dialog.accepted(item.modelData.path);
            }
        }

        Item {
            id: iconArea

            anchors.horizontalCenter: parent.horizontalCenter
            anchors.top: parent.top
            anchors.topMargin: Tokens.padding.medium

            implicitWidth: Sizes.itemWidth - Tokens.padding.medium * 2
            implicitHeight: implicitWidth

            CachingIconImage {
                anchors.fill: parent
                visible: item.modelData.isImage
                source: item.modelData.isImage ? Qt.resolvedUrl(item.modelData.path) : ""
            }

            MaterialIcon {
                anchors.centerIn: parent
                visible: !item.modelData.isImage
                fontStyle: Tokens.font.icon.builders.extraLarge.scale(1.8).build()
                color: item.modelData.isDir ? Colours.palette.m3primary : Colours.palette.m3onSurfaceVariant
                fill: item.modelData.isDir ? 1 : 0

                text: {
                    if (!item.modelData.isDir)
                        return "description";
                    const n = item.modelData.name.toLowerCase();
                    if (["downloads", "indirilenler", "i̇ndirilenler"].includes(n))
                        return "file_download";
                    if (["desktop", "masaüstü", "masaustu"].includes(n))
                        return "desktop_windows";
                    if (["documents", "belgeler"].includes(n))
                        return "description";
                    if (["pictures", "resimler", "photos", "fotoğraflar"].includes(n))
                        return "image";
                    if (["music", "müzik", "muzik"].includes(n))
                        return "music_note";
                    if (["videos", "videolar"].includes(n))
                        return "video_library";
                    return "folder";
                }
            }
        }

        StyledText {
            id: name

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: iconArea.bottom
            anchors.topMargin: Tokens.spacing.small
            anchors.margins: Tokens.padding.medium

            horizontalAlignment: Text.AlignHCenter
            elide: item.GridView.isCurrentItem ? Text.ElideNone : Text.ElideRight
            wrapMode: item.GridView.isCurrentItem ? Text.WrapAtWordBoundaryOrAnywhere : Text.NoWrap

            Component.onCompleted: text = item.modelData.name
        }

        Behavior on implicitHeight {
            Anim {}
        }
    }
}
