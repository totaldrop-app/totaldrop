pragma Singleton
import QtQuick

// Interface text, English and Turkish, the same strings as the Android app
// (app/lib/l10n). The language follows the desktop's locale.
QtObject {
    readonly property bool turkish: Qt.locale().name.startsWith("tr")
    readonly property var en: {
            "appTagline": "Any file, to anyone, anywhere.",
            "homeSendTitle": "Send",
            "homeSendBody": "Pick files and share a short code",
            "homeReceiveTitle": "Receive",
            "homeReceiveBody": "Enter a code and save the files",
            "homeFooter": "End-to-end encrypted. No account, no email, no phone number.",
            "settings": "Settings",
            "sendTitle": "Send files",
            "sendAddFiles": "Add files",
            "sendEmpty": "Nothing selected yet",
            "sendEmptyHint": "Add the files you want to send. The receiver gets exactly these, nothing else.",
            "sendStrongCode": "Stronger code",
            "sendStrongCodeHint": "Three words instead of two: much harder to guess",
            "sendCreateCode": "Create code",
            "sendRemove": "Remove",
            "sendPreparing": "Preparing your files…",
            "sendYourCode": "Your code",
            "sendCodeHint": "On the other device, open TotalDrop, tap Receive and enter this code.",
            "sendWaiting": "Waiting for the receiver…",
            "sendCopy": "Copy",
            "sendCopied": "Code copied",
            "sendShare": "Share",
            "sendShareText": "Get my files with TotalDrop. Code: {code}",
            "sendSending": "Sending",
            "sendDone": "Sent and verified",
            "sendDoneBody": "{count, plural, =1{The file arrived intact.} other{All {count} files arrived intact.}}",
            "sendSomeFailed": "{count, plural, =1{One file did not arrive intact and was discarded.} other{{count} files did not arrive intact and were discarded.}}",
            "itemsAndSize": "{count, plural, =1{1 item} other{{count} items}} · {size}",
            "receiveTitle": "Receive files",
            "receiveCodeLabel": "Code",
            "receiveCodeHint": "7-harbor-rocket",
            "receiveCodeHelp": "Ask the sender for their code.",
            "receiveCodeInvalid": "That does not look like a code. It starts with a number, then two or three words.",
            "receivePaste": "Paste",
            "receiveConnect": "Connect",
            "receiveConnecting": "Connecting securely…",
            "receiveOffer": "Incoming files",
            "receiveOfferSummary": "{files, plural, =1{1 file} other{{files} files}} · {size}",
            "receiveAndMore": "and {count} more",
            "receiveSaveTo": "Saved to {folder}",
            "receiveAccept": "Accept",
            "receiveDecline": "Decline",
            "receiveDeclined": "You declined the files.",
            "receiveReceiving": "Receiving",
            "receiveDone": "Received and verified",
            "verificationTitle": "Verification words",
            "verificationHint": "The other person sees the same four words. If they differ, stop.",
            "sameNetworkNote": "For now both devices must be on the same network. Transfers across the internet arrive with the relay.",
            "done": "Done",
            "tryAgain": "Try again",
            "cancel": "Cancel",
            "cancelTransferTitle": "Stop this transfer?",
            "cancelTransferBody": "The other side will be told the transfer stopped.",
            "keepGoing": "Keep going",
            "stop": "Stop",
            "moreInfo": "Details",
            "errorWrongCode": "The code did not match. It was mistyped, or someone tried to intercept the transfer. Nothing was sent. Start again with a new code.",
            "errorPeerLeft": "The other side left before the transfer finished.",
            "errorDeclined": "The receiver declined the files.",
            "errorCancelled": "The other side stopped the transfer.",
            "errorInvalidCode": "That code is not valid.",
            "errorServer": "Could not reach the TotalDrop server. Check your internet connection.",
            "errorConnect": "Could not connect to the other device. For now both devices must be on the same network.",
            "errorInvalidOffer": "The offer broke a safety rule, so it was refused.",
            "errorStorage": "A file could not be read or saved.",
            "errorOther": "Something went wrong.",
            "settingsServer": "Rendezvous server",
            "settingsServerHelp": "Helps two devices with the same code find each other. It never sees your files or keys.",
            "settingsServerReset": "Use the default",
            "settingsServerInvalid": "Must start with wss://",
            "settingsSave": "Save",
            "settingsAbout": "About",
            "settingsAboutBody": "TotalDrop is free and open source, funded by donations. Licensed GPL-3.0-or-later.",
            "settingsPrivacy": "Privacy",
            "settingsPrivacyBody": "The rendezvous server sees your IP address and when you connect, but no file names, contents or keys. It keeps no logs. Files go directly between the two devices, encrypted end to end.",
            "settingsVersion": "Version {version}",
            "sendDropHere": "Drop files here",
            "sendDropHint": "You can also drag files onto this window.",
            "sendQrHint": "Or scan this code with the other device's camera.",
            "sendWaitingAccept": "Check the words, then wait for the receiver to accept.",
            "receiveFromImage": "Read QR from image",
            "receiveQrNotFound": "There is no TotalDrop code in that image.",
            "pickTitle": "Choose files to send",
            "pickImageTitle": "Choose an image with a QR code",
            "openFolder": "Open folder",
            "back": "Back",
            "codeSuggestions": "Suggestions"
    }
    readonly property var tr: {
            "appTagline": "Her dosyayı, herkese, her yere.",
            "homeSendTitle": "Gönder",
            "homeSendBody": "Dosyaları seç, kısa bir kod paylaş",
            "homeReceiveTitle": "Al",
            "homeReceiveBody": "Kodu gir, dosyaları kaydet",
            "homeFooter": "Uçtan uca şifreli. Hesap, e-posta, telefon numarası yok.",
            "settings": "Ayarlar",
            "sendTitle": "Dosya gönder",
            "sendAddFiles": "Dosya ekle",
            "sendEmpty": "Henüz bir şey seçilmedi",
            "sendEmptyHint": "Göndermek istediğiniz dosyaları ekleyin. Alıcı yalnızca bunları alır, başka hiçbir şeyi değil.",
            "sendStrongCode": "Daha güçlü kod",
            "sendStrongCodeHint": "İki yerine üç kelime: tahmin etmesi çok daha zor",
            "sendCreateCode": "Kod oluştur",
            "sendRemove": "Kaldır",
            "sendPreparing": "Dosyalarınız hazırlanıyor…",
            "sendYourCode": "Kodunuz",
            "sendCodeHint": "Diğer cihazda TotalDrop'u açın, Al'a dokunun ve bu kodu girin.",
            "sendWaiting": "Alıcı bekleniyor…",
            "sendCopy": "Kopyala",
            "sendCopied": "Kod kopyalandı",
            "sendShare": "Paylaş",
            "sendShareText": "Dosyalarımı TotalDrop ile al. Kod: {code}",
            "sendSending": "Gönderiliyor",
            "sendDone": "Gönderildi ve doğrulandı",
            "sendDoneBody": "{count, plural, =1{Dosya eksiksiz ulaştı.} other{{count} dosyanın hepsi eksiksiz ulaştı.}}",
            "sendSomeFailed": "{count, plural, =1{Bir dosya eksiksiz ulaşmadı ve silindi.} other{{count} dosya eksiksiz ulaşmadı ve silindi.}}",
            "itemsAndSize": "{count} öğe · {size}",
            "receiveTitle": "Dosya al",
            "receiveCodeLabel": "Kod",
            "receiveCodeHint": "7-harbor-rocket",
            "receiveCodeHelp": "Gönderenden kodunu isteyin.",
            "receiveCodeInvalid": "Bu bir koda benzemiyor. Kod bir sayıyla başlar, ardından iki ya da üç kelime gelir.",
            "receivePaste": "Yapıştır",
            "receiveConnect": "Bağlan",
            "receiveConnecting": "Güvenli bağlantı kuruluyor…",
            "receiveOffer": "Gelen dosyalar",
            "receiveOfferSummary": "{files} dosya · {size}",
            "receiveAndMore": "ve {count} tane daha",
            "receiveSaveTo": "{folder} klasörüne kaydedildi",
            "receiveAccept": "Kabul et",
            "receiveDecline": "Reddet",
            "receiveDeclined": "Dosyaları reddettiniz.",
            "receiveReceiving": "Alınıyor",
            "receiveDone": "Alındı ve doğrulandı",
            "verificationTitle": "Doğrulama kelimeleri",
            "verificationHint": "Karşı taraf da aynı dört kelimeyi görür. Farklıysa durun.",
            "sameNetworkNote": "Şimdilik iki cihaz aynı ağda olmalı. İnternet üzerinden gönderim, aktarma sunucusuyla gelecek.",
            "done": "Tamam",
            "tryAgain": "Tekrar dene",
            "cancel": "Vazgeç",
            "cancelTransferTitle": "Bu gönderim durdurulsun mu?",
            "cancelTransferBody": "Karşı tarafa gönderimin durduğu bildirilir.",
            "keepGoing": "Devam et",
            "stop": "Durdur",
            "moreInfo": "Ayrıntılar",
            "errorWrongCode": "Kod eşleşmedi. Ya yanlış yazıldı ya da biri gönderimi araya girerek almaya çalıştı. Hiçbir şey gönderilmedi. Yeni bir kodla yeniden başlayın.",
            "errorPeerLeft": "Karşı taraf gönderim bitmeden ayrıldı.",
            "errorDeclined": "Alıcı dosyaları reddetti.",
            "errorCancelled": "Karşı taraf gönderimi durdurdu.",
            "errorInvalidCode": "Bu kod geçerli değil.",
            "errorServer": "TotalDrop sunucusuna ulaşılamadı. İnternet bağlantınızı kontrol edin.",
            "errorConnect": "Diğer cihaza bağlanılamadı. Şimdilik iki cihaz aynı ağda olmalı.",
            "errorInvalidOffer": "Teklif bir güvenlik kuralını çiğnediği için reddedildi.",
            "errorStorage": "Bir dosya okunamadı ya da kaydedilemedi.",
            "errorOther": "Bir şeyler ters gitti.",
            "settingsServer": "Buluşma sunucusu",
            "settingsServerHelp": "Aynı kodu bilen iki cihazın birbirini bulmasını sağlar. Dosyalarınızı ve anahtarlarınızı asla görmez.",
            "settingsServerReset": "Varsayılanı kullan",
            "settingsServerInvalid": "wss:// ile başlamalı",
            "settingsSave": "Kaydet",
            "settingsAbout": "Hakkında",
            "settingsAboutBody": "TotalDrop ücretsiz ve açık kaynaktır, bağışlarla yürür. Lisansı GPL-3.0-or-later.",
            "settingsPrivacy": "Gizlilik",
            "settingsPrivacyBody": "Buluşma sunucusu IP adresinizi ve ne zaman bağlandığınızı görür, ama dosya adlarını, içerikleri ya da anahtarları görmez. Kayıt tutmaz. Dosyalar iki cihaz arasında doğrudan, uçtan uca şifreli gider.",
            "settingsVersion": "Sürüm {version}",
            "sendDropHere": "Dosyaları buraya bırakın",
            "sendDropHint": "Dosyaları bu pencereye sürükleyip de ekleyebilirsiniz.",
            "sendQrHint": "Ya da bu kodu diğer cihazın kamerasıyla okutun.",
            "sendWaitingAccept": "Kelimeleri karşılaştırın, ardından alıcının kabul etmesini bekleyin.",
            "receiveFromImage": "Resimden QR oku",
            "receiveQrNotFound": "Bu resimde TotalDrop kodu yok.",
            "pickTitle": "Gönderilecek dosyaları seçin",
            "pickImageTitle": "QR kodlu bir resim seçin",
            "openFolder": "Klasörü aç",
            "back": "Geri",
            "codeSuggestions": "Öneriler"
    }
    readonly property var s: turkish ? tr : en

    // `{name}` placeholders and ICU `{n, plural, =1{..} other{..}}`.
    function f(key, args) {
        let text = s[key] || en[key] || key
        args = args || {}
        text = plurals(text, args)
        for (const k in args) text = text.split("{" + k + "}").join(args[k])
        return text
    }

    function plurals(text, args) {
        const re = /\{(\w+), plural, /
        let m
        while ((m = re.exec(text)) !== null) {
            // Find the end of this plural block by counting braces.
            let depth = 0, end = m.index
            for (; end < text.length; end++) {
                if (text[end] === "{") depth++
                else if (text[end] === "}" && --depth === 0) break
            }
            const body = text.slice(m.index + m[0].length, end)
            const cases = {}
            const caseRe = /(=\d+|other|one)\{/g
            let c
            while ((c = caseRe.exec(body)) !== null) {
                let d = 1, i = caseRe.lastIndex
                for (; i < body.length && d > 0; i++) {
                    if (body[i] === "{") d++
                    else if (body[i] === "}") d--
                }
                cases[c[1]] = body.slice(caseRe.lastIndex, i - 1)
                caseRe.lastIndex = i
            }
            const n = Number(args[m[1]])
            const chosen = cases["=" + n] !== undefined ? cases["=" + n]
                         : (n === 1 && cases.one !== undefined ? cases.one : (cases.other || ""))
            text = text.slice(0, m.index) + chosen + text.slice(end + 1)
        }
        return text
    }

    // Locale-aware sizes: "1,2 MB" in Turkish.
    function size(bytes) {
        const units = ["B", "KB", "MB", "GB", "TB"]
        let v = bytes, i = 0
        while (v >= 1000 && i < units.length - 1) { v /= 1000; i++ }
        const digits = i === 0 || v >= 100 ? 0 : 1
        return Number(v).toLocaleString(Qt.locale(), "f", digits) + " " + units[i]
    }
}
