// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Hindi (`hi`).
class AppLocalizationsHi extends AppLocalizations {
  AppLocalizationsHi([String locale = 'hi']) : super(locale);

  @override
  String get appTagline => 'हर फ़ाइल, हर किसी को, हर जगह।';

  @override
  String get homeSendTitle => 'भेजें';

  @override
  String get homeSendBody => 'फ़ाइलें चुनें और एक छोटा कोड साझा करें';

  @override
  String get homeReceiveTitle => 'प्राप्त करें';

  @override
  String get homeReceiveBody => 'कोड डालें और फ़ाइलें सहेजें';

  @override
  String get homeFooter =>
      'एंड-टू-एंड एन्क्रिप्टेड। न खाता, न ईमेल, न फ़ोन नंबर।';

  @override
  String get settings => 'सेटिंग';

  @override
  String get sendTitle => 'फ़ाइलें भेजें';

  @override
  String get sendAddFiles => 'फ़ाइलें जोड़ें';

  @override
  String get sendEmpty => 'अभी कुछ नहीं चुना गया';

  @override
  String get sendEmptyHint =>
      'वे फ़ाइलें जोड़ें जिन्हें आप भेजना चाहते हैं। पाने वाले को ठीक यही मिलेंगी, इसके अलावा कुछ नहीं।';

  @override
  String get sendStrongCode => 'ज़्यादा मज़बूत कोड';

  @override
  String get sendStrongCodeHint =>
      'दो की जगह तीन शब्द: अनुमान लगाना कहीं ज़्यादा कठिन';

  @override
  String get sendCreateCode => 'कोड बनाएँ';

  @override
  String get sendCounting => 'गिनती हो रही है…';

  @override
  String get sendRemove => 'हटाएँ';

  @override
  String get sendPreparing => 'आपकी फ़ाइलें तैयार की जा रही हैं…';

  @override
  String get sendYourCode => 'आपका कोड';

  @override
  String get sendCodeHint =>
      'दूसरे डिवाइस पर TotalDrop खोलें, प्राप्त करें पर टैप करें और यह कोड डालें।';

  @override
  String get sendWaiting => 'पाने वाले का इंतज़ार है…';

  @override
  String get sendCopy => 'कॉपी करें';

  @override
  String get sendCopied => 'कोड कॉपी हो गया';

  @override
  String get sendShare => 'साझा करें';

  @override
  String sendShareText(String code) {
    return 'TotalDrop से मेरी फ़ाइलें लें। कोड: $code';
  }

  @override
  String get sendSending => 'भेजा जा रहा है';

