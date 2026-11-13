import QtQuick
import QtQuick.Controls

// The settings scenario, by the rules in apps/shared/settings.rs.
Rectangle {
    id: window

    required property var bench
    readonly property var pages: [
        ["Wi-Fi", "wifi"], ["Bluetooth", "bluetooth"], ["Network", "globe"],
        ["Display", "monitor"], ["Sound", "volume-2"], ["Notifications", "bell"],
        ["Battery & Power", "battery-full"], ["Appearance", "palette"]
    ]
    property int page: -1

    function pageTitle(page) { return pages[page % pages.length][0] }
    function pageIcon(page) { return "icons/" + pages[page % pages.length][1] + ".svg" }

    color: "#eef2f7"

    Column {
        id: sidebar

        x: 14
        y: 14
        width: window.bench.sidebarWidth - 16
        spacing: 2

        Text {
            leftPadding: 8
            topPadding: 6
            bottomPadding: 6
            text: "Settings"
            font.pixelSize: 11
            color: "#64748b"
        }

        Repeater {
            model: window.bench.pages

            Rectangle {
                required property int index

                width: sidebar.width
                height: 32
                radius: 8
                color: index === window.page ? "#dde4ee" : "transparent"

                Row {
                    x: 8
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 8

                    Image {
                        source: window.pageIcon(index)
                        sourceSize: Qt.size(16, 16)
                        anchors.verticalCenter: parent.verticalCenter
                    }

                    Text {
                        text: window.pageTitle(index)
                        font.pixelSize: 13
                        color: "#0f172a"
                        anchors.verticalCenter: parent.verticalCenter
                    }
                }
            }
        }
    }

    Rectangle {
        x: window.bench.sidebarWidth + 6
        y: 6
        width: window.width - x - 6
        height: window.height - 12
        radius: 12
        color: "#f8fafc"
        clip: true

        Loader {
            id: content

            anchors.fill: parent
        }
    }

    Component {
        id: pageComponent

        Flickable {
            id: flickable

            readonly property int page: window.page

            contentHeight: column.implicitHeight + 64
            boundsBehavior: Flickable.StopAtBounds

            Column {
                id: column

                x: 32
                y: 32
                width: flickable.width - 64
                spacing: 24

                Text {
                    text: window.pageTitle(flickable.page)
                    font.pixelSize: 22
                    font.weight: Font.DemiBold
                    color: "#0f172a"
                }

                Repeater {
                    model: window.bench.sections

                    Column {
                        id: group

                        required property int index

                        width: column.width
                        spacing: 8

                        Text {
                            leftPadding: 4
                            text: "SECTION " + (group.index + 1)
                            font.pixelSize: 12
                            font.weight: Font.Medium
                            color: "#64748b"
                        }

                        Rectangle {
                            width: group.width
                            height: rows.implicitHeight + 2
                            radius: 12
                            color: "#ffffff"
                            border.color: "#e2e8f0"

                            Column {
                                id: rows

                                x: 1
                                y: 1
                                width: parent.width - 2

                                Repeater {
                                    model: window.bench.sectionRows

                                    SettingRow {
                                        required property int index

                                        width: rows.width
                                        page: flickable.page
                                        number: group.index * window.bench.sectionRows + index
                                        divided: index > 0
                                        title: window.pageTitle(flickable.page) + " option " + (number + 1)
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    FrameAnimation {
        running: true
        onTriggered: {
            const opened = Math.floor(elapsedTime / window.bench.pageSeconds);
            const page = opened % window.bench.pages;
            if (page !== window.page) {
                window.page = page;
                content.sourceComponent = undefined;
                content.sourceComponent = pageComponent;
            }
            const flickable = content.item;
            const travel = Math.max(1, flickable.contentHeight - flickable.height);
            flickable.contentY = ((elapsedTime - opened * window.bench.pageSeconds) * window.bench.pageScrollSpeed) % travel;
        }
    }
}
