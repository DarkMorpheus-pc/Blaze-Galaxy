import QtQuick
import "../../config"
import "../../services"

PerformanceCard {
    title: "Disk"
    subtitle: SystemService.diskUsed.toFixed(1) + " GB / " + SystemService.diskTotal.toFixed(1) + " GB"
    percentage: SystemService.diskUsage
}
