pragma Singleton
import QtQuick

Item {
    id: root

    property string city: "Quito"
    property real currentTemp: 0
    property string currentCondition: "Unknown"
    property string currentIcon: "❓"
    property var forecast: []

    readonly property string _url: "https://api.open-meteo.com/v1/forecast?latitude=-0.2299&longitude=-78.5249&current=temperature_2m,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto"

    readonly property var _dayNames: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]

    function weatherCodeToText(code) {
        switch (code) {
            case 0: return "Sunny";
            case 1: return "Mostly Clear";
            case 2: return "Partly Cloudy";
            case 3: return "Cloudy";
            case 45:
            case 48: return "Fog";
            case 51:
            case 53:
            case 55: return "Drizzle";
            case 61:
            case 65: return "Rain";
            case 63: return "Heavy Rain";
            case 71:
            case 73:
            case 75: return "Snow";
            case 80:
            case 81:
            case 82: return "Showers";
            case 95:
            case 96:
            case 99: return "Thunderstorm";
            default: return "Unknown";
        }
    }

    function weatherCodeToIcon(code) {
        switch (code) {
            case 0: return "☀";
            case 1: return "☀";
            case 2: return "⛅";
            case 3: return "☁";
            case 45:
            case 48: return "🌫";
            case 51:
            case 53:
            case 55: return "🌧";
            case 61:
            case 65: return "🌧";
            case 63: return "🌧";
            case 71:
            case 73:
            case 75: return "❄";
            case 80:
            case 81:
            case 82: return "🌧";
            case 95:
            case 96:
            case 99: return "⚡";
            default: return "❓";
        }
    }

    function _fetch() {
        const xhr = new XMLHttpRequest();
        xhr.onreadystatechange = function () {
            if (xhr.readyState === XMLHttpRequest.DONE) {
                if (xhr.status === 200) {
                    try {
                        root._applyData(JSON.parse(xhr.responseText));
                    } catch (e) {
                        console.log("WeatherService: failed to parse response -", e);
                    }
                } else {
                    console.log("WeatherService: request failed, status", xhr.status);
                }
            }
        };
        xhr.open("GET", root._url);
        xhr.send();
    }

    function _applyData(data) {
        if (data.current) {
            root.currentTemp = data.current.temperature_2m;
            const code = data.current.weather_code;
            root.currentCondition = root.weatherCodeToText(code);
            root.currentIcon = root.weatherCodeToIcon(code);
        }

        if (data.daily && data.daily.time) {
            const times = data.daily.time;
            const codes = data.daily.weather_code;
            const highs = data.daily.temperature_2m_max;
            const lows = data.daily.temperature_2m_min;

            let days = [];
            // Skip index 0 (today, already shown in the current-weather block);
            // take the next 5 days for the forecast row.
            for (let i = 1; i < times.length && days.length < 5; i++) {
                const date = new Date(times[i] + "T00:00:00");
                const code = codes[i];
                const avgTemp = Math.round((highs[i] + lows[i]) / 2);

                days.push({
                    day: root._dayNames[date.getUTCDay()],
                    temp: avgTemp,
                    condition: root.weatherCodeToText(code),
                    icon: root.weatherCodeToIcon(code)
                });
            }
            root.forecast = days;
        }
    }

    Timer {
        interval: 900000   // 15 minutes
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: root._fetch()
    }
}
