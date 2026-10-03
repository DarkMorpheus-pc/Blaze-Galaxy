pragma Singleton

import QtQuick
import Quickshell
import Quickshell.Io
import Caelestia
import Caelestia.Config

Singleton {
    id: root

    readonly property string home: Quickshell.env("HOME")
    readonly property string pictures: getUserDir("pictures")
    readonly property string videos: getUserDir("videos")

    readonly property string data: `${Quickshell.env("XDG_DATA_HOME") || `${home}/.local/share`}/caelestia`
    readonly property string state: `${Quickshell.env("XDG_STATE_HOME") || `${home}/.local/state`}/caelestia`
    readonly property string cache: `${Quickshell.env("XDG_CACHE_HOME") || `${home}/.cache`}/caelestia`
    readonly property string config: `${Quickshell.env("XDG_CONFIG_HOME") || `${home}/.config`}/caelestia`

    readonly property string imagecache: `${cache}/imagecache`
    readonly property string notifimagecache: `${imagecache}/notifs`
    readonly property string wallsdir: Quickshell.env("CAELESTIA_WALLPAPERS_DIR") || absolutePath(GlobalConfig.paths.wallpaperDir)
    readonly property string recsdir: Quickshell.env("CAELESTIA_RECORDINGS_DIR") || `${videos}/Recordings`
    readonly property string libdir: Quickshell.env("CAELESTIA_LIB_DIR") || "/usr/lib/caelestia"

    property var xdgDirs: ({})

    FileView {
        path: `${root.home}/.config/user-dirs.dirs`
        onLoaded: {
            const dirs = {};
            const lines = text().split("\n");
            for (const line of lines) {
                const trimmed = line.trim();
                if (!trimmed || trimmed.startsWith("#"))
                    continue;
                const m = trimmed.match(/^XDG_([A-Z]+)_DIR="?([^"\n]+)"?/);
                if (m) {
                    const key = m[1].toLowerCase();
                    const path = m[2].replace("$HOME", root.home);
                    dirs[key] = path;
                }
            }
            root.xdgDirs = dirs;
        }
    }

    function getUserDir(name: string): string {
        const lower = name.toLowerCase();
        if (xdgDirs[lower]) return xdgDirs[lower];
        if (lower === "downloads" && xdgDirs["download"]) return xdgDirs["download"];
        if (lower === "pictures" && xdgDirs["pictures"]) return xdgDirs["pictures"];
        if (lower === "desktop" && xdgDirs["desktop"]) return xdgDirs["desktop"];
        if (lower === "documents" && xdgDirs["documents"]) return xdgDirs["documents"];
        if (lower === "music" && xdgDirs["music"]) return xdgDirs["music"];
        if (lower === "videos" && xdgDirs["videos"]) return xdgDirs["videos"];

        const turkishMap = {
            "downloads": "İndirilenler",
            "desktop": "Masaüstü",
            "documents": "Belgeler",
            "pictures": "Resimler",
            "music": "Müzik",
            "videos": "Videolar"
        };
        if (turkishMap[lower]) {
            return `${root.home}/${turkishMap[lower]}`;
        }
        return `${root.home}/${name}`;
    }

    function getUserDirName(name: string): string {
        const full = getUserDir(name);
        if (full.startsWith(root.home + "/")) {
            return full.slice(root.home.length + 1);
        }
        return name;
    }

    function toLocalFile(path: url): string {
        path = Qt.resolvedUrl(path);
        return path.toString() ? CUtils.toLocalFile(path) : "";
    }

    function absolutePath(path: string): string {
        return toLocalFile(path.replace(/~|(\$({?)HOME(}?))+/, home));
    }

    function shortenHome(path: string): string {
        return path.replace(home, "~");
    }
}
