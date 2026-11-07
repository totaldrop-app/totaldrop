// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Arabic (`ar`).
class AppLocalizationsAr extends AppLocalizations {
  AppLocalizationsAr([String locale = 'ar']) : super(locale);

  @override
  String get appTagline => 'أي ملف، لأي شخص، في أي مكان.';

  @override
  String get homeSendTitle => 'إرسال';

  @override
  String get homeSendBody => 'اختر الملفات وشارك رمزًا قصيرًا';

  @override
  String get homeReceiveTitle => 'استلام';

  @override
  String get homeReceiveBody => 'أدخل الرمز واحفظ الملفات';

  @override
  String get homeFooter =>
      'مشفّر بين الطرفين. بلا حساب ولا بريد إلكتروني ولا رقم هاتف.';

  @override
  String get settings => 'الإعدادات';

  @override
  String get sendTitle => 'إرسال الملفات';

  @override
  String get sendAddFiles => 'إضافة ملفات';

  @override
  String get sendEmpty => 'لم يتم اختيار أي شيء بعد';

  @override
  String get sendEmptyHint =>
      'أضف الملفات التي تريد إرسالها. يحصل المستلم على هذه الملفات فقط، ولا شيء غيرها.';

  @override
  String get sendStrongCode => 'رمز أقوى';

  @override
  String get sendStrongCodeHint =>
      'ثلاث كلمات بدلًا من كلمتين: تخمينه أصعب بكثير';

  @override
  String get sendCreateCode => 'إنشاء رمز';

  @override
  String get sendCounting => 'جارٍ العد…';

  @override
  String get sendRemove => 'إزالة';

  @override
  String get sendPreparing => 'جارٍ تجهيز ملفاتك…';

  @override
  String get sendAddFailed =>
      'تعذّرت إضافة الملفات. حاول مرة أخرى. إذا تكرر ذلك، فتأكد من وجود مساحة فارغة كافية على هذا الجهاز، أو اختر الملفات من مكان آخر.';

  @override
  String get sendYourCode => 'رمزك';

  @override
  String get sendCodeHint =>
      'على الجهاز الآخر، افتح TotalDrop واضغط على استلام وأدخل هذا الرمز.';

  @override
  String get sendWaiting => 'في انتظار المستلم…';

  @override
  String get sendCopy => 'نسخ';

  @override
  String get sendCopied => 'تم نسخ الرمز';

  @override
  String get sendShare => 'مشاركة';

  @override
  String sendShareText(String code) {
    return 'استلم ملفاتي عبر TotalDrop. الرمز: $code';
  }

  @override
  String get sendSending => 'جارٍ الإرسال';

  @override
  String get sendDone => 'تم الإرسال والتحقق';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'وصلت الملفات كلها سليمة، وعددها $countString ملف.',
      many: 'وصلت الملفات كلها سليمة، وعددها $countString ملفًا.',
      few: 'وصلت الملفات الـ$countString كلها سليمة.',
      two: 'وصل الملفان سليمين.',
      one: 'وصل الملف سليمًا.',
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
      other: 'لم يصل $countString ملف سليمًا، فتم حذفها.',
      many: 'لم يصل $countString ملفًا سليمًا، فتم حذفها.',
      few: 'لم تصل $countString ملفات سليمة، فتم حذفها.',
      two: 'لم يصل ملفان سليمين، فتم حذفهما.',
      one: 'لم يصل ملف واحد سليمًا، فتم حذفه.',
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
      other: '$countString عنصر',
      many: '$countString عنصرًا',
      few: '$countString عناصر',
      two: 'عنصران',
      one: 'عنصر واحد',
      zero: 'لا عناصر',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'استلام الملفات';

  @override
  String get receiveCodeLabel => 'الرمز';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'اطلب الرمز من المرسل.';

  @override
  String get receiveCodeInvalid =>
      'لا يبدو هذا رمزًا. يبدأ الرمز برقم، تليه كلمتان أو ثلاث كلمات.';

  @override
  String get receivePaste => 'لصق';

  @override
  String get receiveConnect => 'اتصال';

