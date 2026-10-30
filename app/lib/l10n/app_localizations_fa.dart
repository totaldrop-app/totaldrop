// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Persian (`fa`).
class AppLocalizationsFa extends AppLocalizations {
  AppLocalizationsFa([String locale = 'fa']) : super(locale);

  @override
  String get appTagline => 'هر فایلی، برای هر کسی، در هر جایی.';

  @override
  String get homeSendTitle => 'ارسال';

  @override
  String get homeSendBody =>
      'فایل‌ها را انتخاب کنید و یک کد کوتاه هم‌رسانی کنید';

  @override
  String get homeReceiveTitle => 'دریافت';

  @override
  String get homeReceiveBody => 'کد را وارد کنید و فایل‌ها را ذخیره کنید';

  @override
  String get homeFooter =>
      'رمزگذاری سرتاسری. بدون حساب، بدون ایمیل، بدون شماره تلفن.';

  @override
  String get settings => 'تنظیمات';

  @override
  String get sendTitle => 'ارسال فایل';

  @override
  String get sendAddFiles => 'افزودن فایل';

  @override
  String get sendEmpty => 'هنوز چیزی انتخاب نشده است';

  @override
  String get sendEmptyHint =>
      'فایل‌هایی را که می‌خواهید بفرستید اضافه کنید. گیرنده دقیقا همین‌ها را دریافت می‌کند، نه چیز دیگری.';

  @override
  String get sendStrongCode => 'کد قوی‌تر';

  @override
  String get sendStrongCodeHint =>
      'سه کلمه به جای دو کلمه: حدس زدن آن بسیار سخت‌تر است';

  @override
  String get sendCreateCode => 'ساخت کد';

  @override
  String get sendCounting => 'در حال شمارش…';

  @override
  String get sendRemove => 'حذف';

  @override
  String get sendPreparing => 'در حال آماده‌سازی فایل‌ها…';

  @override
  String get sendYourCode => 'کد شما';

  @override
  String get sendCodeHint =>
      'در دستگاه دیگر، TotalDrop را باز کنید، روی دریافت بزنید و این کد را وارد کنید.';

  @override
  String get sendWaiting => 'در انتظار گیرنده…';

  @override
  String get sendCopy => 'کپی';

  @override
  String get sendCopied => 'کد کپی شد';

  @override
  String get sendShare => 'هم‌رسانی';

  @override
  String sendShareText(String code) {
    return 'فایل‌های من را با TotalDrop دریافت کن. کد: $code';
  }

  @override
  String get sendSending => 'در حال ارسال';

  @override
  String get sendDone => 'ارسال و تأیید شد';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'هر $countString فایل سالم رسیدند.',
      one: 'فایل سالم رسید.',
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
      other: '$countString فایل سالم نرسیدند و دور ریخته شدند.',
      one: 'یک فایل سالم نرسید و دور ریخته شد.',
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
      other: '$countString مورد',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'دریافت فایل';

  @override
  String get receiveCodeLabel => 'کد';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'کد را از فرستنده بخواهید.';

  @override
  String get receiveCodeInvalid =>
      'این شبیه کد نیست. کد با یک عدد شروع می‌شود و پس از آن دو یا سه کلمه می‌آید.';

  @override
  String get receivePaste => 'جای‌گذاری';

  @override
  String get receiveConnect => 'اتصال';

  @override
  String get receiveConnecting => 'در حال اتصال امن…';

