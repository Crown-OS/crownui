import QtQuick

Window {
    id: window

    required property var config
    readonly property var bench: ({
        width: Number(config.width),
        height: Number(config.height),
        rows: Number(config.rows),
        rowHeight: Number(config.row_height),
        swatch: Number(config.swatch),
        scrollSpeed: Number(config.scroll_speed),
        boxes: Number(config.boxes),
        boxSize: Number(config.box_size),
        labels: Number(config.labels),
        pages: Number(config.pages),
        sections: Number(config.sections),
        sectionRows: Number(config.section_rows),
        sidebarWidth: Number(config.sidebar_width),
        pageScrollSpeed: Number(config.page_scroll_speed),
        pageSeconds: Number(config.page_seconds)
    })
    readonly property var pages: ({
        hello: "Hello.qml",
        list: "List.qml",
        animation: "Animation.qml",
        settings: "Settings.qml"
    })

    width: bench.width
    height: bench.height
    visible: true
    title: "Qt bench"
    color: "#f8fafc"

    Loader {
        id: page
        anchors.fill: parent
    }

    Component.onCompleted: page.setSource(window.pages[window.config.scenario], { bench: window.bench })
}
