// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for German (`de`).
class AppLocalizationsDe extends AppLocalizations {
  AppLocalizationsDe([String locale = 'de']) : super(locale);

  @override
  String get appTagline => 'Jede Datei, an jeden, überallhin.';

  @override
  String get homeSendTitle => 'Senden';

  @override
  String get homeSendBody => 'Dateien auswählen und einen kurzen Code teilen';

  @override
  String get homeReceiveTitle => 'Empfangen';

  @override
  String get homeReceiveBody => 'Code eingeben und Dateien speichern';

  @override
  String get homeFooter =>
      'Ende-zu-Ende-verschlüsselt. Kein Konto, keine E-Mail, keine Telefonnummer.';

  @override
  String get settings => 'Einstellungen';

  @override
  String get sendTitle => 'Dateien senden';

  @override
  String get sendAddFiles => 'Dateien hinzufügen';

  @override
  String get sendEmpty => 'Noch nichts ausgewählt';

  @override
  String get sendEmptyHint =>
      'Fügen Sie die Dateien hinzu, die Sie senden möchten. Der Empfänger erhält genau diese, sonst nichts.';

  @override
  String get sendStrongCode => 'Stärkerer Code';

  @override
  String get sendStrongCodeHint =>
      'Drei Wörter statt zwei: viel schwerer zu erraten';

  @override
  String get sendCreateCode => 'Code erstellen';

  @override
  String get sendCounting => 'Wird gezählt…';

  @override
  String get sendRemove => 'Entfernen';

  @override
  String get sendPreparing => 'Dateien werden vorbereitet…';

  @override
  String get sendYourCode => 'Ihr Code';

  @override
  String get sendCodeHint =>
      'Öffnen Sie auf dem anderen Gerät TotalDrop, tippen Sie auf Empfangen und geben Sie diesen Code ein.';

  @override
  String get sendWaiting => 'Warten auf den Empfänger…';

  @override
  String get sendCopy => 'Kopieren';

  @override
  String get sendCopied => 'Code kopiert';

  @override
  String get sendShare => 'Teilen';

  @override
  String sendShareText(String code) {
    return 'Meine Dateien mit TotalDrop empfangen. Code: $code';
  }

  @override
  String get sendSending => 'Wird gesendet';

  @override
  String get sendDone => 'Gesendet und überprüft';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Alle $countString Dateien sind unversehrt angekommen.',
      one: 'Die Datei ist unversehrt angekommen.',
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
          '$countString Dateien sind nicht unversehrt angekommen und wurden verworfen.',
      one: 'Eine Datei ist nicht unversehrt angekommen und wurde verworfen.',
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
      other: '$countString Elemente',
      one: '$countString Element',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Dateien empfangen';

  @override
  String get receiveCodeLabel => 'Code';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Bitten Sie den Absender um den Code.';

  @override
  String get receiveCodeInvalid =>
      'Das sieht nicht wie ein Code aus. Er beginnt mit einer Zahl, danach folgen zwei oder drei Wörter.';

  @override
  String get receivePaste => 'Einfügen';

  @override
  String get receiveConnect => 'Verbinden';

  @override
  String get receiveConnecting => 'Sichere Verbindung wird hergestellt…';

