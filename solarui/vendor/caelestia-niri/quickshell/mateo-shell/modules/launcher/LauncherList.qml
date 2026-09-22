import QtQuick
import "../../services"

ListView {
    id: root

    clip: true
    spacing: 4
    model: AppLauncherService.filteredApps

    delegate: AppEntry {
        required property var modelData
        width: root.width
        name: modelData.name
        comment: modelData.comment
        icon: modelData.icon
        onActivated: AppLauncherService.launch(modelData.exec)
    }
}
