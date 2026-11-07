// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Russian (`ru`).
class AppLocalizationsRu extends AppLocalizations {
  AppLocalizationsRu([String locale = 'ru']) : super(locale);

  @override
  String get appTagline => 'Любой файл, кому угодно, куда угодно.';

  @override
  String get homeSendTitle => 'Отправить';

  @override
  String get homeSendBody => 'Выберите файлы и поделитесь коротким кодом';

  @override
  String get homeReceiveTitle => 'Получить';

  @override
  String get homeReceiveBody => 'Введите код и сохраните файлы';

  @override
  String get homeFooter =>
      'Сквозное шифрование. Без аккаунта, почты и номера телефона.';

  @override
  String get settings => 'Настройки';

  @override
  String get sendTitle => 'Отправка файлов';

  @override
  String get sendAddFiles => 'Добавить файлы';

  @override
  String get sendEmpty => 'Пока ничего не выбрано';

  @override
  String get sendEmptyHint =>
      'Добавьте файлы, которые хотите отправить. Получатель получит только их и ничего больше.';

  @override
  String get sendStrongCode => 'Более надёжный код';

  @override
  String get sendStrongCodeHint =>
      'Три слова вместо двух: угадать намного сложнее';

  @override
  String get sendCreateCode => 'Создать код';

  @override
  String get sendCounting => 'Подсчёт…';

  @override
  String get sendRemove => 'Убрать';

  @override
  String get sendPreparing => 'Подготовка файлов…';

  @override
  String get sendAddFailed =>
      'Не удалось добавить файлы. Попробуйте ещё раз. Если это повторится, проверьте, достаточно ли свободного места на этом устройстве, или выберите файлы в другом месте.';

  @override
  String get sendYourCode => 'Ваш код';

  @override
  String get sendCodeHint =>
      'На другом устройстве откройте TotalDrop, нажмите «Получить» и введите этот код.';

  @override
  String get sendWaiting => 'Ожидание получателя…';

  @override
  String get sendCopy => 'Копировать';

  @override
  String get sendCopied => 'Код скопирован';

  @override
  String get sendShare => 'Поделиться';

  @override
  String sendShareText(String code) {
    return 'Получи мои файлы через TotalDrop. Код: $code';
  }

  @override
  String get sendSending => 'Отправка';

  @override
  String get sendDone => 'Отправлено и проверено';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Все $countString файла доставлены без повреждений.',
      many: 'Все $countString файлов доставлены без повреждений.',
      few: 'Все $countString файла доставлены без повреждений.',
      one: '$countString файл доставлен без повреждений.',
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
      other: '$countString файла не дошли целыми и были удалены.',
      many: '$countString файлов не дошли целыми и были удалены.',
      few: '$countString файла не дошли целыми и были удалены.',
      one: '$countString файл не дошёл целым и был удалён.',
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
      other: '$countString элемента',
      many: '$countString элементов',
      few: '$countString элемента',
      one: '$countString элемент',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Получение файлов';

  @override
  String get receiveCodeLabel => 'Код';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Попросите код у отправителя.';

  @override
  String get receiveCodeInvalid =>
      'Это не похоже на код. Код начинается с числа, за которым идут два или три слова.';

  @override
  String get receivePaste => 'Вставить';

  @override
  String get receiveConnect => 'Подключиться';

  @override
  String get receiveConnecting => 'Безопасное подключение…';

