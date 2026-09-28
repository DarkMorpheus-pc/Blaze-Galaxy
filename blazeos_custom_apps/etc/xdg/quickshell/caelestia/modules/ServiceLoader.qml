import QtQuick
import Quickshell
import Caelestia.Config
import Caelestia.I18n
import qs.services

Scope {
    Component.onCompleted: {
        // Force certain singletons to load on shell init instead of lazily

        Tr;
        IdleInhibitor;
        GameMode;
        Notifs;
        Players;
        Brightness;
        NiriService;
        Weather.reload();

        if (GlobalConfig.utilities.vpn.enabled)
            VPN;
    }
}
