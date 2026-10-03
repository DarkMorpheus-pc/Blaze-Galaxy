pragma ComponentBehavior: Bound

import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import qs.components.containers
import qs.components.misc
import qs.services

Scope {
    LazyLoader {
        id: root

        property bool freeze
        property bool closing
        property bool clipboardOnly

        Variants {
            model: Screens.screens

            StyledWindow {
                id: win

                required property ShellScreen modelData

                screen: modelData
                name: "area-picker"
                WlrLayershell.exclusionMode: ExclusionMode.Ignore
                WlrLayershell.layer: WlrLayer.Overlay
                WlrLayershell.keyboardFocus: root.closing ? WlrKeyboardFocus.None : WlrKeyboardFocus.Exclusive
                mask: root.closing ? empty : null

                anchors.top: true
                anchors.bottom: true
                anchors.left: true
                anchors.right: true

                Region {
                    id: empty
                }

                Picker {
                    loader: root
                    screen: win.modelData
                }
            }
        }
    }

    IpcHandler {
        function open(): void {
            root.freeze = false;
            root.closing = false;
            root.clipboardOnly = false;
            root.activeAsync = true;
        }

        function openFreeze(): void {
            root.freeze = true;
            root.closing = false;
            root.clipboardOnly = false;
            root.activeAsync = true;
        }

        function openClip(): void {
            root.freeze = false;
            root.closing = false;
            root.clipboardOnly = true;
            root.activeAsync = true;
        }

        function openFreezeClip(): void {
            root.freeze = true;
            root.closing = false;
            root.clipboardOnly = true;
            root.activeAsync = true;
        }

        target: "picker"
    }

    // Quickshell shortcuts disabled to allow Niri to handle screenshots natively via grim/satty
    // CustomShortcut { ... }
}
