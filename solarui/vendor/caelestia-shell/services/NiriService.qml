pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io

Singleton {
    id: root

    readonly property bool isNiri: Boolean(Quickshell.env("NIRI_SOCKET")) || Quickshell.env("XDG_CURRENT_DESKTOP") === "niri"
    property var workspaces: []
    property int activeWsId: 1
    property var windows: []
    property var occupied: ({})

    function focusWorkspace(idx: int): void {
        if (!isNiri)
            return;
        Quickshell.execDetached(["niri", "msg", "action", "focus-workspace", idx.toString()]);
    }

    function focusWorkspaceUp(): void {
        if (!isNiri)
            return;
        Quickshell.execDetached(["niri", "msg", "action", "focus-workspace-up"]);
    }

    function focusWorkspaceDown(): void {
        if (!isNiri)
            return;
        Quickshell.execDetached(["niri", "msg", "action", "focus-workspace-down"]);
    }

    function _updateState(): void {
        const occ = {};

        // Track occupancy from windows
        for (const win of root.windows) {
            if (win.workspace_id) {
                const wsObj = root.workspaces.find(w => w.id === win.workspace_id);
                const wsNum = wsObj?.idx ?? win.workspace_id;
                occ[wsNum] = true;
            }
        }

        // Also track occupancy and focus from workspaces array
        for (let i = 0; i < root.workspaces.length; i++) {
            const ws = root.workspaces[i];
            const wsNum = ws.idx ?? (i + 1);
            if (ws.active_window_id !== null && ws.active_window_id !== undefined)
                occ[wsNum] = true;
            if (ws.is_focused)
                root.activeWsId = wsNum;
        }

        root.occupied = occ;
    }

    Process {
        id: eventStream

        command: ["niri", "msg", "-j", "event-stream"]
        running: root.isNiri

        stdout: SplitParser {
            splitMarker: "\n"
            onRead: line => {
                if (!line)
                    return;
                let event;
                try {
                    event = JSON.parse(line);
                } catch (e) {
                    return;
                }

                if (event.WorkspacesChanged) {
                    root.workspaces = event.WorkspacesChanged.workspaces;
                    root._updateState();
                } else if (event.WorkspaceActivated) {
                    const wsObj = root.workspaces.find(w => w.id === event.WorkspaceActivated.id);
                    if (wsObj?.idx)
                        root.activeWsId = wsObj.idx;
                    else if (event.WorkspaceActivated.id)
                        root.activeWsId = event.WorkspaceActivated.id;
                    root._updateState();
                } else if (event.WindowsChanged) {
                    root.windows = event.WindowsChanged.windows;
                    root._updateState();
                } else if (event.WindowOpenedOrChanged) {
                    const w = event.WindowOpenedOrChanged.window;
                    const updated = root.windows.filter(win => win.id !== w.id);
                    updated.push(w);
                    root.windows = updated;
                    root._updateState();
                } else if (event.WindowClosed) {
                    root.windows = root.windows.filter(win => win.id !== event.WindowClosed.id);
                    root._updateState();
                } else if (event.WindowFocusChanged) {
                    root._updateState();
                }
            }
        }

        onExited: if (root.isNiri) restartTimer.start()
    }

    Timer {
        id: restartTimer

        interval: 1000
        onTriggered: if (root.isNiri) eventStream.running = true
    }
}
