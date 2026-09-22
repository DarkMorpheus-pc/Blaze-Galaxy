import QtQuick
import "../../config"
import "../../services"

WeatherCard {
    anchors.fill: parent

    CurrentWeather {
        width: parent.width
        city: WeatherService.city
        temperature: Math.round(WeatherService.currentTemp) + "°C"
        condition: WeatherService.currentCondition
        icon: WeatherService.currentIcon
    }

    Item {
        width: 1
        height: 4
    }

    ForecastRow {
        width: parent.width
        days: WeatherService.forecast
    }
}
