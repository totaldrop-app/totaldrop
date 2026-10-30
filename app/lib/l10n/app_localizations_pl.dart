// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Polish (`pl`).
class AppLocalizationsPl extends AppLocalizations {
  AppLocalizationsPl([String locale = 'pl']) : super(locale);

  @override
  String get appTagline => 'Każdy plik, do każdego, wszędzie.';

  @override
  String get homeSendTitle => 'Wyślij';

  @override
  String get homeSendBody => 'Wybierz pliki i przekaż krótki kod';

  @override
  String get homeReceiveTitle => 'Odbierz';

  @override
  String get homeReceiveBody => 'Wpisz kod i zapisz pliki';

  @override
  String get homeFooter =>
      'Szyfrowanie end-to-end. Bez konta, e-maila i numeru telefonu.';

  @override
  String get settings => 'Ustawienia';

  @override
  String get sendTitle => 'Wysyłanie plików';

  @override
  String get sendAddFiles => 'Dodaj pliki';

  @override
  String get sendEmpty => 'Nic jeszcze nie wybrano';

  @override
  String get sendEmptyHint =>
      'Dodaj pliki, które chcesz wysłać. Odbiorca dostanie dokładnie je i nic więcej.';

  @override
  String get sendStrongCode => 'Silniejszy kod';

  @override
  String get sendStrongCodeHint =>
      'Trzy słowa zamiast dwóch: znacznie trudniej zgadnąć';

  @override
  String get sendCreateCode => 'Utwórz kod';

  @override
  String get sendCounting => 'Liczenie…';

  @override
  String get sendRemove => 'Usuń';

  @override
  String get sendPreparing => 'Przygotowywanie plików…';

  @override
  String get sendYourCode => 'Twój kod';

  @override
  String get sendCodeHint =>
      'Na drugim urządzeniu otwórz TotalDrop, stuknij Odbierz i wpisz ten kod.';

  @override
  String get sendWaiting => 'Czekanie na odbiorcę…';

  @override
  String get sendCopy => 'Kopiuj';

  @override
  String get sendCopied => 'Skopiowano kod';

  @override
  String get sendShare => 'Udostępnij';

  @override
  String sendShareText(String code) {
    return 'Odbierz moje pliki w TotalDrop. Kod: $code';
  }

  @override
  String get sendSending => 'Wysyłanie';

  @override
  String get sendDone => 'Wysłano i zweryfikowano';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Wszystkie $countString pliku dotarło w całości.',
      many: 'Wszystkie $countString plików dotarło w całości.',
      few: 'Wszystkie $countString pliki dotarły w całości.',
      one: 'Plik dotarł w całości.',
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
      other: '$countString pliku nie dotarło w całości i zostało odrzuconych.',
      many: '$countString plików nie dotarło w całości i zostało odrzuconych.',
      few: '$countString pliki nie dotarły w całości i zostały odrzucone.',
      one: 'Jeden plik nie dotarł w całości i został odrzucony.',
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
      other: '$countString elementu',
      many: '$countString elementów',
      few: '$countString elementy',
      one: '$countString element',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Odbieranie plików';

  @override
  String get receiveCodeLabel => 'Kod';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Poproś nadawcę o kod.';

  @override
  String get receiveCodeInvalid =>
      'To nie wygląda na kod. Kod zaczyna się od liczby, po której są dwa lub trzy słowa.';

  @override
  String get receivePaste => 'Wklej';

  @override
  String get receiveConnect => 'Połącz';

  @override
  String get receiveConnecting => 'Bezpieczne łączenie…';

