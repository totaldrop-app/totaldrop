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
  String get sendAddFailed =>
      'फ़ाइलें जोड़ी नहीं जा सकीं। दोबारा कोशिश करें। अगर फिर से ऐसा हो, तो देखें कि इस डिवाइस पर पर्याप्त खाली जगह है, या फ़ाइलें किसी दूसरी जगह से चुनें।';

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
  String get errorCodeNotInUse =>
      'यह कोड इस्तेमाल में नहीं है: यह गलत टाइप हुआ, इसकी अवधि खत्म हो गई या यह पहले ही इस्तेमाल हो चुका है। भेजने वाले से नया कोड माँगें।';

  @override
  String get errorCodeExpired =>
      'किसी के इस्तेमाल करने से पहले ही कोड की अवधि खत्म हो गई। नया कोड पाने के लिए फिर से भेजें।';

  @override
  String get errorBadServer =>
      'सर्वर का पता मान्य नहीं है। यह wss:// से शुरू होना चाहिए।';

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
  String get causeTimedOut =>
      'कनेक्शन ने जवाब देना बंद कर दिया। हो सकता है किसी डिवाइस का नेटवर्क चला गया हो या वह स्लीप में चला गया हो।';

  @override
  String get causeConnectionLost => 'दूसरे डिवाइस से कनेक्शन टूट गया।';

  @override
  String get causeNoRoute =>
      'दूसरे डिवाइस तक न सीधे पहुँचा जा सका, न रिले सर्वर के ज़रिए।';

  @override
  String get causeRelay =>
      'रिले सर्वर तक नहीं पहुँचा जा सका या वह सेट नहीं है, या दोनों डिवाइस एक ही रिले सर्वर इस्तेमाल नहीं करते।';

  @override
  String get causeProtocol =>
      'दूसरे डिवाइस ने कुछ ऐसा भेजा जो यह संस्करण नहीं समझता। दोनों डिवाइस पर TotalDrop अपडेट करने से मदद मिल सकती है।';

  @override
  String get causeDiskFull => 'इस डिवाइस पर पर्याप्त खाली जगह नहीं है।';

  @override
  String get causeNoPermission =>
      'TotalDrop को किसी ज़रूरी फ़ाइल या फ़ोल्डर को पढ़ने या लिखने की अनुमति नहीं है।';

  @override
  String get causeNotFound =>
      'कोई फ़ाइल या फ़ोल्डर नहीं मिला। हो सकता है उसे हटाया या मिटाया गया हो।';

  @override
  String get causeReadOnly =>
      'फ़ोल्डर ऐसी डिस्क पर है जिस पर लिखा नहीं जा सकता।';

  @override
  String get causeFileChanged =>
      'भेजते समय एक फ़ाइल बदल गई। सेव होने के बाद उसे फिर से भेजें।';

  @override
  String get technicalDetail =>
      'अंग्रेज़ी में तकनीकी संदेश, समस्या की रिपोर्ट करते समय जोड़ने के लिए:';

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
      'संपर्क सर्वर, रिले सर्वर और Cloudflare का STUN सर्वर आपका IP पता देखते हैं, और सीधे कनेक्ट होने पर दूसरा डिवाइस भी देखता है (जब तक निजता मोड चालू न हो)। इनमें से कोई भी फ़ाइलों के नाम, सामग्री या कुंजियाँ नहीं देखता। हमारे सर्वर अपना कोई लॉग नहीं रखते, लेकिन उन्हें चलाने वाला Cloudflare IP पतों के साथ अनुरोधों का एक नमूना कम से कम 31 दिन रखता है। जब संभव हो, फ़ाइलें सीधे डिवाइस के बीच जाती हैं, नहीं तो रिले सर्वर से होकर; दोनों ही तरह वे एंड-टू-एंड एन्क्रिप्टेड रहती हैं।';

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
      'इस अनुवाद की समीक्षा हो रही है। GitHub पर इसे बेहतर बनाने में मदद करें।';

  @override
  String get settingsWordsEnglish =>
      'कोड और सत्यापन शब्द हर भाषा में अंग्रेज़ी में रहते हैं, ताकि अलग-अलग भाषाओं वाले दो डिवाइस एक जैसे शब्द दिखाएँ।';

  @override
  String get sendDropHere => 'फ़ाइलें यहाँ छोड़ें';

  @override
  String get sendDropHint => 'आप फ़ाइलों को इस विंडो पर खींचकर भी ला सकते हैं।';

  @override
  String get keepAwakeReason => 'TotalDrop फ़ाइलें भेज या पा रहा है';

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

  @override
  String get editUndo => 'पूर्ववत करें';

  @override
  String get editRedo => 'फिर से करें';

  @override
  String get editCut => 'काटें';

  @override
  String get editCopy => 'कॉपी करें';

  @override
  String get editPaste => 'चिपकाएँ';

  @override
  String get editDelete => 'हटाएँ';

  @override
  String get editSelectAll => 'सभी चुनें';

  @override
  String get transferNoticeChannel => 'स्थानांतरण';

  @override
  String get transferNoticeSending => 'TotalDrop से भेजा जा रहा है';

  @override
  String get transferNoticeReceiving => 'TotalDrop से प्राप्त किया जा रहा है';

  @override
  String get transferNoticeBody =>
      'दूसरे ऐप इस्तेमाल करते समय भी चलता रहता है।';

  @override
  String get shareBusy =>
      'एक ट्रांसफ़र अभी चल रहा है। उसके पूरा होने का इंतज़ार करें या उसे रोकें, फिर दोबारा शेयर करें।';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'शेयर किए गए $count आइटम पढ़े नहीं जा सके, इसलिए छोड़ दिए गए।',
      one: 'शेयर किया गया एक आइटम पढ़ा नहीं जा सका, इसलिए छोड़ दिया गया।',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'फ़ाइल ट्रांसफ़र';

  @override
  String get desktopComment =>
      'फ़ाइलें सीधे दूसरे डिवाइस पर भेजें, एंड-टू-एंड एन्क्रिप्टेड';

  @override
  String get desktopKeywords => 'भेजें;पाएं;शेयर;ट्रांसफ़र;फ़ाइलें;';

  @override
  String get desktopSendWith => 'TotalDrop से भेजें';

  @override
  String get metainfoSummary => 'फ़ाइलें सीधे दूसरे डिवाइस पर भेजें';

  @override
  String get metainfoAbout =>
      'TotalDrop फ़ाइलें और फ़ोल्डर एक डिवाइस से दूसरे पर भेजता है, एंड-टू-एंड एन्क्रिप्टेड। फ़ाइलें चुनें, एक छोटा कोड या QR कोड शेयर करें, और दूसरा डिवाइस उन्हें पा लेगा। न खाता, न ईमेल, न फ़ोन नंबर।';

  @override
  String get metainfoVerified =>
      'पहुँचने पर हर फ़ाइल की जाँच होती है; ख़राब फ़ाइलें हटा दी जाती हैं';

  @override
  String get metainfoWords =>
      'चार सत्यापन शब्द दिखाते हैं कि बीच में कोई नहीं है';

  @override
  String get metainfoTheme =>
      'आपके डेस्कटॉप की थीम अपनाता है: Plasma, Hyprland, Omarchy, GNOME';

  @override
  String get metainfoOnePackage =>
      'एक समय में एक ही Linux पैकेज इस्तेमाल करें। Flatpak और .pkg, .deb या .rpm पैकेज का नाम एक ही है, इसलिए एक खुला हो तो दूसरा चलाने पर दूसरी विंडो नहीं खुलती, खुली विंडो सामने आ जाती है।';

  @override
  String get otherPackageFlatpak =>
      'TotalDrop पहले से Flatpak से खुला है, इसलिए वही विंडो सामने लाई गई। एक समय में एक ही पैकेज इस्तेमाल करें: इसे खोलने के लिए उस विंडो को बंद करें।';

  @override
  String get otherPackageSystem =>
      'TotalDrop पहले से .pkg, .deb या .rpm पैकेज से खुला है, इसलिए वही विंडो सामने लाई गई। एक समय में एक ही पैकेज इस्तेमाल करें: इसे खोलने के लिए उस विंडो को बंद करें।';

  @override
  String get sendText => 'टेक्स्ट भेजें';

  @override
  String get sendTextHint => 'भेजने के लिए टेक्स्ट लिखें या पेस्ट करें';

  @override
  String sendTextTooLong(String size) {
    return 'टेक्स्ट ज़्यादा से ज़्यादा $size का हो सकता है। लंबा टेक्स्ट फ़ाइल के रूप में भेजें।';
  }

  @override
  String get sendTextDone => 'टेक्स्ट पूरा पहुँच गया।';

  @override
  String get sendClipboard => 'क्लिपबोर्ड भेजें';

  @override
  String get sendClipboardEmpty => 'क्लिपबोर्ड में कोई टेक्स्ट नहीं है।';

  @override
  String get receiveOfferText => 'आने वाला टेक्स्ट';

  @override
  String get receiveOfferTextNote =>
      'टेक्स्ट यहाँ दिखाया जाता है, सहेजा नहीं जाता।';

  @override
  String get receiveTextDone => 'टेक्स्ट प्राप्त हुआ और सत्यापित हुआ';

  @override
  String get receiveTextDeclined => 'आपने टेक्स्ट अस्वीकार कर दिया।';

  @override
  String get receiveTextHidden =>
      'टेक्स्ट में छिपे हुए अक्षर हैं, जो U+… के रूप में दिखाए गए हैं। कॉपी करने पर टेक्स्ट ठीक वैसा ही मिलता है जैसा आया था।';

  @override
  String get sendTextHidden =>
      'टेक्स्ट में दिशा-चिह्न जैसे छिपे हुए अक्षर हैं। ये भी टेक्स्ट के साथ भेजे जाते हैं।';

  @override
  String get sendClipboardKept =>
      'पहले से लिखा पाठ रखा गया है; क्लिपबोर्ड का उपयोग नहीं हुआ।';

  @override
  String get sendTextShortened =>
      'पाठ का केवल आरंभ दिखाया गया है; पूरा पाठ भेजा जाएगा।';

  @override
  String get receiveTextShortened =>
      'पाठ का केवल आरंभ दिखाया गया है। पाठ कॉपी करें पूरा पाठ कॉपी करता है।';

  @override
  String get copyText => 'टेक्स्ट कॉपी करें';

  @override
  String get textCopied => 'टेक्स्ट कॉपी हो गया';

  @override
  String get errorTextDamaged =>
      'टेक्स्ट पूरा नहीं पहुँचा, इसलिए दिखाया नहीं जा रहा।';

  @override
  String get setupBackgroundTitle => 'ट्रांसफ़र बैकग्राउंड में चलते रहें';

  @override
  String get setupBackgroundBody =>
      'ऐप बदलने या स्क्रीन लॉक करने पर भी ट्रांसफ़र चलता रहे, इसके लिए TotalDrop को बिना बैटरी प्रतिबंध के चलने दें। TotalDrop बैकग्राउंड में केवल तभी काम करता है जब आपका शुरू किया हुआ ट्रांसफ़र चल रहा हो।';

  @override
  String get setupBackgroundAllowed =>
      'TotalDrop के लिए Android का बैटरी ऑप्टिमाइज़ेशन बंद है।';

  @override
  String get setupBackgroundRestricted =>
      'TotalDrop के लिए Android का बैटरी ऑप्टिमाइज़ेशन चालू है: ऐप बदलने पर ट्रांसफ़र रुक सकता है।';

  @override
  String get setupBackgroundBattery => 'बैटरी सेटिंग खोलें';

  @override
  String get setupBackgroundBatteryHint =>
      'TotalDrop के ऐप जानकारी पेज पर \"बैटरी\" खोलें और बिना रोक-टोक इस्तेमाल की अनुमति दें। इन सेटिंग के नाम हर फ़ोन में अलग होते हैं।';

  @override
  String setupBackgroundMaker(String maker) {
    return '$maker फ़ोन का अपना बैटरी सेवर होता है';
  }

  @override
  String get setupBackgroundMakerBody =>
      'TotalDrop का बैटरी सेवर भी खोलें और \"कोई प्रतिबंध नहीं\" चुनें। फ़ोन में ऑटोस्टार्ट हो तो उसे TotalDrop के लिए चालू करें।';

  @override
  String get setupBackgroundMakerOpen => 'TotalDrop का बैटरी सेवर खोलें';

  @override
  String get setupBackgroundAutostart => 'ऑटोस्टार्ट खोलें';

  @override
  String get setupBackgroundSkip => 'अभी नहीं';

  @override
  String get settingsBackground => 'बैकग्राउंड में ट्रांसफ़र';

  @override
  String get settingsBackgroundOpen => 'बैटरी सेटिंग';
}
