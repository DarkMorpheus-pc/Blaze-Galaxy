import Quickshell
import "./modules/sidebar"
import "./modules/dashboard"
import "./modules/launcher"
import "./modules/session"
import "./modules/power"
import "./modules/osd"

ShellRoot {
    Sidebar {}

    DashboardWindow {}

    LauncherWindow {}

    SessionWindow {}

    PowerWindow {}

    OsdWindow {}
}
