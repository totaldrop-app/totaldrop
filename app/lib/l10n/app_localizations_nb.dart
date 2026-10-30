// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Norwegian Bokmål (`nb`).
class AppLocalizationsNb extends AppLocalizations {
  AppLocalizationsNb([String locale = 'nb']) : super(locale);

  @override
  String get appTagline =>
      'Hvilken som helst fil, til hvem som helst, hvor som helst.';

  @override
  String get homeSendTitle => 'Send';

  @override
  String get homeSendBody => 'Velg filer og del en kort kode';

  @override
  String get homeReceiveTitle => 'Motta';

  @override
  String get homeReceiveBody => 'Skriv inn en kode og lagre filene';

  @override
  String get homeFooter =>
      'Ende-til-ende-kryptert. Ingen konto, ingen e-post, intet telefonnummer.';

  @override
  String get settings => 'Innstillinger';

  @override
  String get sendTitle => 'Send filer';

  @override
  String get sendAddFiles => 'Legg til filer';

  @override
  String get sendEmpty => 'Ingenting valgt ennå';

  @override
  String get sendEmptyHint =>
      'Legg til filene du vil sende. Mottakeren får nøyaktig disse, ingenting annet.';

  @override
  String get sendStrongCode => 'Sterkere kode';

  @override
  String get sendStrongCodeHint =>
      'Tre ord i stedet for to: mye vanskeligere å gjette';

  @override
  String get sendCreateCode => 'Lag kode';

  @override
  String get sendCounting => 'Teller …';

  @override
  String get sendRemove => 'Fjern';

  @override
  String get sendPreparing => 'Klargjør filene dine …';

  @override
  String get sendYourCode => 'Koden din';

  @override
  String get sendCodeHint =>
      'Åpne TotalDrop på den andre enheten, trykk på Motta og skriv inn denne koden.';

  @override
  String get sendWaiting => 'Venter på mottakeren …';

  @override
  String get sendCopy => 'Kopier';

  @override
  String get sendCopied => 'Koden er kopiert';

  @override
  String get sendShare => 'Del';

  @override
  String sendShareText(String code) {
    return 'Hent filene mine med TotalDrop. Kode: $code';
  }

  @override
  String get sendSending => 'Sender';

  @override
  String get sendDone => 'Sendt og bekreftet';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Alle $countString filene kom fram uskadd.',
      one: 'Filen kom fram uskadd.',
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
      other: '$countString filer kom ikke fram uskadd og ble forkastet.',
      one: 'Én fil kom ikke fram uskadd og ble forkastet.',
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
  String get receiveTitle => 'Motta filer';

  @override
  String get receiveCodeLabel => 'Kode';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Be avsenderen om koden.';

  @override
  String get receiveCodeInvalid =>
      'Det ser ikke ut som en kode. Den starter med et tall, etterfulgt av to eller tre ord.';

  @override
  String get receivePaste => 'Lim inn';

  @override
  String get receiveConnect => 'Koble til';

  @override
  String get receiveConnecting => 'Kobler til sikkert …';

  @override
  String get receiveOffer => 'Innkommende filer';

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

