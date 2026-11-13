import QtQuick
import "palette.js" as Palette

Item {
    id: stage

    required property var bench
    property real seconds: 0
    readonly property real cell: bench.boxSize * 1.5
    readonly property int columns: Math.max(1, Math.floor(bench.width / cell))
    readonly property real inset: (cell - bench.boxSize) / 2
    readonly property real labelWidth: bench.width / 20
    readonly property real labelTop: Math.ceil(bench.boxes / columns) * cell + 8

    Repeater {
        model: stage.bench.boxes

        Rectangle {
            required property int index
            readonly property real phase: stage.seconds * 2 + index * 0.15

            x: (index % stage.columns) * stage.cell + stage.inset + 6 * Math.sin(phase)
            y: Math.floor(index / stage.columns) * stage.cell + stage.inset + 6 * Math.cos(phase)
            width: stage.bench.boxSize
            height: stage.bench.boxSize
            radius: 3
            rotation: (stage.seconds * 90 + index * 3) % 360
            color: Palette.swatch(index)
        }
    }

    Repeater {
        model: stage.bench.labels

        Text {
            required property int index

            x: (index % 20) * stage.labelWidth
            y: stage.labelTop + Math.floor(index / 20) * 18
            font.pixelSize: 11
            color: "#0f172a"
            text: String(Math.floor(stage.seconds * 1000) + index)
        }
    }

    FrameAnimation {
        running: true
        onTriggered: stage.seconds = elapsedTime
    }
}
