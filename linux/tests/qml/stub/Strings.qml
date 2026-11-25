// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

pragma Singleton
import QtQuick

// English stand-in for Strings.qml (which reads the Rust Texts singleton)
// so QML component tests can instantiate SendPage, ReceivePage, ResultPanel
// and TransferProgress without a Rust build.
QtObject {
    readonly property bool rtl: false
    readonly property bool draft: false

    readonly property var s: ({
        appTagline: "Any file, to anyone, anywhere.",
        homeSendTitle: "Send",
        homeSendBody: "Pick files and share a short code",
        homeReceiveTitle: "Receive",
        homeReceiveBody: "Enter a code and save the files",
        homeFooter: "End-to-end encrypted. No account, no email, no phone number.",
        settings: "Settings",
        sendTitle: "Send files",
        sendAddFiles: "Add files",
        sendEmpty: "Nothing selected yet",
        sendEmptyHint: "Add the files you want to send. The receiver gets exactly these, nothing else.",
        sendStrongCode: "Stronger code",
        sendStrongCodeHint: "Three words instead of two: much harder to guess",
        sendCreateCode: "Create code",
        sendCounting: "Counting…",
        sendRemove: "Remove",
        sendPreparing: "Preparing your files…",
        sendAddFailed: "The files could not be added. Try again. If it happens again, check that this device has enough free space, or choose the files from another place.",
        sendYourCode: "Your code",
        sendCodeHint: "On the other device, open TotalDrop, tap Receive and enter this code.",
        sendWaiting: "Waiting for the receiver…",
        sendCopy: "Copy",
        sendCopied: "Code copied",
        sendShare: "Share",
        sendSending: "Sending",
        sendDone: "Sent and verified",
        receiveTitle: "Receive files",
        receiveCodeLabel: "Code",
        receiveCodeHint: "7-harbor-rocket",
        receiveCodeHelp: "Ask the sender for their code.",
        receiveCodeInvalid: "That does not look like a code. It starts with a number, then two or three words.",
        receivePaste: "Paste",
        receiveConnect: "Connect",
        receiveConnecting: "Connecting securely…",
        receiveOffer: "Incoming files",
        receiveAccept: "Accept",
        receiveDecline: "Decline",
        receiveDeclined: "You declined the files.",
        receiveReceiving: "Receiving",
        receiveDone: "Received and verified",
        verificationTitle: "Verification words",
        verificationHint: "The other person sees the same four words. If they differ, stop.",
        sameNetworkNote: "Works across the internet. When the devices cannot reach each other directly, the encrypted transfer goes through the relay, which cannot read it.",
        done: "Done",
        tryAgain: "Try again",
        cancel: "Cancel",
        cancelTransferTitle: "Stop this transfer?",
        cancelTransferBody: "The other side will be told the transfer stopped.",
        keepGoing: "Keep going",
        stop: "Stop",
        moreInfo: "Details",
        errorWrongCode: "The code did not match. It was mistyped, or someone tried to intercept the transfer. Nothing was sent. Start again with a new code.",
        errorPeerLeft: "The other side left before the transfer finished.",
        errorDeclined: "The receiver declined the files.",
        errorCancelled: "The other side stopped the transfer.",
        errorStopped: "Stopped on this device.",
        errorInvalidCode: "That code is not valid.",
        errorServer: "Could not reach the TotalDrop server. Check your internet connection.",
        errorCodeNotInUse: "That code is not in use: it was mistyped, has expired or was already used. Ask the sender for a new one.",
        errorCodeExpired: "Nobody used the code before it expired. Send again to get a new code.",
        errorBadServer: "The server address is not valid. It must start with wss://.",
        errorConnect: "Could not connect to the other device. Check that both devices are online and try again.",
        errorInvalidOffer: "The offer broke a safety rule, so it was refused.",
        errorStorage: "A file could not be read or saved.",
        errorOther: "Something went wrong.",
        causeTimedOut: "The connection stopped responding. One of the devices may have lost its network or gone to sleep.",
        causeConnectionLost: "The connection to the other device was cut.",
        causeNoRoute: "No way of reaching the other device worked, directly or through the relay.",
        causeRelay: "The relay server could not be reached or is not set up, or the two devices do not use the same one.",
        causeProtocol: "The other device sent something this version does not understand. Updating TotalDrop on both devices may help.",
        causeDiskFull: "There is not enough free space on this device.",
        causeNoPermission: "TotalDrop is not allowed to read or write a file or folder it needs.",
        causeNotFound: "A file or folder could not be found. It may have been moved or deleted.",
        causeReadOnly: "The folder is on a disk that cannot be written to.",
        causeFileChanged: "A file changed while it was being sent. Send it again once it is saved.",
        technicalDetail: "Technical message in English, to include when you report a problem:",
        settingsServer: "Rendezvous server",
        settingsServerHelp: "Helps two devices with the same code find each other. It never sees your files or keys.",
        settingsServerReset: "Use the default",
        settingsServerInvalid: "Must start with wss://",
        settingsSave: "Save",
        settingsAbout: "About",
        settingsAboutBody: "TotalDrop is free and open source, funded by donations. Licensed GPL-3.0-or-later.",
        settingsPrivacy: "Privacy",
        settingsPrivacyBody: "The rendezvous server, the relay and Cloudflare's STUN server see your IP address, and so does the other device when you connect directly (unless privacy mode is on). None of them sees file names, contents or keys, and our servers keep no logs. Files go directly between the devices when they can, otherwise through the relay; they are encrypted end to end either way.",
        privacyMode: "Hide my address from the other device",
        privacyModeHint: "Privacy mode: everything goes through the relay, so the other device never learns your IP address. Slower, and the other device must use the same relay (the default one does).",
        privacyModeConfirmTitle: "Turn on privacy mode?",
        privacyModeConfirmBody: "Transfers will be slower, because nothing goes directly between the devices. The rendezvous server and the relay still see your IP address; they never see your files or keys.",
        privacyModeTurnOn: "Turn on",
        draftTranslation: "This translation is being reviewed. Help improve it on GitHub.",
        settingsWordsEnglish: "Codes and verification words are in English in every language, so two devices set to different languages show the same words.",
        sendDropHere: "Drop files here",
        sendDropHint: "You can also drag files onto this window.",
        keepAwakeReason: "TotalDrop is sending or receiving files",
        sendQrHint: "Or scan this code with the other device's camera.",
        sendWaitingAccept: "Check the words, then wait for the receiver to accept.",
        receiveFromImage: "Read QR from image",
        receiveQrNotFound: "There is no TotalDrop code in that image.",
        pickTitle: "Choose files to send",
        pickImageTitle: "Choose an image with a QR code",
        openFolder: "Open folder",
        back: "Back",
        codeSuggestions: "Suggestions",
        sendFlatpakUnreadable: "This file cannot be read from the Flatpak. Choose it with Add files, or close this window and try again.",
        sendText: "Send text",
        sendTextHint: "Type or paste the text to send",
        sendTextDone: "The text arrived intact.",
        sendClipboard: "Send clipboard",
        sendClipboardEmpty: "The clipboard holds no text.",
        receiveOfferText: "Incoming text",
        receiveOfferTextNote: "The text is shown here, not saved.",
        receiveTextDone: "Text received and verified",
        receiveTextDeclined: "You declined the text.",
        receiveTextHidden: "The text holds hidden characters, shown as U+…. Copying gives the text exactly as it came.",
        sendTextHidden: "The text holds hidden characters, such as direction marks. They are sent with it.",
        sendClipboardKept: "The text already typed is kept; the clipboard was not used.",
        sendTextShortened: "Only the beginning of the text is shown; all of it is sent.",
        receiveTextShortened: "Only the beginning of the text is shown. Copy text copies all of it.",
        copyText: "Copy text",
        textCopied: "Text copied",
        errorTextDamaged: "The text did not arrive intact, so it is not shown."
    })

    function f(key, args) {
        const a = args || {}
        switch (key) {
        case "itemsAndSize":
            return (a.count === 1 ? "1 item" : a.count + " items") + " · " + a.size
        case "progressPercent":
            return a.percent + "%"
        case "receiveAndMore":
            return "and " + a.count + " more"
        case "receiveOfferSummary":
            return (a.files === 1 ? "1 file" : a.files + " files") + " · " + a.size
        case "receiveSaveTo":
            return "Saved to " + a.folder
        case "sendDoneBody":
            return a.count === 1 ? "The file arrived intact." : "All " + a.count + " files arrived intact."
        case "sendSomeFailed":
            return a.count === 1 ? "One file did not arrive intact and was discarded."
                                 : a.count + " files did not arrive intact and were discarded."
        case "sendTextTooLong":
            return "A text can be at most " + a.size + ". Send a longer one as a file."
        case "settingsVersion":
            return "Version " + a.version
        default:
            return key
        }
    }

    function size(bytes) {
        return Math.round(bytes) + " B"
    }

    function name(text) {
        return text
    }
}