  @override
  String get receiveOffer => 'Przychodzące pliki';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString pliku',
      many: '$filesString plików',
      few: '$filesString pliki',
      one: '$filesString plik',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'i $countString więcej';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Zapisano w $folder';
  }

  @override
  String get receiveAccept => 'Akceptuj';

  @override
  String get receiveDecline => 'Odrzuć';

  @override
  String get receiveDeclined => 'Odrzucono pliki.';

  @override
  String get receiveReceiving => 'Odbieranie';

  @override
  String get receiveDone => 'Odebrano i zweryfikowano';

  @override
  String get verificationTitle => 'Słowa weryfikacyjne';

  @override
  String get verificationHint =>
      'Druga osoba widzi te same cztery słowa. Jeśli się różnią, przerwij.';

  @override
  String get sameNetworkNote =>
      'Działa przez internet. Gdy urządzenia nie mogą połączyć się bezpośrednio, zaszyfrowany transfer przechodzi przez serwer pośredniczący, który nie może go odczytać.';

  @override
  String get done => 'Gotowe';

  @override
  String get tryAgain => 'Spróbuj ponownie';

  @override
  String get cancel => 'Anuluj';

  @override
  String get cancelTransferTitle => 'Zatrzymać ten transfer?';

  @override
  String get cancelTransferBody =>
      'Druga strona zostanie powiadomiona, że transfer został zatrzymany.';

  @override
  String get keepGoing => 'Kontynuuj';

  @override
  String get stop => 'Zatrzymaj';

  @override
  String get moreInfo => 'Szczegóły';

  @override
  String get errorWrongCode =>
      'Kod się nie zgadzał. Został źle wpisany albo ktoś próbował przechwycić transfer. Nic nie zostało wysłane. Zacznij od nowa z nowym kodem.';

  @override
  String get errorPeerLeft =>
      'Druga strona rozłączyła się przed zakończeniem transferu.';

  @override
  String get errorDeclined => 'Odbiorca odrzucił pliki.';

  @override
  String get errorCancelled => 'Druga strona zatrzymała transfer.';

  @override
  String get errorStopped => 'Zatrzymano na tym urządzeniu.';

  @override
  String get errorInvalidCode => 'Ten kod jest nieprawidłowy.';

  @override
  String get errorServer =>
      'Nie można połączyć się z serwerem TotalDrop. Sprawdź połączenie z internetem.';

  @override
  String get errorConnect =>
      'Nie można połączyć się z drugim urządzeniem. Sprawdź, czy oba urządzenia są online, i spróbuj ponownie.';

  @override
  String get errorInvalidOffer =>
      'Oferta naruszyła regułę bezpieczeństwa, więc została odrzucona.';

  @override
  String get errorStorage => 'Nie można odczytać lub zapisać pliku.';

  @override
  String get errorOther => 'Coś poszło nie tak.';

  @override
  String get settingsServer => 'Serwer spotkań';

  @override
  String get settingsServerHelp =>
      'Pomaga dwóm urządzeniom z tym samym kodem odnaleźć się nawzajem. Nigdy nie widzi twoich plików ani kluczy.';

  @override
  String get settingsServerReset => 'Użyj domyślnego';

  @override
  String get settingsServerInvalid => 'Musi zaczynać się od wss://';

  @override
  String get settingsSave => 'Zapisz';

  @override
  String get settingsAbout => 'O aplikacji';

  @override
  String get settingsAboutBody =>
      'TotalDrop jest darmowy i ma otwarte źródła, a utrzymuje się z darowizn. Licencja GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Prywatność';

  @override
  String get settingsPrivacyBody =>
      'Serwer spotkań, serwer pośredniczący i serwer STUN Cloudflare widzą twój adres IP, podobnie jak drugie urządzenie przy połączeniu bezpośrednim (chyba że włączony jest tryb prywatności). Żaden z nich nie widzi nazw plików, ich zawartości ani kluczy, a nasze serwery nie prowadzą logów. Pliki trafiają bezpośrednio między urządzeniami, gdy to możliwe, a w przeciwnym razie przez serwer pośredniczący; w obu przypadkach są szyfrowane end-to-end.';

  @override
  String get privacyMode => 'Ukryj mój adres przed drugim urządzeniem';

  @override
  String get privacyModeHint =>
      'Tryb prywatności: wszystko przechodzi przez serwer pośredniczący, więc drugie urządzenie nigdy nie pozna twojego adresu IP. Działa wolniej, a drugie urządzenie musi używać tego samego serwera pośredniczącego (domyślny to zapewnia).';

  @override
  String get privacyModeConfirmTitle => 'Włączyć tryb prywatności?';

  @override
  String get privacyModeConfirmBody =>
      'Transfery będą wolniejsze, bo nic nie przechodzi bezpośrednio między urządzeniami. Serwer spotkań i serwer pośredniczący nadal widzą twój adres IP; nigdy nie widzą twoich plików ani kluczy.';

  @override
  String get privacyModeTurnOn => 'Włącz';

  @override
  String settingsVersion(String version) {
    return 'Wersja $version';
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
      'To tłumaczenie jest w trakcie sprawdzania. Pomóż je ulepszyć na GitHub.';

  @override
  String get sendDropHere => 'Upuść pliki tutaj';

  @override
  String get sendDropHint => 'Możesz też przeciągnąć pliki na to okno.';

  @override
  String get sendQrHint =>
      'Albo zeskanuj ten kod aparatem drugiego urządzenia.';

  @override
  String get sendWaitingAccept =>
      'Sprawdź słowa, a potem poczekaj, aż odbiorca zaakceptuje.';

  @override
  String get receiveFromImage => 'Odczytaj kod QR z obrazu';

  @override
  String get receiveQrNotFound => 'Na tym obrazie nie ma kodu TotalDrop.';

  @override
  String get pickTitle => 'Wybierz pliki do wysłania';

  @override
  String get pickImageTitle => 'Wybierz obraz z kodem QR';

  @override
  String get openFolder => 'Otwórz folder';

  @override
  String get back => 'Wstecz';

  @override
  String get codeSuggestions => 'Sugestie';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
