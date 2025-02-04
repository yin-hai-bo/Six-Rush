import QtQuick

Window {
    width: 640
    height: 480
    visible: true
    title: qsTr("Hello World")
    Text {
        text: "Hello, world"
        anchors.centerIn: parent
        font.pointSize: 24
        color: "black"
    }
}
