//! Builds the QML module and the Rust QObjects with CXX-Qt.

use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

fn main() {
    let qml = [
        QmlFile::from("qml/Main.qml"),
        QmlFile::from("qml/Theme.qml").singleton(true),
        QmlFile::from("qml/Strings.qml").singleton(true),
        QmlFile::from("qml/Mark.qml"),
        QmlFile::from("qml/HomePage.qml"),
        QmlFile::from("qml/ActionCard.qml"),
        QmlFile::from("qml/PageHeader.qml"),
        QmlFile::from("qml/PrimaryButton.qml"),
        QmlFile::from("qml/VerificationWords.qml"),
        QmlFile::from("qml/QrCode.qml"),
        QmlFile::from("qml/TransferProgress.qml"),
        QmlFile::from("qml/ResultPanel.qml"),
        QmlFile::from("qml/SendPage.qml"),
        QmlFile::from("qml/ReceivePage.qml"),
        QmlFile::from("qml/SettingsPage.qml"),
    ];
    CxxQtBuilder::new_qml_module(QmlModule::new("app.totaldrop").qml_files(qml))
        .qt_module("Gui")
        .qt_module("Quick")
        .qt_module("QuickControls2")
        .files(["src/bridge/theme.rs", "src/bridge/transfer.rs"])
        .build();
}
