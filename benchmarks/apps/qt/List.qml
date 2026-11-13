import QtQuick
import "palette.js" as Palette

ListView {
    id: list

    required property var bench
    readonly property real travel: Math.max(1, bench.rows * bench.rowHeight - height)

    clip: true
    model: bench.rows
    delegate: Item {
        required property int index

        width: ListView.view.width
        height: list.bench.rowHeight

        Rectangle {
            id: swatch
            x: 16
            anchors.verticalCenter: parent.verticalCenter
            width: list.bench.swatch
            height: list.bench.swatch
            radius: 6
            color: Palette.swatch(index)
        }

        Column {
            anchors.left: swatch.right
            anchors.leftMargin: 12
            anchors.verticalCenter: parent.verticalCenter
            spacing: 2

            Text {
                text: "Row " + index
                font.pixelSize: 15
                font.weight: Font.Medium
                color: "#0f172a"
            }

            Text {
                text: "Subtitle for row " + index
                font.pixelSize: 12
                color: "#64748b"
            }
        }
    }

    FrameAnimation {
        running: true
        onTriggered: list.contentY = (elapsedTime * list.bench.scrollSpeed) % list.travel
    }
}
