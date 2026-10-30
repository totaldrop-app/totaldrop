// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Ukrainian (`uk`).
class AppLocalizationsUk extends AppLocalizations {
  AppLocalizationsUk([String locale = 'uk']) : super(locale);

  @override
  String get appTagline => 'Будь-який файл, будь-кому, будь-куди.';

  @override
  String get homeSendTitle => 'Надіслати';

  @override
  String get homeSendBody => 'Виберіть файли й поділіться коротким кодом';

  @override
  String get homeReceiveTitle => 'Отримати';

  @override
  String get homeReceiveBody => 'Введіть код і збережіть файли';

  @override
  String get homeFooter =>
      'Наскрізне шифрування. Без облікового запису, пошти й номера телефону.';

  @override
  String get settings => 'Налаштування';

  @override
  String get sendTitle => 'Надсилання файлів';

  @override
  String get sendAddFiles => 'Додати файли';

  @override
  String get sendEmpty => 'Ще нічого не вибрано';

  @override
  String get sendEmptyHint =>
      'Додайте файли, які хочете надіслати. Отримувач отримає саме їх і нічого більше.';

  @override
  String get sendStrongCode => 'Надійніший код';

  @override
  String get sendStrongCodeHint =>
      'Три слова замість двох: вгадати набагато важче';

  @override
  String get sendCreateCode => 'Створити код';

  @override
  String get sendCounting => 'Підрахунок…';

  @override
  String get sendRemove => 'Прибрати';

  @override
  String get sendPreparing => 'Підготовка файлів…';

  @override
  String get sendYourCode => 'Ваш код';

  @override
  String get sendCodeHint =>
      'На іншому пристрої відкрийте TotalDrop, торкніться «Отримати» і введіть цей код.';

  @override
  String get sendWaiting => 'Очікування отримувача…';

  @override
  String get sendCopy => 'Копіювати';

  @override
  String get sendCopied => 'Код скопійовано';

  @override
  String get sendShare => 'Поділитися';

  @override
  String sendShareText(String code) {
    return 'Отримай мої файли через TotalDrop. Код: $code';
  }

  @override
  String get sendSending => 'Надсилання';

  @override
  String get sendDone => 'Надіслано й перевірено';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Усі $countString файлу доставлено без пошкоджень.',
      many: 'Усі $countString файлів доставлено без пошкоджень.',
      few: 'Усі $countString файли доставлено без пошкоджень.',
      one: '$countString файл доставлено без пошкоджень.',
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
      other: '$countString файлу не дійшли цілими, їх видалено.',
      many: '$countString файлів не дійшли цілими, їх видалено.',
      few: '$countString файли не дійшли цілими, їх видалено.',
      one: '$countString файл не дійшов цілим, його видалено.',
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
      other: '$countString елемента',
      many: '$countString елементів',
      few: '$countString елементи',
      one: '$countString елемент',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Отримання файлів';

  @override
  String get receiveCodeLabel => 'Код';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Попросіть код у відправника.';

  @override
  String get receiveCodeInvalid =>
      'Це не схоже на код. Код починається з числа, за яким ідуть два або три слова.';

  @override
  String get receivePaste => 'Вставити';

  @override
  String get receiveConnect => 'Підключитися';

  @override
  String get receiveConnecting => 'Безпечне підключення…';