    return 'og $countString til';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Lagres i $folder';
  }

  @override
  String get receiveAccept => 'Godta';

  @override
  String get receiveDecline => 'Avslå';

  @override
  String get receiveDeclined => 'Du avslo filene.';

  @override
  String get receiveReceiving => 'Mottar';

  @override
  String get receiveDone => 'Mottatt og bekreftet';

  @override
  String get verificationTitle => 'Bekreftelsesord';

  @override
  String get verificationHint =>
      'Den andre personen ser de samme fire ordene. Hvis de er forskjellige, stopp.';

  @override
  String get sameNetworkNote =>
      'Fungerer over internett. Når enhetene ikke når hverandre direkte, går den krypterte overføringen gjennom reléserveren, som ikke kan lese den.';

  @override
  String get done => 'Ferdig';

  @override
  String get tryAgain => 'Prøv igjen';

  @override
  String get cancel => 'Avbryt';

  @override
  String get cancelTransferTitle => 'Stoppe overføringen?';

  @override
  String get cancelTransferBody =>
      'Den andre siden får beskjed om at overføringen ble stoppet.';

  @override
  String get keepGoing => 'Fortsett';

  @override
  String get stop => 'Stopp';

  @override
  String get moreInfo => 'Detaljer';

  @override
  String get errorWrongCode =>
      'Koden stemte ikke. Den ble skrevet feil, eller noen prøvde å avskjære overføringen. Ingenting ble sendt. Start på nytt med en ny kode.';

  @override
  String get errorPeerLeft =>
      'Den andre siden forlot før overføringen var ferdig.';

  @override
  String get errorDeclined => 'Mottakeren avslo filene.';

  @override
  String get errorCancelled => 'Den andre siden stoppet overføringen.';

  @override
  String get errorStopped => 'Stoppet på denne enheten.';

  @override
  String get errorInvalidCode => 'Koden er ikke gyldig.';

  @override
  String get errorServer =>
      'Kunne ikke nå TotalDrop-serveren. Sjekk internettilkoblingen.';

  @override
  String get errorConnect =>
      'Kunne ikke koble til den andre enheten. Sjekk at begge enhetene er på nett, og prøv igjen.';

  @override
  String get errorInvalidOffer =>
      'Tilbudet brøt en sikkerhetsregel og ble avvist.';

  @override
  String get errorStorage => 'En fil kunne ikke leses eller lagres.';

  @override
  String get errorOther => 'Noe gikk galt.';

  @override
  String get settingsServer => 'Møteserver';

  @override
  String get settingsServerHelp =>
      'Hjelper to enheter med samme kode å finne hverandre. Den ser aldri filene eller nøklene dine.';

  @override
  String get settingsServerReset => 'Bruk standard';

  @override
  String get settingsServerInvalid => 'Må starte med wss://';

  @override
  String get settingsSave => 'Lagre';

  @override
  String get settingsAbout => 'Om';

  @override
  String get settingsAboutBody =>
      'TotalDrop er gratis og åpen kildekode, finansiert av donasjoner. Lisens: GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Personvern';

  @override
  String get settingsPrivacyBody =>
      'Møteserveren, reléserveren og Cloudflares STUN-server ser IP-adressen din, og det gjør også den andre enheten når dere kobler direkte (med mindre personvernmodus er på). Ingen av dem ser filnavn, innhold eller nøkler, og serverne våre lagrer ingen logger. Filene går direkte mellom enhetene når det er mulig, ellers gjennom reléserveren; de er ende-til-ende-kryptert uansett.';

  @override
  String get privacyMode => 'Skjul adressen min for den andre enheten';

  @override
  String get privacyModeHint =>
      'Personvernmodus: alt går gjennom reléserveren, så den andre enheten får aldri vite IP-adressen din. Tregere, og den andre enheten må bruke samme reléserver (standardserveren gjør det).';

  @override
  String get privacyModeConfirmTitle => 'Slå på personvernmodus?';

  @override
  String get privacyModeConfirmBody =>
      'Overføringer blir tregere, fordi ingenting går direkte mellom enhetene. Møteserveren og reléserveren ser fortsatt IP-adressen din; de ser aldri filene eller nøklene dine.';

  @override
  String get privacyModeTurnOn => 'Slå på';

  @override
  String settingsVersion(String version) {
    return 'Versjon $version';
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
      'Denne oversettelsen blir gjennomgått. Hjelp til med å forbedre den på GitHub.';

  @override
  String get sendDropHere => 'Slipp filer her';

  @override
  String get sendDropHint => 'Du kan også dra filer inn i dette vinduet.';

  @override
  String get sendQrHint =>
      'Eller skann koden med kameraet på den andre enheten.';

  @override
  String get sendWaitingAccept =>
      'Sjekk ordene, og vent deretter på at mottakeren godtar.';

  @override
  String get receiveFromImage => 'Les QR-kode fra bilde';

  @override
  String get receiveQrNotFound => 'Det er ingen TotalDrop-kode i bildet.';

  @override
  String get pickTitle => 'Velg filer som skal sendes';

  @override
  String get pickImageTitle => 'Velg et bilde med en QR-kode';

  @override
  String get openFolder => 'Åpne mappe';

  @override
  String get back => 'Tilbake';

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
