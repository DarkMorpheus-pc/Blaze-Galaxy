pragma Singleton
import QtQuick
import Quickshell
import Quickshell.Io

Item {
    id: root

    property var apps: []
    property string searchText: ""

    readonly property var filteredApps: _filter(apps, searchText)

    function _filter(list, query) {
        if (!query || query.length === 0) return list;
        const q = query.toLowerCase();
        return list.filter(a => a.name.toLowerCase().indexOf(q) !== -1);
    }

    function launch(exec) {
        const cleaned = _cleanExec(exec);
        if (cleaned.length === 0) return;
        Quickshell.execDetached(["sh", "-c", cleaned]);
    }

    function launchFirstMatch() {
        if (filteredApps.length > 0) launch(filteredApps[0].exec);
    }

    function _cleanExec(exec) {
        // Strip desktop-entry field codes (%f, %F, %u, %U, %i, %c, %k, ...)
        return exec.replace(/%[a-zA-Z]/g, "").trim();
    }

    readonly property string _marker: "===MATEO_APP==="

    // One-shot scan at startup. Enumerates + reads .desktop files via a
    // single sh -c call (same Process pattern already used for whoami/df/
    // playerctl elsewhere in this repo) rather than any dedicated app-list
    // binary (no rofi/fuzzel/walker involved).
    Process {
        id: scanProc
        command: [
            "sh", "-c",
            "for f in /usr/share/applications/*.desktop \"$HOME/.local/share/applications\"/*.desktop; do " +
            "[ -f \"$f\" ] || continue; echo '===MATEO_APP==='; cat \"$f\" 2>/dev/null; done"
        ]
        running: true

        stdout: StdioCollector {
            onStreamFinished: root._parse(text)
        }
    }

    function _parse(content) {
        const blocks = content.split(root._marker).slice(1);
        let parsed = [];

        for (const block of blocks) {
            const nameMatch = block.match(/^Name=(.+)$/m);
            const execMatch = block.match(/^Exec=(.+)$/m);
            const iconMatch = block.match(/^Icon=(.+)$/m);
            const commentMatch = block.match(/^Comment=(.+)$/m);
            const typeMatch = block.match(/^Type=(.+)$/m);

            if (block.match(/^Hidden=true/m)) continue;
            if (block.match(/^NoDisplay=true/m)) continue;
            if (typeMatch && typeMatch[1].trim() !== "Application") continue;
            if (!nameMatch || !execMatch) continue;

            parsed.push({
                name: nameMatch[1].trim(),
                exec: execMatch[1].trim(),
                icon: iconMatch ? iconMatch[1].trim() : "",
                comment: commentMatch ? commentMatch[1].trim() : ""
            });
        }

        // De-duplicate by name — system and local dirs can both define the same app
        let seen = {};
        let deduped = [];
        for (const app of parsed) {
            if (seen[app.name]) continue;
            seen[app.name] = true;
            deduped.push(app);
        }
        deduped.sort((a, b) => a.name.localeCompare(b.name));

        root.apps = deduped;
    }
}
