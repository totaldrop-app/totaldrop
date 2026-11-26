// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import QtQuick

// Stand-in for the Rust Transfer QObject (linux/src/bridge/transfer.rs),
// exposing the same properties, signals and methods so SendPage and
// ReceivePage can be driven through every stage in QML tests.
QtObject {
    property string state: "idle"
    property string role: ""
    property string code: ""
    property string link: ""
    property string qr: ""
    property int qrSize: 0
    property var words: []
    property real done: 0
    property real total: 0
    property int items: 0
    property real size: 0
    property int offerFiles: 0
    property var offerNames: []
    property int filesOk: 0
    property int filesFailed: 0
    property string savedTo: ""
    property string savedToUrl: ""
    property string errorKind: ""
    property string errorDetail: ""
    property string errorCause: ""
    property string server: "wss://rendezvous.example"
    property string defaultServer: "wss://rendezvous.example"
    property bool privacyMode: false
    property string destination: "/tmp"
    property bool isText: false
    property string receivedText: ""
    property string shownText: ""
    property bool shownShort: false
    property bool shownHidden: false
    property int pickFilesCalls: 0

    signal picked(var paths)
    signal codeFound(string code)
    signal codeNotFound()
    signal measured(string path, real bytes)

    function send(paths, strong) { role = "send"; isText = false; state = "preparing" }
    function sendText(text, strong) { role = "send"; isText = true; state = "preparing" }
    function textBytes(text) { return text.length }
    function maxText() { return 1048576 }
    function shownChars() { return 16384 }
    function clipboardText() { return "" }
    function copyText(text) {}
    function receive(code) { role = "receive"; state = "connecting" }
    function accept() { state = "transferring" }
    function decline() { state = "declined" }
    function cancel() { state = "idle"; role = "" }
    function reset() {
        state = "idle"
        role = ""
        errorKind = ""
        errorDetail = ""
        errorCause = ""
        isText = false
        filesOk = 0
        filesFailed = 0
        shownText = ""
        shownShort = false
        shownHidden = false
    }
    function normalizeCode(input) { return input.trim() }
    function completions(prefix, limit) { return [] }
    function codeFromLink(text) { return "" }
    function describe(path) { return path.split("/").pop() + "\t10\t0" }
    function measure(path) {}
    function localPaths(urls) { return [] }
    function pickFiles(title) { pickFilesCalls += 1 }
    function readQrImage(title) {}
    function saveServer(url) { server = url; return true }
    function savePrivacyMode(on) { privacyMode = on; return true }
    function resetServer() { server = defaultServer }
    function notify(title, body) {}
}
