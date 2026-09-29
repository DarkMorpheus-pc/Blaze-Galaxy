pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import Caelestia.Config
import qs.modules.bar as Bar

Region {
    id: root

    required property Bar.BarWrapper bar
    required property Panels panels
    required property var win

    readonly property real borderThickness: win.contentItem.Config.border.thickness
    readonly property real edgeTrigger: 3

    // 1. Sol Cubuk (Caelestia Bar): Yalnizca sol cubuk alani tiklama alir
    x: 0
    y: 0
    width: root.bar.clampedWidth
    height: root.win.height

    // 2. Acik Popout (WiFi, Ses, Bluetooth, Pil penceresi acildiginda)
    Region {
        x: root.panels.popoutsWrapper.x + root.bar.implicitWidth
        y: root.panels.popoutsWrapper.y
        width: root.panels.popouts.hasCurrent ? root.panels.popoutsWrapper.width : 0
        height: root.panels.popouts.hasCurrent ? root.panels.popoutsWrapper.height : 0
    }

    // 3. Kontrol Merkezi (Dashboard / Ust Cekmece + Kenar Algilayici)
    Region {
        x: root.panels.dashboard.x + root.bar.implicitWidth
        y: 0
        width: root.panels.dashboard.width
        height: root.panels.dashboard.offsetScale < 1 ? (root.panels.dashboard.height * (1 - root.panels.dashboard.offsetScale) + root.borderThickness) : root.edgeTrigger
    }

    // 4. Uygulama Baslatici (Launcher / Alt Cekmece + Kenar Algilayici)
    Region {
        x: root.panels.launcher.x + root.bar.implicitWidth
        y: root.win.height - height
        width: root.panels.launcher.width
        height: root.panels.launcher.offsetScale < 1 ? (root.panels.launcher.height * (1 - root.panels.launcher.offsetScale) + root.borderThickness) : root.edgeTrigger
    }

    // 5. Oturum Cekmecesi (Session / Guc Menusu + Kenar Algilayici)
    Region {
        x: root.win.width - width
        y: root.panels.sessionWrapper.y + root.borderThickness
        width: root.panels.session.offsetScale < 1 ? (root.panels.session.width * (1 - root.panels.session.offsetScale) + root.borderThickness) : root.edgeTrigger
        height: root.panels.session.height
    }

    // 6. Yan Cekmece (Sidebar / Bildirim & Denetim + Sag Kenar Algilayici)
    Region {
        x: root.win.width - width
        y: root.panels.sidebar.y + root.borderThickness
        width: root.panels.sidebar.offsetScale < 1 ? (root.panels.sidebar.width * (1 - root.panels.sidebar.offsetScale) + root.borderThickness) : root.edgeTrigger
        height: root.panels.sidebar.height
    }

    // 7. Hizli Araclar (Utilities / Sag Alt Cekmece)
    Region {
        x: root.win.width - width
        y: root.win.height - height
        width: root.panels.utilities.offsetScale < 1 ? (root.panels.utilities.width * (1 - root.panels.utilities.offsetScale) + root.borderThickness) : root.edgeTrigger
        height: root.panels.utilities.offsetScale < 1 ? (root.panels.utilities.height * (1 - root.panels.utilities.offsetScale) + root.borderThickness) : root.edgeTrigger
    }

    // 8. Bildirimler
    Region {
        x: root.panels.notifications.x + root.bar.implicitWidth
        y: 0
        width: root.panels.notifications.width
        height: root.panels.notifications.height > 0 ? (root.panels.notifications.height + root.borderThickness) : 0
    }

    // 9. Ses / Parlaklik Gostergesi (OSD)
    Region {
        x: root.win.width - width
        y: root.panels.osdWrapper.y + root.borderThickness
        width: root.panels.osd.offsetScale < 1 ? (root.panels.osd.width * (1 - root.panels.osd.offsetScale) + root.borderThickness) : 0
        height: root.panels.osd.height
    }
}
