import QtQuick
import QtQuick.Controls

Item {
    id: row

    required property int page
    required property int number
    required property bool divided
    required property string title
    readonly property int kind: (page + number) % 4

    implicitHeight: label.implicitHeight + 28

    Rectangle {
        visible: row.divided
        width: row.width
        height: 1
        color: "#e2e8f0"
    }

    Column {
        id: label

        x: 18
        anchors.verticalCenter: parent.verticalCenter
        spacing: 3

        Text {
            text: row.title
            font.pixelSize: 14
            color: "#0f172a"
        }

        Text {
            text: "Describes what option " + (row.number + 1) + " changes"
            font.pixelSize: 12
            color: "#64748b"
        }
    }

    Loader {
        anchors.right: parent.right
        anchors.rightMargin: 18
        anchors.verticalCenter: parent.verticalCenter
        sourceComponent: [toggle, slider, value, chevron][row.kind]
    }

    Component {
        id: toggle

        Switch {
            checked: (row.page + row.number) % 3 === 0
        }
    }

    Component {
        id: slider

        Slider {
            width: 160
            value: ((row.page * 7 + row.number * 13) % 100) / 100
        }
    }

    Component {
        id: value

        Row {
            spacing: 4

            Text {
                text: "Automatic"
                font.pixelSize: 14
                color: "#64748b"
                anchors.verticalCenter: parent.verticalCenter
            }

            Image {
                source: "icons/chevron-right.svg"
                sourceSize: Qt.size(16, 16)
                anchors.verticalCenter: parent.verticalCenter
            }
        }
    }

    Component {
        id: chevron

        Image {
            source: "icons/chevron-right.svg"
            sourceSize: Qt.size(16, 16)
        }
    }
}
