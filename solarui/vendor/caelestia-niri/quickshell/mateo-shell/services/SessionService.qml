pragma Singleton
import QtQuick

Item {
    id: root

    property bool wifiEnabled: true
    property bool bluetoothEnabled: true
    property bool doNotDisturb: false
    property bool airplaneMode: false
    property bool micMuted: false
    property bool keepAwake: false

    function toggleWifi() { wifiEnabled = !wifiEnabled }
    function toggleBluetooth() { bluetoothEnabled = !bluetoothEnabled }
    function toggleDoNotDisturb() { doNotDisturb = !doNotDisturb }
    function toggleAirplaneMode() { airplaneMode = !airplaneMode }
    function toggleMicMute() { micMuted = !micMuted }
    function toggleKeepAwake() { keepAwake = !keepAwake }

    function openSettings() {
        // TODO: wire to real settings app once backend step lands
        console.log("SessionService: openSettings() not yet implemented")
    }
}