  @override
  String get receiveConnecting => 'جارٍ الاتصال بأمان…';

  @override
  String get receiveOffer => 'ملفات واردة';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString ملف',
      many: '$filesString ملفًا',
      few: '$filesString ملفات',
      two: 'ملفان',
      one: 'ملف واحد',
      zero: 'لا ملفات',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'و$countString أخرى';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'تم الحفظ في $folder';
  }

  @override
  String get receiveAccept => 'قبول';

  @override
  String get receiveDecline => 'رفض';

  @override
  String get receiveDeclined => 'لقد رفضت الملفات.';

  @override
  String get receiveReceiving => 'جارٍ الاستلام';

  @override
  String get receiveDone => 'تم الاستلام والتحقق';

  @override
  String get verificationTitle => 'كلمات التحقق';

  @override
  String get verificationHint =>
      'يرى الطرف الآخر الكلمات الأربع نفسها. إذا اختلفت، فتوقف.';

  @override
  String get sameNetworkNote =>
      'يعمل عبر الإنترنت. عندما يتعذر على الجهازين الوصول أحدهما إلى الآخر مباشرة، يمر النقل المشفّر عبر خادم الترحيل، الذي لا يستطيع قراءته.';

  @override
  String get done => 'تم';

  @override
  String get tryAgain => 'إعادة المحاولة';

  @override
  String get cancel => 'إلغاء';

  @override
  String get cancelTransferTitle => 'هل تريد إيقاف هذا النقل؟';

  @override
  String get cancelTransferBody => 'سيتم إبلاغ الطرف الآخر بأن النقل توقف.';

  @override
  String get keepGoing => 'متابعة';

  @override
  String get stop => 'إيقاف';

  @override
  String get moreInfo => 'التفاصيل';

  @override
  String get errorWrongCode =>
      'الرمز غير مطابق. إما أنه كُتب بشكل خاطئ، أو أن شخصًا ما حاول اعتراض النقل. لم يُرسَل أي شيء. ابدأ من جديد برمز جديد.';

  @override
  String get errorPeerLeft => 'غادر الطرف الآخر قبل انتهاء النقل.';

  @override
  String get errorDeclined => 'رفض المستلم الملفات.';

  @override
  String get errorCancelled => 'أوقف الطرف الآخر النقل.';

  @override
  String get errorStopped => 'تم الإيقاف على هذا الجهاز.';

  @override
  String get errorInvalidCode => 'هذا الرمز غير صالح.';

  @override
  String get errorServer =>
      'تعذر الوصول إلى خادم TotalDrop. تحقق من اتصالك بالإنترنت.';

  @override
  String get errorCodeNotInUse =>
      'هذا الرمز غير مستخدم: ربما كُتب بشكل خاطئ، أو انتهت صلاحيته، أو استُخدم من قبل. اطلب رمزًا جديدًا من المرسل.';

  @override
  String get errorCodeExpired =>
      'انتهت صلاحية الرمز قبل أن يستخدمه أحد. أرسل مرة أخرى للحصول على رمز جديد.';

  @override
  String get errorBadServer => 'عنوان الخادم غير صالح. يجب أن يبدأ بـ wss://.';

  @override
  String get errorConnect =>
      'تعذر الاتصال بالجهاز الآخر. تأكد من أن الجهازين كليهما متصلان بالإنترنت، ثم أعد المحاولة.';

  @override
  String get errorInvalidOffer => 'خالف العرض قاعدة أمان، لذلك تم رفضه.';

  @override
  String get errorStorage => 'تعذرت قراءة ملف أو حفظه.';

  @override
  String get errorOther => 'حدث خطأ ما.';

  @override
  String get causeTimedOut =>
      'توقف الاتصال عن الاستجابة. ربما فقد أحد الجهازين الشبكة أو دخل في وضع السكون.';

  @override
  String get causeConnectionLost => 'انقطع الاتصال بالجهاز الآخر.';

  @override
  String get causeNoRoute =>
      'لم تنجح أي طريقة للوصول إلى الجهاز الآخر، لا مباشرة ولا عبر خادم الترحيل.';

  @override
  String get causeRelay =>
      'تعذر الوصول إلى خادم الترحيل أو أنه غير مُعدّ، أو أن الجهازين لا يستخدمان الخادم نفسه.';

  @override
  String get causeProtocol =>
      'أرسل الجهاز الآخر شيئًا لا يفهمه هذا الإصدار. قد يساعد تحديث TotalDrop على الجهازين.';

  @override
  String get causeDiskFull => 'لا توجد مساحة فارغة كافية على هذا الجهاز.';

  @override
  String get causeNoPermission =>
      'لا يُسمح لـ TotalDrop بقراءة ملف أو مجلد يحتاج إليه أو الكتابة فيه.';

  @override
  String get causeNotFound => 'تعذر العثور على ملف أو مجلد. ربما نُقل أو حُذف.';

  @override
  String get causeReadOnly => 'المجلد على قرص لا تمكن الكتابة عليه.';

  @override
  String get causeFileChanged =>
      'تغيّر ملف أثناء إرساله. أرسله مرة أخرى بعد حفظه.';

  @override
  String get technicalDetail =>
      'رسالة تقنية بالإنجليزية، لإرفاقها عند الإبلاغ عن مشكلة:';

  @override
  String get settingsServer => 'خادم الالتقاء';

  @override
  String get settingsServerHelp =>
      'يساعد جهازين يحملان الرمز نفسه على أن يجد أحدهما الآخر. لا يرى ملفاتك أو مفاتيحك أبدًا.';

  @override
  String get settingsServerReset => 'استخدام الخادم الافتراضي';

  @override
  String get settingsServerInvalid => 'يجب أن يبدأ بـ wss://';

  @override
  String get settingsSave => 'حفظ';

  @override
  String get settingsAbout => 'حول';

  @override
  String get settingsAboutBody =>
      'TotalDrop مجاني ومفتوح المصدر، ويُموَّل بالتبرعات. مرخّص بموجب GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'الخصوصية';

  @override
  String get settingsPrivacyBody =>
      'يرى خادم الالتقاء وخادم الترحيل وخادم STUN التابع لـ Cloudflare عنوان IP الخاص بك، وكذلك الجهاز الآخر عند الاتصال المباشر (ما لم يكن وضع الخصوصية مفعّلًا). لا يرى أي منها أسماء الملفات أو محتواها أو المفاتيح، ولا تحتفظ خوادمنا بأي سجلات. تنتقل الملفات مباشرة بين الجهازين متى أمكن، وإلا فعبر خادم الترحيل؛ وهي مشفّرة بين الطرفين في الحالتين.';

  @override
  String get privacyMode => 'إخفاء عنواني عن الجهاز الآخر';

  @override
  String get privacyModeHint =>
      'وضع الخصوصية: يمر كل شيء عبر خادم الترحيل، فلا يعرف الجهاز الآخر عنوان IP الخاص بك أبدًا. النقل أبطأ، ويجب أن يستخدم الجهاز الآخر خادم الترحيل نفسه (الخادم الافتراضي يفي بذلك).';

  @override
  String get privacyModeConfirmTitle => 'هل تريد تفعيل وضع الخصوصية؟';

  @override
  String get privacyModeConfirmBody =>
      'ستصبح عمليات النقل أبطأ، لأن لا شيء ينتقل مباشرة بين الجهازين. يظل خادم الالتقاء وخادم الترحيل يريان عنوان IP الخاص بك، لكنهما لا يريان ملفاتك أو مفاتيحك أبدًا.';

  @override
  String get privacyModeTurnOn => 'تفعيل';

  @override
  String settingsVersion(String version) {
    return 'الإصدار $version';
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
      'هذه الترجمة قيد المراجعة. ساعد في تحسينها على GitHub.';

  @override
  String get settingsWordsEnglish =>
      'تبقى الرموز وكلمات التحقق بالإنجليزية في كل اللغات، حتى يعرض جهازان بلغتين مختلفتين الكلمات نفسها.';

  @override
  String get sendDropHere => 'أفلت الملفات هنا';

  @override
  String get sendDropHint => 'يمكنك أيضًا سحب الملفات إلى هذه النافذة.';

  @override
  String get sendQrHint => 'أو امسح رمز QR هذا بكاميرا الجهاز الآخر.';

  @override
  String get sendWaitingAccept => 'تحقق من الكلمات، ثم انتظر حتى يقبل المستلم.';

  @override
  String get receiveFromImage => 'قراءة رمز QR من صورة';

  @override
  String get receiveQrNotFound => 'لا يوجد رمز TotalDrop في هذه الصورة.';

  @override
  String get pickTitle => 'اختر الملفات المراد إرسالها';

  @override
  String get pickImageTitle => 'اختر صورة تحتوي على رمز QR';

  @override
  String get openFolder => 'فتح المجلد';

  @override
  String get back => 'رجوع';

  @override
  String get codeSuggestions => 'اقتراحات';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString٪';
  }

  @override
  String get editUndo => 'تراجع';

  @override
  String get editRedo => 'إعادة';

  @override
  String get editCut => 'قص';

  @override
  String get editCopy => 'نسخ';

  @override
  String get editPaste => 'لصق';

  @override
  String get editDelete => 'حذف';

  @override
  String get editSelectAll => 'تحديد الكل';

  @override
  String get transferNoticeChannel => 'عمليات النقل';

  @override
  String get transferNoticeSending => 'جارٍ الإرسال عبر TotalDrop';

  @override
  String get transferNoticeReceiving => 'جارٍ الاستلام عبر TotalDrop';

  @override
  String get transferNoticeBody => 'يستمر أثناء استخدامك تطبيقات أخرى.';

  @override
  String get shareBusy =>
      'لا يزال هناك نقل جارٍ. انتظر حتى ينتهي أو أوقفه، ثم شارك مرة أخرى.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'تعذّرت قراءة $count عنصر مشترك، فتم استبعادها.',
      many: 'تعذّرت قراءة $count عنصرًا مشتركًا، فتم استبعادها.',
      few: 'تعذّرت قراءة $count عناصر مشتركة، فتم استبعادها.',
      two: 'تعذّرت قراءة عنصرين مشتركين، فتم استبعادهما.',
      one: 'تعذّرت قراءة عنصر مشترك واحد، فتم استبعاده.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'نقل الملفات';

  @override
  String get desktopComment =>
      'أرسل الملفات مباشرة إلى جهاز آخر، بتشفير تام بين الطرفين';

  @override
  String get desktopKeywords => 'إرسال;استلام;مشاركة;نقل;ملفات;';

  @override
  String get desktopSendWith => 'إرسال باستخدام TotalDrop';

  @override
  String get metainfoSummary => 'أرسل الملفات مباشرة إلى جهاز آخر';

  @override
  String get metainfoAbout =>
      'يرسل TotalDrop الملفات والمجلدات من جهاز إلى آخر بتشفير تام بين الطرفين. اختر الملفات وشارك رمزًا قصيرًا أو رمز QR، فيستلمها الجهاز الآخر. بلا حساب ولا بريد إلكتروني ولا رقم هاتف.';

  @override
  String get metainfoVerified =>
      'يُتحقَّق من كل ملف عند وصوله، وتُحذف الملفات التالفة';

  @override
  String get metainfoWords => 'أربع كلمات تحقق تُظهر أن أحدًا لم يتدخل بينكما';

  @override
  String get metainfoTheme =>
      'يتبع سمة سطح المكتب لديك: Plasma وHyprland وOmarchy وGNOME';

  @override
  String get metainfoOnePackage =>
      'استخدم حزمة Linux واحدة في كل مرة. تشترك حزمة Flatpak وحزمة .pkg أو .deb أو .rpm في اسم واحد، لذا عندما تكون إحداهما مفتوحة، يؤدي تشغيل الأخرى إلى إبراز النافذة المفتوحة بدلًا من فتح نافذة ثانية.';

  @override
  String get otherPackageFlatpak =>
      'TotalDrop مفتوح بالفعل من حزمة Flatpak، لذا أُبرزت تلك النافذة. استخدم حزمة واحدة في كل مرة: أغلق تلك النافذة لفتح هذه.';

  @override
  String get otherPackageSystem =>
      'TotalDrop مفتوح بالفعل من حزمة .pkg أو .deb أو .rpm، لذا أُبرزت تلك النافذة. استخدم حزمة واحدة في كل مرة: أغلق تلك النافذة لفتح هذه.';

  @override
  String get sendText => 'إرسال نص';

  @override
  String get sendTextHint => 'اكتب النص الذي تريد إرساله أو الصقه';

  @override
  String sendTextTooLong(String size) {
    return 'يمكن أن يصل النص إلى $size على الأكثر. أرسل النص الأطول كملف.';
  }

  @override
  String get sendTextDone => 'وصل النص كاملًا.';

  @override
  String get sendClipboard => 'إرسال محتوى الحافظة';

  @override
  String get sendClipboardEmpty => 'لا يوجد نص في الحافظة.';

  @override
  String get receiveOfferText => 'نص وارد';

  @override
  String get receiveOfferTextNote => 'يُعرض النص هنا ولا يُحفظ.';

  @override
  String get receiveTextDone => 'تم استلام النص والتحقق منه';

  @override
  String get receiveTextDeclined => 'لقد رفضت النص.';

  @override
  String get receiveTextHidden =>
      'يحتوي النص على أحرف مخفية تظهر على شكل U+…. عند النسخ يُنسخ النص كما وصل تمامًا.';

  @override
  String get sendTextHidden =>
      'يحتوي النص على أحرف مخفية، مثل علامات الاتجاه. وتُرسل معه أيضًا.';

  @override
  String get sendClipboardKept =>
      'تم الإبقاء على النص المكتوب، ولم تُستخدم الحافظة.';

  @override
  String get sendTextShortened => 'يظهر بداية النص فقط، ويُرسَل كله.';

  @override
  String get receiveTextShortened =>
      'تظهر بداية النص فقط. «نسخ النص» ينسخه كله.';

  @override
  String get copyText => 'نسخ النص';

  @override
  String get textCopied => 'تم نسخ النص';

  @override
  String get errorTextDamaged => 'لم يصل النص كاملًا، لذلك لا يُعرض.';

  @override
  String get setupBackgroundTitle => 'استمرار عمليات النقل في الخلفية';

  @override
  String get setupBackgroundBody =>
      'لكي يستمر النقل عند التبديل إلى تطبيق آخر أو قفل الشاشة، اسمح لـ TotalDrop بالعمل بلا قيود على البطارية. لا يعمل TotalDrop في الخلفية إلا أثناء نقل بدأته أنت.';

  @override
  String get setupBackgroundAllowed =>
      'تحسين البطارية في Android متوقف لـ TotalDrop.';

  @override
  String get setupBackgroundRestricted =>
      'تحسين البطارية في Android مفعّل لـ TotalDrop: قد يتوقف النقل عند التبديل إلى تطبيق آخر.';

  @override
  String get setupBackgroundBattery => 'فتح إعدادات البطارية';

  @override
  String get setupBackgroundBatteryHint =>
      'ابحث عن TotalDrop في القائمة وأوقف تحسين البطارية له. تختلف أسماء هذه الإعدادات من هاتف إلى آخر.';

  @override
  String setupBackgroundMaker(String maker) {
    return 'لهواتف $maker موفّر بطارية خاص بها';
  }

  @override
  String get setupBackgroundMakerBody =>
      'افتح أيضًا موفّر البطارية الخاص بـ TotalDrop واختر \"بلا قيود\". وإن كان في الهاتف خيار التشغيل التلقائي، ففعّله لـ TotalDrop.';

  @override
  String get setupBackgroundMakerOpen => 'فتح موفّر البطارية لـ TotalDrop';

  @override
  String get setupBackgroundAutostart => 'فتح التشغيل التلقائي';

  @override
  String get setupBackgroundSkip => 'ليس الآن';

  @override
  String get settingsBackground => 'النقل في الخلفية';

  @override
  String get settingsBackgroundOpen => 'إعدادات البطارية';
}
