pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

Item {
    id: root

    property var workspaces: []
    property int focusedIndex: -1

    // New: window tracking, needed for ContextIndicator + workspace occupancy.
    // Verify "WindowsChanged"/"WindowFocusChanged" field names (id, title,
    // app_id, workspace_id) against your niri version's IPC output.
    property var windows: []
    property var focusedWindow: null

    function focusWorkspace(idx) {
        Quickshell.execDetached(["niri", "msg", "action", "focus-workspace-index", idx.toString()]);
    }

    function _updateFocused() {
        for (let i = 0; i < workspaces.length; i++) {
            if (workspaces[i].is_focused) {
                focusedIndex = i;
                return;
            }
        }
    }

    function _updateFocusedWindow(focusedId) {
        if (focusedId === null || focusedId === undefined) {
            root.focusedWindow = null;
            return;
        }
        for (let i = 0; i < root.windows.length; i++) {
            if (root.windows[i].id === focusedId) {
                root.focusedWindow = root.windows[i];
                return;
            }
        }
        root.focusedWindow = null;
    }

    function isWorkspaceOccupied(workspaceId) {
        for (let i = 0; i < root.windows.length; i++) {
            if (root.windows[i].workspace_id === workspaceId) return true;
        }
        return false;
    }

    function firstWindowAppId(workspaceId) {
        for (let i = 0; i < root.windows.length; i++) {
            if (root.windows[i].workspace_id === workspaceId) {
                return root.windows[i].app_id || "";
            }
        }
        return "";
    }

    Process {
        id: eventStream
        command: ["niri", "msg", "-j", "event-stream"]
        running: true

        stdout: SplitParser {
            splitMarker: "\n"
            onRead: line => {
                if (!line) return;
                let event;
                try { event = JSON.parse(line); } catch (e) { return; }

                if (event.WorkspacesChanged) {
                    root.workspaces = event.WorkspacesChanged.workspaces;
                    root._updateFocused();
                } else if (event.WorkspaceActivated) {
                    root._updateFocused();
                } else if (event.WindowsChanged) {
                    root.windows = event.WindowsChanged.windows;
                } else if (event.WindowOpenedOrChanged) {
                    const w = event.WindowOpenedOrChanged.window;
                    let updated = root.windows.filter(win => win.id !== w.id);
                    updated.push(w);
                    root.windows = updated;
                    if (w.is_focused) root.focusedWindow = w;
                } else if (event.WindowClosed) {
                    root.windows = root.windows.filter(win => win.id !== event.WindowClosed.id);
                } else if (event.WindowFocusChanged) {
                    root._updateFocusedWindow(event.WindowFocusChanged.id);
                }
            }
        }

        onExited: restartTimer.start()
    }

    Timer {
        id: restartTimer
        interval: 1000
        onTriggered: eventStream.running = true
    }
}
