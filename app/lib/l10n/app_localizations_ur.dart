// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Urdu (`ur`).
class AppLocalizationsUr extends AppLocalizations {
  AppLocalizationsUr([String locale = 'ur']) : super(locale);

  @override
  String get appTagline => 'کوئی بھی فائل، کسی کو بھی، کہیں بھی۔';

  @override
  String get homeSendTitle => 'بھیجیں';

  @override
  String get homeSendBody => 'فائلیں منتخب کریں اور ایک مختصر کوڈ شیئر کریں';

  @override
  String get homeReceiveTitle => 'وصول کریں';

  @override
  String get homeReceiveBody => 'کوڈ درج کریں اور فائلیں محفوظ کریں';

  @override
  String get homeFooter =>
      'اینڈ ٹو اینڈ انکرپٹڈ۔ نہ اکاؤنٹ، نہ ای میل، نہ فون نمبر۔';

  @override
  String get settings => 'ترتیبات';

  @override
  String get sendTitle => 'فائلیں بھیجیں';

  @override
  String get sendAddFiles => 'فائلیں شامل کریں';

  @override
  String get sendEmpty => 'ابھی کچھ منتخب نہیں کیا گیا';

  @override
  String get sendEmptyHint =>
      'وہ فائلیں شامل کریں جو آپ بھیجنا چاہتے ہیں۔ وصول کنندہ کو بالکل یہی ملیں گی، اور کچھ نہیں۔';

  @override
  String get sendStrongCode => 'زیادہ مضبوط کوڈ';

  @override
  String get sendStrongCodeHint =>
      'دو کے بجائے تین الفاظ: اندازہ لگانا کہیں زیادہ مشکل';

  @override
  String get sendCreateCode => 'کوڈ بنائیں';

  @override
  String get sendCounting => 'گنتی ہو رہی ہے…';

  @override
  String get sendRemove => 'ہٹائیں';

  @override
  String get sendPreparing => 'آپ کی فائلیں تیار کی جا رہی ہیں…';

  @override
  String get sendYourCode => 'آپ کا کوڈ';

  @override
  String get sendCodeHint =>
      'دوسرے آلے پر TotalDrop کھولیں، وصول کریں پر ٹیپ کریں اور یہ کوڈ درج کریں۔';

  @override
  String get sendWaiting => 'وصول کنندہ کا انتظار ہے…';

  @override
  String get sendCopy => 'کاپی کریں';

  @override
  String get sendCopied => 'کوڈ کاپی ہو گیا';

  @override
  String get sendShare => 'شیئر کریں';

  @override
  String sendShareText(String code) {
    return 'میری فائلیں TotalDrop سے حاصل کریں۔ کوڈ: $code';
  }

  @override
  String get sendSending => 'بھیجا جا رہا ہے';

  @override
  String get sendDone => 'بھیج دیا گیا اور تصدیق ہو گئی';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'تمام $countString فائلیں صحیح سلامت پہنچ گئیں۔',
      one: 'فائل صحیح سلامت پہنچ گئی۔',
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
      other:
          '$countString فائلیں صحیح سلامت نہیں پہنچیں اور انہیں مسترد کر دیا گیا۔',
      one: 'ایک فائل صحیح سلامت نہیں پہنچی اور اسے مسترد کر دیا گیا۔',
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
      other: '$countString آئٹمز',
      one: '$countString آئٹم',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'فائلیں وصول کریں';

  @override
  String get receiveCodeLabel => 'کوڈ';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'بھیجنے والے سے ان کا کوڈ مانگیں۔';

  @override
  String get receiveCodeInvalid =>
      'یہ کوڈ نہیں لگتا۔ کوڈ ایک نمبر سے شروع ہوتا ہے، پھر دو یا تین الفاظ آتے ہیں۔';

  @override
  String get receivePaste => 'پیسٹ کریں';

  @override
  String get receiveConnect => 'منسلک کریں';

