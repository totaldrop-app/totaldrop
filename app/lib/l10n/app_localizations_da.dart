// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Danish (`da`).
class AppLocalizationsDa extends AppLocalizations {
  AppLocalizationsDa([String locale = 'da']) : super(locale);

  @override
  String get appTagline => 'Enhver fil, til alle, overalt.';

  @override
  String get homeSendTitle => 'Send';

  @override
  String get homeSendBody => 'Vælg filer, og del en kort kode';

  @override
  String get homeReceiveTitle => 'Modtag';

  @override
  String get homeReceiveBody => 'Indtast en kode, og gem filerne';

  @override
  String get homeFooter =>
      'End-to-end-krypteret. Ingen konto, ingen e-mail, intet telefonnummer.';

  @override
  String get settings => 'Indstillinger';

  @override
  String get sendTitle => 'Send filer';

  @override
  String get sendAddFiles => 'Tilføj filer';

  @override
  String get sendEmpty => 'Intet valgt endnu';

  @override
  String get sendEmptyHint =>
      'Tilføj de filer, du vil sende. Modtageren får præcis dem og intet andet.';

  @override
  String get sendStrongCode => 'Stærkere kode';

  @override
  String get sendStrongCodeHint =>
      'Tre ord i stedet for to: meget sværere at gætte';

  @override
  String get sendCreateCode => 'Opret kode';

  @override
  String get sendCounting => 'Tæller…';

  @override
  String get sendRemove => 'Fjern';

  @override
  String get sendPreparing => 'Forbereder dine filer…';

  @override
  String get sendYourCode => 'Din kode';

  @override
  String get sendCodeHint =>
      'Åbn TotalDrop på den anden enhed, tryk på Modtag, og indtast denne kode.';

  @override
  String get sendWaiting => 'Venter på modtageren…';

  @override
  String get sendCopy => 'Kopiér';

  @override
  String get sendCopied => 'Koden er kopieret';

  @override
  String get sendShare => 'Del';

  @override
  String sendShareText(String code) {
    return 'Hent mine filer med TotalDrop. Kode: $code';
  }

  @override
  String get sendSending => 'Sender';

  @override
  String get sendDone => 'Sendt og bekræftet';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Alle $countString filer kom frem i god behold.',
      one: 'Filen kom frem i god behold.',
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
      other: '$countString filer kom ikke frem i god behold og blev kasseret.',
      one: 'Én fil kom ikke frem i god behold og blev kasseret.',
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
      other: '$countString elementer',
      one: '$countString element',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Modtag filer';

  @override
  String get receiveCodeLabel => 'Kode';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Bed afsenderen om koden.';

  @override
  String get receiveCodeInvalid =>
      'Det ligner ikke en kode. Den starter med et tal efterfulgt af to eller tre ord.';

  @override
  String get receivePaste => 'Indsæt';

  @override
  String get receiveConnect => 'Opret forbindelse';

  @override
  String get receiveConnecting => 'Opretter sikker forbindelse…';

