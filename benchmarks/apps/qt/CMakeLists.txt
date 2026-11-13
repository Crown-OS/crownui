cmake_minimum_required(VERSION 3.21)
project(qt-bench LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Quick)
qt_standard_project_setup(REQUIRES 6.5)

qt_add_executable(qt-bench main.cpp)

# The settings icons live with the web apps; the QML reaches them as icons/<name>.svg.
file(GLOB icon_files "${CMAKE_CURRENT_SOURCE_DIR}/../web/icons/*.svg")
foreach(icon_file IN LISTS icon_files)
    get_filename_component(icon_name "${icon_file}" NAME)
    set_source_files_properties("${icon_file}" PROPERTIES QT_RESOURCE_ALIAS "icons/${icon_name}")
endforeach()

qt_add_qml_module(qt-bench
    URI Bench
    VERSION 1.0
    QML_FILES Main.qml Hello.qml List.qml Animation.qml Settings.qml SettingRow.qml palette.js
    RESOURCES ${icon_files}
)
target_link_libraries(qt-bench PRIVATE Qt6::Quick)