  @override
  String get receiveConnecting => 'محفوظ طریقے سے منسلک ہو رہا ہے…';

  @override
  String get receiveOffer => 'آنے والی فائلیں';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString فائلیں',
      one: '$filesString فائل',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'اور $countString مزید';
  }

  @override
  String receiveSaveTo(String folder) {
    return '$folder میں محفوظ کر دیا گیا';
  }

  @override
  String get receiveAccept => 'قبول کریں';

  @override
  String get receiveDecline => 'مسترد کریں';

  @override
  String get receiveDeclined => 'آپ نے فائلیں مسترد کر دیں۔';

  @override
  String get receiveReceiving => 'وصول ہو رہا ہے';

  @override
  String get receiveDone => 'وصول ہو گیا اور تصدیق ہو گئی';

  @override
  String get verificationTitle => 'تصدیقی الفاظ';

  @override
  String get verificationHint =>
      'دوسرا شخص بھی یہی چار الفاظ دیکھتا ہے۔ اگر الفاظ مختلف ہوں تو رک جائیں۔';

  @override
  String get sameNetworkNote =>
      'انٹرنیٹ پر کام کرتا ہے۔ جب آلات ایک دوسرے تک براہ راست نہ پہنچ سکیں تو انکرپٹڈ منتقلی ریلے سے گزرتی ہے، جو اسے پڑھ نہیں سکتا۔';

  @override
  String get done => 'ہو گیا';

  @override
  String get tryAgain => 'دوبارہ کوشش کریں';

  @override
  String get cancel => 'منسوخ کریں';

  @override
  String get cancelTransferTitle => 'یہ منتقلی روک دیں؟';

  @override
  String get cancelTransferBody =>
      'دوسری طرف کو بتا دیا جائے گا کہ منتقلی روک دی گئی۔';

  @override
  String get keepGoing => 'جاری رکھیں';

  @override
  String get stop => 'روکیں';

  @override
  String get moreInfo => 'تفصیلات';

  @override
  String get errorWrongCode =>
      'کوڈ میل نہیں کھایا۔ یا تو یہ غلط ٹائپ ہوا، یا کسی نے منتقلی میں مداخلت کی کوشش کی۔ کچھ نہیں بھیجا گیا۔ نئے کوڈ کے ساتھ دوبارہ شروع کریں۔';

  @override
  String get errorPeerLeft => 'منتقلی مکمل ہونے سے پہلے دوسری طرف چلی گئی۔';

  @override
  String get errorDeclined => 'وصول کنندہ نے فائلیں مسترد کر دیں۔';

  @override
  String get errorCancelled => 'دوسری طرف نے منتقلی روک دی۔';

  @override
  String get errorStopped => 'اس آلے پر روک دیا گیا۔';

  @override
  String get errorInvalidCode => 'یہ کوڈ درست نہیں ہے۔';

  @override
  String get errorServer =>
      'TotalDrop سرور تک رسائی نہیں ہو سکی۔ اپنا انٹرنیٹ کنکشن چیک کریں۔';

  @override
  String get errorConnect =>
      'دوسرے آلے سے رابطہ نہیں ہو سکا۔ یقینی بنائیں کہ دونوں آلات آن لائن ہیں اور دوبارہ کوشش کریں۔';

  @override
  String get errorInvalidOffer =>
      'پیشکش نے ایک حفاظتی اصول توڑا، اس لیے اسے مسترد کر دیا گیا۔';

  @override
  String get errorStorage => 'کوئی فائل پڑھی یا محفوظ نہیں کی جا سکی۔';

  @override
  String get errorOther => 'کچھ غلط ہو گیا۔';

  @override
  String get settingsServer => 'رابطہ سرور';

  @override
  String get settingsServerHelp =>
      'ایک ہی کوڈ والے دو آلات کو ایک دوسرے کو ڈھونڈنے میں مدد دیتا ہے۔ یہ کبھی آپ کی فائلیں یا کلیدیں نہیں دیکھتا۔';

  @override
  String get settingsServerReset => 'ڈیفالٹ استعمال کریں';

  @override
  String get settingsServerInvalid => 'wss:// سے شروع ہونا چاہیے';

  @override
  String get settingsSave => 'محفوظ کریں';

  @override
  String get settingsAbout => 'تعارف';

  @override
  String get settingsAboutBody =>
      'TotalDrop مفت اور اوپن سورس ہے، اور عطیات سے چلتا ہے۔ لائسنس GPL-3.0-or-later۔';

  @override
  String get settingsPrivacy => 'رازداری';

  @override
  String get settingsPrivacyBody =>
      'رابطہ سرور، ریلے اور Cloudflare کا STUN سرور آپ کا IP پتہ دیکھتے ہیں، اور براہ راست منسلک ہونے پر دوسرا آلہ بھی دیکھتا ہے (جب تک رازداری موڈ آن نہ ہو)۔ ان میں سے کوئی بھی فائلوں کے نام، مواد یا کلیدیں نہیں دیکھتا، اور ہمارے سرور کوئی لاگ نہیں رکھتے۔ جب ممکن ہو فائلیں براہ راست آلات کے درمیان جاتی ہیں، ورنہ ریلے کے ذریعے؛ دونوں صورتوں میں وہ اینڈ ٹو اینڈ انکرپٹڈ ہوتی ہیں۔';

  @override
  String get privacyMode => 'میرا پتہ دوسرے آلے سے چھپائیں';

  @override
  String get privacyModeHint =>
      'رازداری موڈ: سب کچھ ریلے سے گزرتا ہے، اس لیے دوسرے آلے کو کبھی آپ کا IP پتہ معلوم نہیں ہوتا۔ یہ سست ہے، اور دوسرے آلے کو بھی وہی ریلے استعمال کرنا ہوگا (ڈیفالٹ ریلے یہی کرتا ہے)۔';

  @override
  String get privacyModeConfirmTitle => 'رازداری موڈ آن کریں؟';

  @override
  String get privacyModeConfirmBody =>
      'منتقلیاں سست ہو جائیں گی، کیونکہ کچھ بھی براہ راست آلات کے درمیان نہیں جاتا۔ رابطہ سرور اور ریلے پھر بھی آپ کا IP پتہ دیکھتے ہیں؛ وہ کبھی آپ کی فائلیں یا کلیدیں نہیں دیکھتے۔';

  @override
  String get privacyModeTurnOn => 'آن کریں';

  @override
  String settingsVersion(String version) {
    return 'ورژن $version';
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
      'اس ترجمے کا جائزہ لیا جا رہا ہے۔ Weblate پر اسے بہتر بنانے میں مدد کریں۔';

  @override
  String get sendDropHere => 'فائلیں یہاں چھوڑیں';

  @override
  String get sendDropHint => 'آپ فائلیں اس ونڈو پر گھسیٹ کر بھی لا سکتے ہیں۔';

  @override
  String get sendQrHint => 'یا یہ کوڈ دوسرے آلے کے کیمرے سے اسکین کریں۔';

  @override
  String get sendWaitingAccept =>
      'الفاظ چیک کریں، پھر وصول کنندہ کے قبول کرنے کا انتظار کریں۔';

  @override
  String get receiveFromImage => 'تصویر سے QR پڑھیں';

  @override
  String get receiveQrNotFound => 'اس تصویر میں کوئی TotalDrop کوڈ نہیں ہے۔';

  @override
  String get pickTitle => 'بھیجنے کے لیے فائلیں منتخب کریں';

  @override
  String get pickImageTitle => 'QR کوڈ والی تصویر منتخب کریں';

  @override
  String get openFolder => 'فولڈر کھولیں';

  @override
  String get back => 'واپس';

  @override
  String get codeSuggestions => 'تجاویز';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
