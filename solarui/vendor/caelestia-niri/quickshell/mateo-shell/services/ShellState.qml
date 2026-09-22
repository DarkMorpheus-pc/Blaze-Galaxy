pragma Singleton
import QtQuick

Item {
    id: root

    property bool dashboardOpen: false
    property string activeTab: "Dashboard"

    property bool launcherOpen: false

    property bool sessionOpen: false
    property bool powerMenuOpen: false
    property bool osdVisible: false

    function toggleDashboard() { dashboardOpen = !dashboardOpen }
    function openDashboard() { dashboardOpen = true }
    function closeDashboard() { dashboardOpen = false }
    function setTab(name) { activeTab = name }

    function toggleLauncher() { launcherOpen = !launcherOpen }
    function openLauncher() { launcherOpen = true }
    function closeLauncher() { launcherOpen = false }

    function toggleSession() { sessionOpen = !sessionOpen }
    function openSession() { sessionOpen = true }
    function closeSession() { sessionOpen = false }

    function togglePowerMenu() { powerMenuOpen = !powerMenuOpen }
    function openPowerMenu() { powerMenuOpen = true }
    function closePowerMenu() { powerMenuOpen = false }
}
