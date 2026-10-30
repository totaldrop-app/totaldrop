// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Dutch Flemish (`nl`).
class AppLocalizationsNl extends AppLocalizations {
  AppLocalizationsNl([String locale = 'nl']) : super(locale);

  @override
  String get appTagline => 'Elk bestand, naar iedereen, overal.';

  @override
  String get homeSendTitle => 'Versturen';

  @override
  String get homeSendBody => 'Kies bestanden en deel een korte code';

  @override
  String get homeReceiveTitle => 'Ontvangen';

  @override
  String get homeReceiveBody => 'Voer een code in en sla de bestanden op';

  @override
  String get homeFooter =>
      'End-to-end versleuteld. Geen account, geen e-mail, geen telefoonnummer.';

  @override
  String get settings => 'Instellingen';

  @override
  String get sendTitle => 'Bestanden versturen';

  @override
  String get sendAddFiles => 'Bestanden toevoegen';

  @override
  String get sendEmpty => 'Nog niets geselecteerd';

  @override
  String get sendEmptyHint =>
      'Voeg de bestanden toe die je wilt versturen. De ontvanger krijgt precies deze, verder niets.';

  @override
  String get sendStrongCode => 'Sterkere code';

  @override
  String get sendStrongCodeHint =>
      'Drie woorden in plaats van twee: veel moeilijker te raden';

  @override
  String get sendCreateCode => 'Code maken';

  @override
  String get sendCounting => 'Tellen…';

  @override
  String get sendRemove => 'Verwijderen';

  @override
  String get sendPreparing => 'Je bestanden worden voorbereid…';

  @override
  String get sendYourCode => 'Je code';

  @override
  String get sendCodeHint =>
      'Open TotalDrop op het andere apparaat, tik op Ontvangen en voer deze code in.';

  @override
  String get sendWaiting => 'Wachten op de ontvanger…';

  @override
  String get sendCopy => 'Kopiëren';

  @override
  String get sendCopied => 'Code gekopieerd';

  @override
  String get sendShare => 'Delen';

  @override
  String sendShareText(String code) {
    return 'Ontvang mijn bestanden met TotalDrop. Code: $code';
  }

  @override
  String get sendSending => 'Versturen';

  @override
  String get sendDone => 'Verstuurd en gecontroleerd';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Alle $countString bestanden zijn ongeschonden aangekomen.',
      one: 'Het bestand is ongeschonden aangekomen.',
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
          '$countString bestanden zijn niet ongeschonden aangekomen en zijn verwijderd.',
      one: 'Eén bestand is niet ongeschonden aangekomen en is verwijderd.',
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
      other: '$countString items',
      one: '$countString item',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Bestanden ontvangen';

  @override
  String get receiveCodeLabel => 'Code';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Vraag de afzender om de code.';

  @override
  String get receiveCodeInvalid =>
      'Dat lijkt geen code. Een code begint met een getal, gevolgd door twee of drie woorden.';

  @override
  String get receivePaste => 'Plakken';

  @override
  String get receiveConnect => 'Verbinden';

  @override
  String get receiveConnecting => 'Veilige verbinding maken…';

