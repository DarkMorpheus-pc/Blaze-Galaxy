pragma Singleton
import qs
import qs.modules.common
import qs.modules.common.functions
import QtQuick
import Quickshell
import Quickshell.Io

/*
 * Distribution-aware system updates service.
 */
Singleton {
    id: root

    property bool available: false
    property alias checking: checkUpdatesProc.running
    property int count: 0
    
    readonly property bool updateAdvised: available && count > Config.options.updates.adviseUpdateThreshold
    readonly property bool updateStronglyAdvised: available && count > Config.options.updates.stronglyAdviseUpdateThreshold

    function load() {}
    function refresh() {
        if (!available) return;
        print("[Updates] Checking for system updates")
        checkUpdatesProc.running = true;
    }

    Timer {
        interval: Config.options.updates.checkInterval * 60 * 1000
        repeat: true
        running: Config.ready && Config.options.updates.enableCheck
        onTriggered: {
            print("[Updates] Periodic update check due")
            root.refresh();
        }
    }

    Process {
        id: checkAvailabilityProc
        running: Config.ready && Config.options.updates.enableCheck
        command: ["bash", "-c", "command -v dnf5 || command -v dnf || command -v checkupdates"]
        onExited: (exitCode, exitStatus) => {
            root.available = (exitCode === 0);
            firstCheckTimer.start();
        }
    }

    Timer {
        id: firstCheckTimer
        interval: 60 * 1000
        onTriggered: root.refresh()
    }

    Process {
        id: checkUpdatesProc
        command: ["bash", "-c", `
            if command -v dnf5 >/dev/null 2>&1; then
                dnf5 -q repoquery --upgrades --qf '%{name}' 2>/dev/null | sort -u | wc -l
            elif command -v dnf >/dev/null 2>&1; then
                dnf -q repoquery --upgrades --qf '%{name}' 2>/dev/null | sort -u | wc -l
            elif command -v checkupdates >/dev/null 2>&1; then
                pacman_count=$(checkupdates 2>/dev/null | wc -l)
                aur_count=0
                if command -v yay >/dev/null 2>&1; then aur_count=$(yay -Qua 2>/dev/null | wc -l)
                elif command -v paru >/dev/null 2>&1; then aur_count=$(paru -Qua 2>/dev/null | wc -l); fi
                echo $((pacman_count + aur_count))
            else
                echo 0
            fi
        `]
        stdout: StdioCollector {
            onStreamFinished: {
                root.count = parseInt(text.trim())
            }
        }
    }
}