  @override
  String get receiveOffer => 'Indgående filer';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString filer',
      one: '$filesString fil',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'og $countString mere';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Gemmes i $folder';
  }

  @override
  String get receiveAccept => 'Acceptér';

  @override
  String get receiveDecline => 'Afvis';

  @override
  String get receiveDeclined => 'Du afviste filerne.';

  @override
  String get receiveReceiving => 'Modtager';

  @override
  String get receiveDone => 'Modtaget og bekræftet';

  @override
  String get verificationTitle => 'Bekræftelsesord';

  @override
  String get verificationHint =>
      'Den anden person ser de samme fire ord. Hvis de er forskellige, så stop.';

  @override
  String get sameNetworkNote =>
      'Virker over internettet. Når enhederne ikke kan nå hinanden direkte, går den krypterede overførsel gennem relæserveren, som ikke kan læse den.';

  @override
  String get done => 'Færdig';

  @override
  String get tryAgain => 'Prøv igen';

  @override
  String get cancel => 'Annuller';

  @override
  String get cancelTransferTitle => 'Stop overførslen?';

  @override
  String get cancelTransferBody =>
      'Den anden side får besked om, at overførslen blev stoppet.';

  @override
  String get keepGoing => 'Fortsæt';

  @override
  String get stop => 'Stop';

  @override
  String get moreInfo => 'Detaljer';

  @override
  String get errorWrongCode =>
      'Koden passede ikke. Den blev skrevet forkert, eller nogen forsøgte at opsnappe overførslen. Intet blev sendt. Start forfra med en ny kode.';

  @override
  String get errorPeerLeft =>
      'Den anden side forlod forbindelsen, før overførslen var færdig.';

  @override
  String get errorDeclined => 'Modtageren afviste filerne.';

  @override
  String get errorCancelled => 'Den anden side stoppede overførslen.';

  @override
  String get errorStopped => 'Stoppet på denne enhed.';

  @override
  String get errorInvalidCode => 'Koden er ikke gyldig.';

  @override
  String get errorServer =>
      'Kunne ikke nå TotalDrop-serveren. Tjek din internetforbindelse.';

  @override
  String get errorCodeNotInUse =>
      'Koden er ikke i brug: den blev skrevet forkert, er udløbet eller er allerede brugt. Bed afsenderen om en ny.';

  @override
  String get errorCodeExpired =>
      'Koden udløb, før nogen brugte den. Send igen for at få en ny kode.';

  @override
  String get errorBadServer =>
      'Serveradressen er ikke gyldig. Den skal starte med wss://.';

  @override
  String get errorConnect =>
      'Kunne ikke oprette forbindelse til den anden enhed. Tjek, at begge enheder er online, og prøv igen.';

  @override
  String get errorInvalidOffer =>
      'Tilbuddet brød en sikkerhedsregel og blev afvist.';

  @override
  String get errorStorage => 'En fil kunne ikke læses eller gemmes.';

  @override
  String get errorOther => 'Noget gik galt.';

  @override
  String get settingsServer => 'Mødeserver';

  @override
  String get settingsServerHelp =>
      'Hjælper to enheder med samme kode med at finde hinanden. Den ser aldrig dine filer eller nøgler.';

  @override
  String get settingsServerReset => 'Brug standard';

  @override
  String get settingsServerInvalid => 'Skal starte med wss://';

  @override
  String get settingsSave => 'Gem';

  @override
  String get settingsAbout => 'Om';

  @override
  String get settingsAboutBody =>
      'TotalDrop er gratis og open source og finansieres af donationer. Licens: GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Privatliv';

  @override
  String get settingsPrivacyBody =>
      'Mødeserveren, relæserveren og Cloudflares STUN-server ser din IP-adresse, og det gør den anden enhed også, når I forbinder direkte (medmindre privatlivstilstand er slået til). Ingen af dem ser filnavne, indhold eller nøgler, og vores servere gemmer ingen logs. Filerne går direkte mellem enhederne, når det er muligt, ellers gennem relæserveren; de er end-to-end-krypterede i begge tilfælde.';

  @override
  String get privacyMode => 'Skjul min adresse for den anden enhed';

  @override
  String get privacyModeHint =>
      'Privatlivstilstand: alt går gennem relæserveren, så den anden enhed aldrig får din IP-adresse at vide. Langsommere, og den anden enhed skal bruge samme relæserver (det gør standardserveren).';

  @override
  String get privacyModeConfirmTitle => 'Slå privatlivstilstand til?';

  @override
  String get privacyModeConfirmBody =>
      'Overførsler bliver langsommere, fordi intet går direkte mellem enhederne. Mødeserveren og relæserveren ser stadig din IP-adresse; de ser aldrig dine filer eller nøgler.';

  @override
  String get privacyModeTurnOn => 'Slå til';

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
      'Denne oversættelse er ved at blive gennemgået. Hjælp med at forbedre den på GitHub.';

  @override
  String get sendDropHere => 'Slip filer her';

  @override
  String get sendDropHint => 'Du kan også trække filer over i dette vindue.';

  @override
  String get sendQrHint => 'Eller scan koden med den anden enheds kamera.';

  @override
  String get sendWaitingAccept =>
      'Tjek ordene, og vent så på, at modtageren accepterer.';

  @override
  String get receiveFromImage => 'Læs QR-kode fra billede';

  @override
  String get receiveQrNotFound => 'Der er ingen TotalDrop-kode i billedet.';

  @override
  String get pickTitle => 'Vælg filer, der skal sendes';

  @override
  String get pickImageTitle => 'Vælg et billede med en QR-kode';

  @override
  String get openFolder => 'Åbn mappe';

  @override
  String get back => 'Tilbage';

  @override
  String get codeSuggestions => 'Forslag';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString %';
  }
}