  @override
  String get sendDone => 'भेजा गया और सत्यापित हुआ';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'सभी $countString फ़ाइलें सही-सलामत पहुँच गईं।',
      one: 'फ़ाइल सही-सलामत पहुँच गई।',
    );
    return '$_temp0';
  }

  @override
  String sendSomeFailed(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$countString फ़ाइलें सही-सलामत नहीं पहुँचीं और हटा दी गईं।',
      one: 'एक फ़ाइल सही-सलामत नहीं पहुँची और हटा दी गई।',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$countString आइटम',
      one: '$countString आइटम',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'फ़ाइलें प्राप्त करें';

  @override
  String get receiveCodeLabel => 'कोड';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'भेजने वाले से उनका कोड माँगें।';

  @override
  String get receiveCodeInvalid =>
      'यह कोड जैसा नहीं दिखता। कोड एक संख्या से शुरू होता है, फिर दो या तीन शब्द आते हैं।';

  @override
  String get receivePaste => 'चिपकाएँ';

  @override
  String get receiveConnect => 'कनेक्ट करें';

  @override
  String get receiveConnecting => 'सुरक्षित रूप से कनेक्ट हो रहा है…';

  @override
  String get receiveOffer => 'आने वाली फ़ाइलें';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString फ़ाइलें',
      one: '$filesString फ़ाइल',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'और $countString अन्य';
  }

  @override
  String receiveSaveTo(String folder) {
    return '$folder में सहेजा गया';
  }

  @override
  String get receiveAccept => 'स्वीकार करें';

  @override
  String get receiveDecline => 'अस्वीकार करें';

  @override
  String get receiveDeclined => 'आपने फ़ाइलें अस्वीकार कर दीं।';

  @override
  String get receiveReceiving => 'प्राप्त हो रहा है';

  @override
  String get receiveDone => 'प्राप्त हुआ और सत्यापित हुआ';

  @override
  String get verificationTitle => 'सत्यापन शब्द';

  @override
  String get verificationHint =>
      'दूसरे व्यक्ति को भी यही चार शब्द दिखते हैं। अगर वे अलग हैं, तो रुक जाएँ।';

  @override
  String get sameNetworkNote =>
      'इंटरनेट पर काम करता है। जब डिवाइस सीधे एक-दूसरे तक नहीं पहुँच पाते, तो एन्क्रिप्टेड ट्रांसफ़र रिले सर्वर से होकर जाता है, जो उसे पढ़ नहीं सकता।';

  @override
  String get done => 'हो गया';

  @override
  String get tryAgain => 'फिर से कोशिश करें';

  @override
  String get cancel => 'रद्द करें';

  @override
  String get cancelTransferTitle => 'यह ट्रांसफ़र रोकें?';

  @override
  String get cancelTransferBody =>
      'दूसरी तरफ़ बताया जाएगा कि ट्रांसफ़र रुक गया।';

  @override
  String get keepGoing => 'जारी रखें';

  @override
  String get stop => 'रोकें';

  @override
  String get moreInfo => 'विवरण';

  @override
  String get errorWrongCode =>
      'कोड मेल नहीं खाया। या तो यह गलत टाइप हुआ, या किसी ने ट्रांसफ़र को बीच में पकड़ने की कोशिश की। कुछ भी नहीं भेजा गया। नए कोड के साथ फिर से शुरू करें।';

  @override
  String get errorPeerLeft =>
      'ट्रांसफ़र पूरा होने से पहले दूसरी तरफ़ वाला चला गया।';

  @override
  String get errorDeclined => 'पाने वाले ने फ़ाइलें अस्वीकार कर दीं।';

  @override
  String get errorCancelled => 'दूसरी तरफ़ वाले ने ट्रांसफ़र रोक दिया।';

  @override
  String get errorStopped => 'इस डिवाइस पर रोका गया।';

  @override
  String get errorInvalidCode => 'यह कोड मान्य नहीं है।';

  @override
  String get errorServer =>
      'TotalDrop सर्वर तक नहीं पहुँच सके। अपना इंटरनेट कनेक्शन जाँचें।';

  @override
  String get errorConnect =>
      'दूसरे डिवाइस से कनेक्ट नहीं हो सका। जाँचें कि दोनों डिवाइस ऑनलाइन हैं और फिर से कोशिश करें।';

  @override
  String get errorInvalidOffer =>
      'प्रस्ताव ने एक सुरक्षा नियम तोड़ा, इसलिए उसे अस्वीकार कर दिया गया।';

  @override
  String get errorStorage => 'कोई फ़ाइल पढ़ी या सहेजी नहीं जा सकी।';

  @override
  String get errorOther => 'कुछ गड़बड़ हो गई।';

  @override
  String get settingsServer => 'संपर्क सर्वर';

  @override
  String get settingsServerHelp =>
      'एक ही कोड वाले दो डिवाइस को एक-दूसरे को ढूँढने में मदद करता है। यह आपकी फ़ाइलें या कुंजियाँ कभी नहीं देखता।';

  @override
  String get settingsServerReset => 'डिफ़ॉल्ट इस्तेमाल करें';

  @override
  String get settingsServerInvalid => 'wss:// से शुरू होना चाहिए';

  @override
  String get settingsSave => 'सहेजें';

  @override
  String get settingsAbout => 'परिचय';

  @override
  String get settingsAboutBody =>
      'TotalDrop मुफ़्त और ओपन सोर्स है, दान से चलता है। लाइसेंस GPL-3.0-or-later।';

  @override
  String get settingsPrivacy => 'निजता';

  @override
  String get settingsPrivacyBody =>
      'संपर्क सर्वर, रिले सर्वर और Cloudflare का STUN सर्वर आपका IP पता देखते हैं, और सीधे कनेक्ट होने पर दूसरा डिवाइस भी देखता है (जब तक निजता मोड चालू न हो)। इनमें से कोई भी फ़ाइलों के नाम, सामग्री या कुंजियाँ नहीं देखता, और हमारे सर्वर कोई लॉग नहीं रखते। जब संभव हो, फ़ाइलें सीधे डिवाइस के बीच जाती हैं, नहीं तो रिले सर्वर से होकर; दोनों ही तरह वे एंड-टू-एंड एन्क्रिप्टेड रहती हैं।';

  @override
  String get privacyMode => 'मेरा पता दूसरे डिवाइस से छिपाएँ';

  @override
  String get privacyModeHint =>
      'निजता मोड: सब कुछ रिले सर्वर से होकर जाता है, इसलिए दूसरे डिवाइस को आपका IP पता कभी पता नहीं चलता। यह धीमा है, और दूसरे डिवाइस को भी वही रिले सर्वर इस्तेमाल करना होगा (डिफ़ॉल्ट सर्वर यही करता है)।';

  @override
  String get privacyModeConfirmTitle => 'निजता मोड चालू करें?';

  @override
  String get privacyModeConfirmBody =>
      'ट्रांसफ़र धीमे होंगे, क्योंकि डिवाइस के बीच कुछ भी सीधे नहीं जाता। संपर्क सर्वर और रिले सर्वर फिर भी आपका IP पता देखते हैं; वे आपकी फ़ाइलें या कुंजियाँ कभी नहीं देखते।';

  @override
  String get privacyModeTurnOn => 'चालू करें';

  @override
  String settingsVersion(String version) {
    return 'वर्शन $version';
  }

  @override
  String sizeBytes(String value) {
    return '$value B';
  }

  @override
  String sizeKB(String value) {
    return '$value kB';
  }

  @override
  String sizeMB(String value) {
    return '$value MB';
  }

  @override
  String sizeGB(String value) {
    return '$value GB';
  }

  @override
  String sizeTB(String value) {
    return '$value TB';
  }

  @override
  String sizePB(String value) {
    return '$value PB';
  }

  @override
  String get draftTranslation =>
      'इस अनुवाद की समीक्षा हो रही है। Weblate पर इसे बेहतर बनाने में मदद करें।';

  @override
  String get sendDropHere => 'फ़ाइलें यहाँ छोड़ें';

  @override
  String get sendDropHint => 'आप फ़ाइलों को इस विंडो पर खींचकर भी ला सकते हैं।';

  @override
  String get sendQrHint => 'या दूसरे डिवाइस के कैमरे से यह कोड स्कैन करें।';

  @override
  String get sendWaitingAccept =>
      'शब्द जाँचें, फिर पाने वाले के स्वीकार करने का इंतज़ार करें।';

  @override
  String get receiveFromImage => 'इमेज से QR पढ़ें';

  @override
  String get receiveQrNotFound => 'इस इमेज में कोई TotalDrop कोड नहीं है।';

  @override
  String get pickTitle => 'भेजने के लिए फ़ाइलें चुनें';

  @override
  String get pickImageTitle => 'QR कोड वाली इमेज चुनें';

  @override
  String get openFolder => 'फ़ोल्डर खोलें';

  @override
  String get back => 'वापस';

  @override
  String get codeSuggestions => 'सुझाव';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
