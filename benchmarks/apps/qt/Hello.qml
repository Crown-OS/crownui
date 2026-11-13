import QtQuick
import QtQuick.Controls

Item {
    required property var bench

    Column {
        anchors.centerIn: parent
        spacing: 16

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Hello, world"
            font.pixelSize: 32
            font.weight: Font.DemiBold
            color: "#0f172a"
        }

        Button {
            anchors.horizontalCenter: parent.horizontalCenter
            text: "Click me"
        }
    }
}