  @override
  String get receiveOffer => 'Eingehende Dateien';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString Dateien',
      one: '$filesString Datei',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'und $countString weitere';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Gespeichert in $folder';
  }

  @override
  String get receiveAccept => 'Annehmen';

  @override
  String get receiveDecline => 'Ablehnen';

  @override
  String get receiveDeclined => 'Sie haben die Dateien abgelehnt.';

  @override
  String get receiveReceiving => 'Wird empfangen';

  @override
  String get receiveDone => 'Empfangen und überprüft';

  @override
  String get verificationTitle => 'Prüfwörter';

  @override
  String get verificationHint =>
      'Die andere Person sieht dieselben vier Wörter. Wenn sie abweichen, brechen Sie ab.';

  @override
  String get sameNetworkNote =>
      'Funktioniert über das Internet. Wenn die Geräte einander nicht direkt erreichen, läuft die verschlüsselte Übertragung über den Relay-Server, der sie nicht lesen kann.';

  @override
  String get done => 'Fertig';

  @override
  String get tryAgain => 'Erneut versuchen';

  @override
  String get cancel => 'Abbrechen';

  @override
  String get cancelTransferTitle => 'Übertragung beenden?';

  @override
  String get cancelTransferBody =>
      'Die Gegenseite erfährt, dass die Übertragung beendet wurde.';

  @override
  String get keepGoing => 'Fortsetzen';

  @override
  String get stop => 'Beenden';

  @override
  String get moreInfo => 'Details';

  @override
  String get errorWrongCode =>
      'Der Code stimmte nicht überein. Er wurde falsch eingegeben, oder jemand hat versucht, die Übertragung abzufangen. Es wurde nichts gesendet. Beginnen Sie neu mit einem neuen Code.';

  @override
  String get errorPeerLeft =>
      'Die Gegenseite hat die Verbindung getrennt, bevor die Übertragung abgeschlossen war.';

  @override
  String get errorDeclined => 'Der Empfänger hat die Dateien abgelehnt.';

  @override
  String get errorCancelled => 'Die Gegenseite hat die Übertragung beendet.';

  @override
  String get errorStopped => 'Auf diesem Gerät beendet.';

  @override
  String get errorInvalidCode => 'Dieser Code ist ungültig.';

  @override
  String get errorServer =>
      'Der TotalDrop-Server ist nicht erreichbar. Prüfen Sie Ihre Internetverbindung.';

  @override
  String get errorConnect =>
      'Keine Verbindung zum anderen Gerät möglich. Prüfen Sie, ob beide Geräte online sind, und versuchen Sie es erneut.';

  @override
  String get errorInvalidOffer =>
      'Das Angebot hat gegen eine Sicherheitsregel verstoßen und wurde deshalb abgelehnt.';

  @override
  String get errorStorage =>
      'Eine Datei konnte nicht gelesen oder gespeichert werden.';

  @override
  String get errorOther => 'Etwas ist schiefgelaufen.';

  @override
  String get settingsServer => 'Vermittlungsserver';

  @override
  String get settingsServerHelp =>
      'Hilft zwei Geräten mit demselben Code, einander zu finden. Er sieht nie Ihre Dateien oder Schlüssel.';

  @override
  String get settingsServerReset => 'Standard verwenden';

  @override
  String get settingsServerInvalid => 'Muss mit wss:// beginnen';

  @override
  String get settingsSave => 'Speichern';

  @override
  String get settingsAbout => 'Über';

  @override
  String get settingsAboutBody =>
      'TotalDrop ist kostenlos und quelloffen und wird durch Spenden finanziert. Lizenz: GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Datenschutz';

  @override
  String get settingsPrivacyBody =>
      'Der Vermittlungsserver, der Relay-Server und der STUN-Server von Cloudflare sehen Ihre IP-Adresse, bei einer direkten Verbindung auch das andere Gerät (außer im Privatsphäre-Modus). Keiner von ihnen sieht Dateinamen, Inhalte oder Schlüssel, und unsere Server speichern keine Protokolle. Dateien gehen nach Möglichkeit direkt zwischen den Geräten, sonst über den Relay-Server; in beiden Fällen sind sie Ende-zu-Ende-verschlüsselt.';

  @override
  String get privacyMode => 'Meine Adresse vor dem anderen Gerät verbergen';

  @override
  String get privacyModeHint =>
      'Privatsphäre-Modus: Alles läuft über den Relay-Server, sodass das andere Gerät Ihre IP-Adresse nie erfährt. Langsamer, und das andere Gerät muss denselben Relay-Server verwenden (beim Standardserver ist das so).';

  @override
  String get privacyModeConfirmTitle => 'Privatsphäre-Modus aktivieren?';

  @override
  String get privacyModeConfirmBody =>
      'Übertragungen werden langsamer, weil nichts direkt zwischen den Geräten läuft. Der Vermittlungsserver und der Relay-Server sehen weiterhin Ihre IP-Adresse, aber nie Ihre Dateien oder Schlüssel.';

  @override
  String get privacyModeTurnOn => 'Aktivieren';

  @override
  String settingsVersion(String version) {
    return 'Version $version';
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
      'Diese Übersetzung wird noch geprüft. Helfen Sie auf GitHub, sie zu verbessern.';

  @override
  String get sendDropHere => 'Dateien hier ablegen';

  @override
  String get sendDropHint =>
      'Sie können Dateien auch auf dieses Fenster ziehen.';

  @override
  String get sendQrHint =>
      'Oder scannen Sie diesen Code mit der Kamera des anderen Geräts.';

  @override
  String get sendWaitingAccept =>
      'Prüfen Sie die Wörter und warten Sie, bis der Empfänger annimmt.';

  @override
  String get receiveFromImage => 'QR-Code aus Bild lesen';

  @override
  String get receiveQrNotFound => 'In diesem Bild ist kein TotalDrop-Code.';

  @override
  String get pickTitle => 'Dateien zum Senden auswählen';

  @override
  String get pickImageTitle => 'Bild mit QR-Code auswählen';

  @override
  String get openFolder => 'Ordner öffnen';

  @override
  String get back => 'Zurück';

  @override
  String get codeSuggestions => 'Vorschläge';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString %';
  }
}