  @override
  String get receiveOffer => 'Вхідні файли';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString файлу',
      many: '$filesString файлів',
      few: '$filesString файли',
      one: '$filesString файл',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'і ще $countString';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Збережено в $folder';
  }

  @override
  String get receiveAccept => 'Прийняти';

  @override
  String get receiveDecline => 'Відхилити';

  @override
  String get receiveDeclined => 'Ви відхилили файли.';

  @override
  String get receiveReceiving => 'Отримання';

  @override
  String get receiveDone => 'Отримано й перевірено';

  @override
  String get verificationTitle => 'Слова перевірки';

  @override
  String get verificationHint =>
      'Інша людина бачить ті самі чотири слова. Якщо вони відрізняються, зупиніться.';

  @override
  String get sameNetworkNote =>
      'Працює через інтернет. Якщо пристрої не можуть з\'єднатися напряму, зашифрована передача йде через ретранслятор, який не може її прочитати.';

  @override
  String get done => 'Готово';

  @override
  String get tryAgain => 'Спробувати знову';

  @override
  String get cancel => 'Скасувати';

  @override
  String get cancelTransferTitle => 'Зупинити передачу?';

  @override
  String get cancelTransferBody =>
      'Іншій стороні повідомлять, що передачу зупинено.';

  @override
  String get keepGoing => 'Продовжити';

  @override
  String get stop => 'Зупинити';

  @override
  String get moreInfo => 'Докладніше';

  @override
  String get errorWrongCode =>
      'Код не збігся. Його введено з помилкою, або хтось намагався перехопити передачу. Нічого не надіслано. Почніть знову з новим кодом.';

  @override
  String get errorPeerLeft => 'Інша сторона вийшла до завершення передачі.';

  @override
  String get errorDeclined => 'Отримувач відхилив файли.';

  @override
  String get errorCancelled => 'Інша сторона зупинила передачу.';

  @override
  String get errorStopped => 'Зупинено на цьому пристрої.';

  @override
  String get errorInvalidCode => 'Цей код недійсний.';

  @override
  String get errorServer =>
      'Не вдалося зв\'язатися із сервером TotalDrop. Перевірте підключення до інтернету.';

  @override
  String get errorConnect =>
      'Не вдалося підключитися до іншого пристрою. Переконайтеся, що обидва пристрої в мережі, і спробуйте знову.';

  @override
  String get errorInvalidOffer =>
      'Пропозиція порушила правило безпеки, тому її відхилено.';

  @override
  String get errorStorage => 'Не вдалося прочитати або зберегти файл.';

  @override
  String get errorOther => 'Щось пішло не так.';

  @override
  String get settingsServer => 'Сервер зустрічі';

  @override
  String get settingsServerHelp =>
      'Допомагає двом пристроям з однаковим кодом знайти одне одного. Він ніколи не бачить ваших файлів і ключів.';

  @override
  String get settingsServerReset => 'Використовувати типовий';

  @override
  String get settingsServerInvalid => 'Має починатися з wss://';

  @override
  String get settingsSave => 'Зберегти';

  @override
  String get settingsAbout => 'Про застосунок';

  @override
  String get settingsAboutBody =>
      'TotalDrop — безкоштовний застосунок з відкритим кодом, який існує на пожертви. Ліцензія GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Конфіденційність';

  @override
  String get settingsPrivacyBody =>
      'Сервер зустрічі, ретранслятор і STUN-сервер Cloudflare бачать вашу IP-адресу, як і інший пристрій під час прямого з\'єднання (якщо не ввімкнено режим конфіденційності). Ніхто з них не бачить назв файлів, їхнього вмісту чи ключів, а наші сервери не ведуть журналів. Файли йдуть напряму між пристроями, коли це можливо, а інакше через ретранслятор; в обох випадках вони наскрізно зашифровані.';

  @override
  String get privacyMode => 'Приховувати мою адресу від іншого пристрою';

  @override
  String get privacyModeHint =>
      'Режим конфіденційності: усе йде через ретранслятор, тож інший пристрій ніколи не дізнається вашої IP-адреси. Це повільніше, і інший пристрій має використовувати той самий ретранслятор (типовий підходить).';

  @override
  String get privacyModeConfirmTitle => 'Увімкнути режим конфіденційності?';

  @override
  String get privacyModeConfirmBody =>
      'Передача буде повільнішою, бо нічого не йде напряму між пристроями. Сервер зустрічі й ретранслятор і далі бачать вашу IP-адресу, але ніколи не бачать ваших файлів і ключів.';

  @override
  String get privacyModeTurnOn => 'Увімкнути';

  @override
  String settingsVersion(String version) {
    return 'Версія $version';
  }

  @override
  String sizeBytes(String value) {
    return '$value Б';
  }

  @override
  String sizeKB(String value) {
    return '$value кБ';
  }

  @override
  String sizeMB(String value) {
    return '$value МБ';
  }

  @override
  String sizeGB(String value) {
    return '$value ГБ';
  }

  @override
  String sizeTB(String value) {
    return '$value ТБ';
  }

  @override
  String sizePB(String value) {
    return '$value ПБ';
  }

  @override
  String get draftTranslation =>
      'Цей переклад перевіряється. Допоможіть покращити його на GitHub.';

  @override
  String get sendDropHere => 'Перетягніть файли сюди';

  @override
  String get sendDropHint => 'Файли також можна перетягнути в це вікно.';

  @override
  String get sendQrHint => 'Або відскануйте цей код камерою іншого пристрою.';

  @override
  String get sendWaitingAccept =>
      'Звірте слова й зачекайте, поки отримувач прийме файли.';

  @override
  String get receiveFromImage => 'Зчитати QR-код із зображення';

  @override
  String get receiveQrNotFound => 'На цьому зображенні немає коду TotalDrop.';

  @override
  String get pickTitle => 'Виберіть файли для надсилання';

  @override
  String get pickImageTitle => 'Виберіть зображення з QR-кодом';

  @override
  String get openFolder => 'Відкрити папку';

  @override
  String get back => 'Назад';

  @override
  String get codeSuggestions => 'Підказки';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
