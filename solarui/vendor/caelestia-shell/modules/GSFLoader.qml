import QtQuick
import Quickshell

Item {
    FontLoader {
        source: Quickshell.shellPath("assets/google-sans-flex/GoogleSansFlex-VariableFont_GRAD,ROND,opsz,slnt,wdth,wght.ttf")
    }

    FontLoader {
        source: Quickshell.shellPath("assets/material-symbols/MaterialSymbolsRounded[FILL,GRAD,opsz,wght].ttf")
    }
}