  @override
  String get receiveOffer => 'Входящие файлы';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString файла',
      many: '$filesString файлов',
      few: '$filesString файла',
      one: '$filesString файл',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'и ещё $countString';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Сохранено в $folder';
  }

  @override
  String get receiveAccept => 'Принять';

  @override
  String get receiveDecline => 'Отклонить';

  @override
  String get receiveDeclined => 'Вы отклонили файлы.';

  @override
  String get receiveReceiving => 'Получение';

  @override
  String get receiveDone => 'Получено и проверено';

  @override
  String get verificationTitle => 'Проверочные слова';

  @override
  String get verificationHint =>
      'Другой человек видит те же четыре слова. Если они отличаются, остановитесь.';

  @override
  String get sameNetworkNote =>
      'Работает через интернет. Если устройства не могут связаться напрямую, зашифрованная передача идёт через ретранслятор, который не может её прочитать.';

  @override
  String get done => 'Готово';

  @override
  String get tryAgain => 'Повторить';

  @override
  String get cancel => 'Отмена';

  @override
  String get cancelTransferTitle => 'Остановить передачу?';

  @override
  String get cancelTransferBody =>
      'Другой стороне сообщат, что передача остановлена.';

  @override
  String get keepGoing => 'Продолжить';

  @override
  String get stop => 'Остановить';

  @override
  String get moreInfo => 'Подробнее';

  @override
  String get errorWrongCode =>
      'Код не совпал. Его ввели с ошибкой, или кто-то пытался перехватить передачу. Ничего не отправлено. Начните заново с новым кодом.';

  @override
  String get errorPeerLeft =>
      'Другая сторона отключилась до завершения передачи.';

  @override
  String get errorDeclined => 'Получатель отклонил файлы.';

  @override
  String get errorCancelled => 'Другая сторона остановила передачу.';

  @override
  String get errorStopped => 'Остановлено на этом устройстве.';

  @override
  String get errorInvalidCode => 'Этот код недействителен.';

  @override
  String get errorServer =>
      'Не удалось связаться с сервером TotalDrop. Проверьте подключение к интернету.';

  @override
  String get errorCodeNotInUse =>
      'Этот код не используется: в нём опечатка, срок его действия истёк или он уже был использован. Попросите у отправителя новый.';

  @override
  String get errorCodeExpired =>
      'Срок действия кода истёк, а им так никто и не воспользовался. Отправьте ещё раз, чтобы получить новый код.';

  @override
  String get errorBadServer =>
      'Неверный адрес сервера. Он должен начинаться с wss://.';

  @override
  String get errorConnect =>
      'Не удалось подключиться к другому устройству. Убедитесь, что оба устройства в сети, и повторите попытку.';

  @override
  String get errorInvalidOffer =>
      'Предложение нарушило правило безопасности, поэтому оно отклонено.';

  @override
  String get errorStorage => 'Не удалось прочитать или сохранить файл.';

  @override
  String get errorOther => 'Что-то пошло не так.';

  @override
  String get causeTimedOut =>
      'Соединение перестало отвечать. Возможно, одно из устройств потеряло сеть или ушло в спящий режим.';

  @override
  String get causeConnectionLost =>
      'Соединение с другим устройством прервалось.';

  @override
  String get causeNoRoute =>
      'До другого устройства не удалось достучаться ни напрямую, ни через ретранслятор.';

  @override
  String get causeRelay =>
      'Ретранслятор недоступен или не настроен, либо устройства используют разные ретрансляторы.';

  @override
  String get causeProtocol =>
      'Другое устройство прислало то, что эта версия не понимает. Может помочь обновление TotalDrop на обоих устройствах.';

  @override
  String get causeDiskFull => 'На этом устройстве не хватает свободного места.';

  @override
  String get causeNoPermission =>
      'У TotalDrop нет разрешения читать или записывать нужный файл или папку.';

  @override
  String get causeNotFound =>
      'Файл или папка не найдены. Возможно, их переместили или удалили.';

  @override
  String get causeReadOnly =>
      'Папка находится на диске, на который нельзя записывать.';

  @override
  String get causeFileChanged =>
      'Файл изменился во время отправки. Отправьте его снова после сохранения.';

  @override
  String get technicalDetail =>
      'Техническое сообщение на английском, чтобы приложить его к сообщению о проблеме:';

  @override
  String get settingsServer => 'Сервер встречи';

  @override
  String get settingsServerHelp =>
      'Помогает двум устройствам с одним кодом найти друг друга. Он никогда не видит ваши файлы и ключи.';

  @override
  String get settingsServerReset => 'Использовать стандартный';

  @override
  String get settingsServerInvalid => 'Должен начинаться с wss://';

  @override
  String get settingsSave => 'Сохранить';

  @override
  String get settingsAbout => 'О приложении';

  @override
  String get settingsAboutBody =>
      'TotalDrop — бесплатное приложение с открытым исходным кодом, которое существует на пожертвования. Лицензия GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Конфиденциальность';

  @override
  String get settingsPrivacyBody =>
      'Сервер встречи, ретранслятор и STUN-сервер Cloudflare видят ваш IP-адрес, как и другое устройство при прямом подключении (если не включён режим конфиденциальности). Никто из них не видит имена файлов, их содержимое или ключи, а наши серверы не ведут журналов. Файлы идут напрямую между устройствами, когда это возможно, а иначе через ретранслятор; в обоих случаях они защищены сквозным шифрованием.';

  @override
  String get privacyMode => 'Скрывать мой адрес от другого устройства';

  @override
  String get privacyModeHint =>
      'Режим конфиденциальности: всё идёт через ретранслятор, поэтому другое устройство никогда не узнает ваш IP-адрес. Это медленнее, и другое устройство должно использовать тот же ретранслятор (стандартный подходит).';

  @override
  String get privacyModeConfirmTitle => 'Включить режим конфиденциальности?';

  @override
  String get privacyModeConfirmBody =>
      'Передача будет медленнее, потому что ничего не идёт напрямую между устройствами. Сервер встречи и ретранслятор по-прежнему видят ваш IP-адрес, но никогда не видят ваши файлы и ключи.';

  @override
  String get privacyModeTurnOn => 'Включить';

  @override
  String settingsVersion(String version) {
    return 'Версия $version';
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
      'Этот перевод проверяется. Помогите улучшить его на GitHub.';

  @override
  String get settingsWordsEnglish =>
      'Коды и проверочные слова всегда на английском, чтобы два устройства с разными языками показывали одинаковые слова.';

  @override
  String get sendDropHere => 'Перетащите файлы сюда';

  @override
  String get sendDropHint => 'Файлы также можно перетащить в это окно.';

  @override
  String get sendQrHint =>
      'Или отсканируйте этот код камерой другого устройства.';

  @override
  String get sendWaitingAccept =>
      'Сверьте слова и дождитесь, пока получатель примет файлы.';

  @override
  String get receiveFromImage => 'Считать QR-код с изображения';

  @override
  String get receiveQrNotFound => 'На этом изображении нет кода TotalDrop.';

  @override
  String get pickTitle => 'Выберите файлы для отправки';

  @override
  String get pickImageTitle => 'Выберите изображение с QR-кодом';

  @override
  String get openFolder => 'Открыть папку';

  @override
  String get back => 'Назад';

  @override
  String get codeSuggestions => 'Подсказки';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }

  @override
  String get editUndo => 'Отменить';

  @override
  String get editRedo => 'Повторить';

  @override
  String get editCut => 'Вырезать';

  @override
  String get editCopy => 'Копировать';

  @override
  String get editPaste => 'Вставить';

  @override
  String get editDelete => 'Удалить';

  @override
  String get editSelectAll => 'Выделить всё';

  @override
  String get transferNoticeChannel => 'Передачи';

  @override
  String get transferNoticeSending => 'Отправка через TotalDrop';

  @override
  String get transferNoticeReceiving => 'Получение через TotalDrop';

  @override
  String get transferNoticeBody =>
      'Продолжается, пока вы пользуетесь другими приложениями.';

  @override
  String get shareTextSoon =>
      'TotalDrop пока не отправляет текст, только файлы. Поделитесь файлом.';

  @override
  String get shareBusy =>
      'Передача ещё идёт. Дождитесь её окончания или остановите её, затем поделитесь снова.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count переданного элемента не удалось прочитать, они пропущены.',
      many: '$count переданных элементов не удалось прочитать, они пропущены.',
      few: '$count переданных элемента не удалось прочитать, они пропущены.',
      one: '$count переданный элемент не удалось прочитать, он пропущен.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'Передача файлов';

  @override
  String get desktopComment =>
      'Отправляйте файлы прямо на другое устройство со сквозным шифрованием';

  @override
  String get desktopKeywords => 'отправить;получить;поделиться;передать;файлы;';

  @override
  String get desktopSendWith => 'Отправить через TotalDrop';

  @override
  String get metainfoSummary => 'Отправляйте файлы прямо на другое устройство';

  @override
  String get metainfoAbout =>
      'TotalDrop передаёт файлы и папки с одного устройства на другое со сквозным шифрованием. Выберите файлы, поделитесь коротким кодом или QR-кодом, и другое устройство их получит. Без учётной записи, почты и номера телефона.';

  @override
  String get metainfoVerified =>
      'Каждый файл проверяется при получении; повреждённые файлы отбрасываются';

  @override
  String get metainfoWords =>
      'Четыре проверочных слова показывают, что посередине никого нет';

  @override
  String get metainfoTheme =>
      'Следует теме вашего рабочего стола: Plasma, Hyprland, Omarchy, GNOME';
}