  @override
  String get receiveOffer => 'فایل‌های ورودی';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString فایل',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'و $countString مورد دیگر';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'در $folder ذخیره شد';
  }

  @override
  String get receiveAccept => 'پذیرفتن';

  @override
  String get receiveDecline => 'رد کردن';

  @override
  String get receiveDeclined => 'فایل‌ها را رد کردید.';

  @override
  String get receiveReceiving => 'در حال دریافت';

  @override
  String get receiveDone => 'دریافت و تأیید شد';

  @override
  String get verificationTitle => 'کلمات تأیید';

  @override
  String get verificationHint =>
      'طرف مقابل همین چهار کلمه را می‌بیند. اگر متفاوت بودند، متوقف شوید.';

  @override
  String get sameNetworkNote =>
      'از طریق اینترنت کار می‌کند. وقتی دستگاه‌ها نتوانند مستقیما به هم برسند، انتقال رمزگذاری‌شده از سرور رله می‌گذرد که نمی‌تواند آن را بخواند.';

  @override
  String get done => 'تمام';

  @override
  String get tryAgain => 'تلاش دوباره';

  @override
  String get cancel => 'لغو';

  @override
  String get cancelTransferTitle => 'این انتقال متوقف شود؟';

  @override
  String get cancelTransferBody =>
      'به طرف مقابل اطلاع داده می‌شود که انتقال متوقف شد.';

  @override
  String get keepGoing => 'ادامه';

  @override
  String get stop => 'توقف';

  @override
  String get moreInfo => 'جزئیات';

  @override
  String get errorWrongCode =>
      'کد مطابقت نداشت. یا اشتباه تایپ شده، یا کسی تلاش کرده انتقال را رهگیری کند. چیزی ارسال نشد. با یک کد جدید دوباره شروع کنید.';

  @override
  String get errorPeerLeft => 'طرف مقابل پیش از پایان انتقال خارج شد.';

  @override
  String get errorDeclined => 'گیرنده فایل‌ها را رد کرد.';

  @override
  String get errorCancelled => 'طرف مقابل انتقال را متوقف کرد.';

  @override
  String get errorStopped => 'در این دستگاه متوقف شد.';

  @override
  String get errorInvalidCode => 'این کد معتبر نیست.';

  @override
  String get errorServer =>
      'دسترسی به سرور TotalDrop ممکن نشد. اتصال اینترنت خود را بررسی کنید.';

  @override
  String get errorCodeNotInUse =>
      'این کد در حال استفاده نیست: اشتباه تایپ شده، منقضی شده یا قبلا استفاده شده است. از فرستنده یک کد جدید بخواهید.';

  @override
  String get errorConnect =>
      'اتصال به دستگاه دیگر ممکن نشد. مطمئن شوید هر دو دستگاه آنلاین هستند و دوباره تلاش کنید.';

  @override
  String get errorInvalidOffer =>
      'پیشنهاد یک قاعده امنیتی را نقض کرد، بنابراین رد شد.';

  @override
  String get errorStorage => 'یک فایل خوانده یا ذخیره نشد.';

  @override
  String get errorOther => 'مشکلی پیش آمد.';

  @override
  String get settingsServer => 'سرور هماهنگی';

  @override
  String get settingsServerHelp =>
      'به دو دستگاه با کد یکسان کمک می‌کند یکدیگر را پیدا کنند. هرگز فایل‌ها یا کلیدهای شما را نمی‌بیند.';

  @override
  String get settingsServerReset => 'استفاده از پیش‌فرض';

  @override
  String get settingsServerInvalid => 'باید با wss:// شروع شود';

  @override
  String get settingsSave => 'ذخیره';

  @override
  String get settingsAbout => 'درباره';

  @override
  String get settingsAboutBody =>
      'TotalDrop رایگان و متن‌باز است و با کمک‌های مالی اداره می‌شود. مجوز GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'حریم خصوصی';

  @override
  String get settingsPrivacyBody =>
      'سرور هماهنگی، سرور رله و سرور STUN شرکت Cloudflare نشانی IP شما را می‌بینند، و وقتی مستقیم وصل می‌شوید دستگاه دیگر هم آن را می‌بیند (مگر اینکه حالت حریم خصوصی روشن باشد). هیچ‌کدام نام فایلها، محتوا یا کلیدها را نمی‌بینند و سرورهای ما هیچ گزارشی نگه نمی‌دارند. فایل‌ها هر وقت ممکن باشد مستقیما بین دستگاه‌ها جابجا می‌شوند و در غیر این صورت از سرور رله می‌گذرند؛ در هر دو حالت رمزگذاری سرتاسری دارند.';

  @override
  String get privacyMode => 'پنهان کردن نشانی من از دستگاه دیگر';

  @override
  String get privacyModeHint =>
      'حالت حریم خصوصی: همه چیز از سرور رله می‌گذرد، بنابراین دستگاه دیگر هرگز نشانی IP شما را نمی‌فهمد. کندتر است و دستگاه دیگر باید از همان سرور رله استفاده کند (سرور پیش‌فرض همین است).';

  @override
  String get privacyModeConfirmTitle => 'حالت حریم خصوصی روشن شود؟';

  @override
  String get privacyModeConfirmBody =>
      'انتقال‌ها کندتر می‌شوند، چون هیچ چیز مستقیما بین دستگاه‌ها جابجا نمی‌شود. سرور هماهنگی و سرور رله همچنان نشانی IP شما را می‌بینند؛ ولی هرگز فایل‌ها یا کلیدهای شما را نمی‌بینند.';

  @override
  String get privacyModeTurnOn => 'روشن کردن';

  @override
  String settingsVersion(String version) {
    return 'نسخه $version';
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
      'این ترجمه در حال بازبینی است. در GitHub به بهتر شدن آن کمک کنید.';

  @override
  String get sendDropHere => 'فایل‌ها را اینجا رها کنید';

  @override
  String get sendDropHint => 'می‌توانید فایل‌ها را به این پنجره هم بکشید.';

  @override
  String get sendQrHint => 'یا این کد را با دوربین دستگاه دیگر اسکن کنید.';

  @override
  String get sendWaitingAccept =>
      'کلمات را بررسی کنید، سپس منتظر بمانید تا گیرنده بپذیرد.';

  @override
  String get receiveFromImage => 'خواندن QR از تصویر';

  @override
  String get receiveQrNotFound => 'در این تصویر کد TotalDrop وجود ندارد.';

  @override
  String get pickTitle => 'فایل‌های مورد نظر برای ارسال را انتخاب کنید';

  @override
  String get pickImageTitle => 'تصویری با کد QR انتخاب کنید';

  @override
  String get openFolder => 'باز کردن پوشه';

  @override
  String get back => 'برگشت';

  @override
  String get codeSuggestions => 'پیشنهادها';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString٪';
  }
}
