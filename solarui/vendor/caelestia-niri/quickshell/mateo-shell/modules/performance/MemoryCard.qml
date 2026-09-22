import QtQuick
import "../../config"
import "../../services"

PerformanceCard {
    title: "Memory"
    subtitle: Math.round(SystemService.ramUsed) + " MB / " + Math.round(SystemService.ramTotal) + " MB"
    percentage: SystemService.ramUsage
}