  @override
  String get receiveOffer => 'Inkomende bestanden';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString bestanden',
      one: '$filesString bestand',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'en nog $countString';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Opgeslagen in $folder';
  }

  @override
  String get receiveAccept => 'Accepteren';

  @override
  String get receiveDecline => 'Weigeren';

  @override
  String get receiveDeclined => 'Je hebt de bestanden geweigerd.';

  @override
  String get receiveReceiving => 'Ontvangen';

  @override
  String get receiveDone => 'Ontvangen en gecontroleerd';

  @override
  String get verificationTitle => 'Verificatiewoorden';

  @override
  String get verificationHint =>
      'De ander ziet dezelfde vier woorden. Als ze verschillen, stop dan.';

  @override
  String get sameNetworkNote =>
      'Werkt via het internet. Als de apparaten elkaar niet rechtstreeks kunnen bereiken, gaat de versleutelde overdracht via de relayserver, die haar niet kan lezen.';

  @override
  String get done => 'Klaar';

  @override
  String get tryAgain => 'Opnieuw proberen';

  @override
  String get cancel => 'Annuleren';

  @override
  String get cancelTransferTitle => 'Deze overdracht stoppen?';

  @override
  String get cancelTransferBody =>
      'De andere kant krijgt te horen dat de overdracht is gestopt.';

  @override
  String get keepGoing => 'Doorgaan';

  @override
  String get stop => 'Stoppen';

  @override
  String get moreInfo => 'Details';

  @override
  String get errorWrongCode =>
      'De code kwam niet overeen. Hij is verkeerd getypt, of iemand heeft geprobeerd de overdracht te onderscheppen. Er is niets verstuurd. Begin opnieuw met een nieuwe code.';

  @override
  String get errorPeerLeft =>
      'De andere kant is vertrokken voordat de overdracht klaar was.';

  @override
  String get errorDeclined => 'De ontvanger heeft de bestanden geweigerd.';

  @override
  String get errorCancelled => 'De andere kant heeft de overdracht gestopt.';

  @override
  String get errorStopped => 'Gestopt op dit apparaat.';

  @override
  String get errorInvalidCode => 'Die code is niet geldig.';

  @override
  String get errorServer =>
      'Kan de TotalDrop-server niet bereiken. Controleer je internetverbinding.';

  @override
  String get errorConnect =>
      'Kan geen verbinding maken met het andere apparaat. Controleer of beide apparaten online zijn en probeer het opnieuw.';

  @override
  String get errorInvalidOffer =>
      'Het aanbod overtrad een veiligheidsregel en is daarom geweigerd.';

  @override
  String get errorStorage =>
      'Een bestand kon niet worden gelezen of opgeslagen.';

  @override
  String get errorOther => 'Er is iets misgegaan.';

  @override
  String get settingsServer => 'Ontmoetingsserver';

  @override
  String get settingsServerHelp =>
      'Helpt twee apparaten met dezelfde code elkaar te vinden. Hij ziet nooit je bestanden of sleutels.';

  @override
  String get settingsServerReset => 'Standaard gebruiken';

  @override
  String get settingsServerInvalid => 'Moet beginnen met wss://';

  @override
  String get settingsSave => 'Opslaan';

  @override
  String get settingsAbout => 'Over';

  @override
  String get settingsAboutBody =>
      'TotalDrop is gratis en open source, gefinancierd met donaties. Licentie: GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Privacy';

  @override
  String get settingsPrivacyBody =>
      'De ontmoetingsserver, de relayserver en de STUN-server van Cloudflare zien je IP-adres, en het andere apparaat ook bij een rechtstreekse verbinding (tenzij de privacymodus aan staat). Geen van hen ziet bestandsnamen, inhoud of sleutels, en onze servers houden geen logboeken bij. Bestanden gaan waar mogelijk rechtstreeks tussen de apparaten, anders via de relayserver; in beide gevallen zijn ze end-to-end versleuteld.';

  @override
  String get privacyMode => 'Mijn adres verbergen voor het andere apparaat';

  @override
  String get privacyModeHint =>
      'Privacymodus: alles gaat via de relayserver, zodat het andere apparaat je IP-adres nooit te weten komt. Langzamer, en het andere apparaat moet dezelfde relayserver gebruiken (de standaardserver doet dat).';

  @override
  String get privacyModeConfirmTitle => 'Privacymodus aanzetten?';

  @override
  String get privacyModeConfirmBody =>
      'Overdrachten worden langzamer, omdat niets rechtstreeks tussen de apparaten gaat. De ontmoetingsserver en de relayserver zien je IP-adres nog steeds; je bestanden of sleutels zien ze nooit.';

  @override
  String get privacyModeTurnOn => 'Aanzetten';

  @override
  String settingsVersion(String version) {
    return 'Versie $version';
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
      'Deze vertaling wordt nagekeken. Help haar te verbeteren op GitHub.';

  @override
  String get sendDropHere => 'Zet bestanden hier neer';

  @override
  String get sendDropHint => 'Je kunt bestanden ook naar dit venster slepen.';

  @override
  String get sendQrHint =>
      'Of scan deze code met de camera van het andere apparaat.';

  @override
  String get sendWaitingAccept =>
      'Controleer de woorden en wacht tot de ontvanger accepteert.';

  @override
  String get receiveFromImage => 'QR-code uit afbeelding lezen';

  @override
  String get receiveQrNotFound =>
      'Er staat geen TotalDrop-code in die afbeelding.';

  @override
  String get pickTitle => 'Kies bestanden om te versturen';

  @override
  String get pickImageTitle => 'Kies een afbeelding met een QR-code';

  @override
  String get openFolder => 'Map openen';

  @override
  String get back => 'Terug';

  @override
  String get codeSuggestions => 'Suggesties';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
