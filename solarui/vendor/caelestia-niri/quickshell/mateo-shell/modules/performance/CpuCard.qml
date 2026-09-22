import QtQuick
import "../../config"
import "../../services"

PerformanceCard {
    title: "CPU"
    subtitle: SystemService.cpuModel
    percentage: SystemService.cpuUsage
}
