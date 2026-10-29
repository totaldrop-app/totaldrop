// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Swedish (`sv`).
class AppLocalizationsSv extends AppLocalizations {
  AppLocalizationsSv([String locale = 'sv']) : super(locale);

  @override
  String get appTagline =>
      'Vilken fil som helst, till vem som helst, var som helst.';

  @override
  String get homeSendTitle => 'Skicka';

  @override
  String get homeSendBody => 'Välj filer och dela en kort kod';

  @override
  String get homeReceiveTitle => 'Ta emot';

  @override
  String get homeReceiveBody => 'Ange en kod och spara filerna';

  @override
  String get homeFooter =>
      'End-to-end-krypterat. Inget konto, ingen e-post, inget telefonnummer.';

  @override
  String get settings => 'Inställningar';

  @override
  String get sendTitle => 'Skicka filer';

  @override
  String get sendAddFiles => 'Lägg till filer';

  @override
  String get sendEmpty => 'Inget valt ännu';

  @override
  String get sendEmptyHint =>
      'Lägg till filerna du vill skicka. Mottagaren får exakt dessa, inget annat.';

  @override
  String get sendStrongCode => 'Starkare kod';

  @override
  String get sendStrongCodeHint =>
      'Tre ord i stället för två: mycket svårare att gissa';

  @override
  String get sendCreateCode => 'Skapa kod';

  @override
  String get sendCounting => 'Räknar…';

  @override
  String get sendRemove => 'Ta bort';

  @override
  String get sendPreparing => 'Förbereder dina filer…';

  @override
  String get sendYourCode => 'Din kod';

  @override
  String get sendCodeHint =>
      'Öppna TotalDrop på den andra enheten, tryck på Ta emot och ange den här koden.';

  @override
  String get sendWaiting => 'Väntar på mottagaren…';

  @override
  String get sendCopy => 'Kopiera';

  @override
  String get sendCopied => 'Koden har kopierats';

  @override
  String get sendShare => 'Dela';

  @override
  String sendShareText(String code) {
    return 'Hämta mina filer med TotalDrop. Kod: $code';
  }

  @override
  String get sendSending => 'Skickar';

  @override
  String get sendDone => 'Skickat och verifierat';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Alla $countString filer kom fram oskadda.',
      one: 'Filen kom fram oskadd.',
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
      other: '$countString filer kom inte fram oskadda och kastades.',
      one: 'En fil kom inte fram oskadd och kastades.',
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
      other: '$countString objekt',
      one: '$countString objekt',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Ta emot filer';

  @override
  String get receiveCodeLabel => 'Kod';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Be avsändaren om koden.';

  @override
  String get receiveCodeInvalid =>
      'Det ser inte ut som en kod. Den börjar med ett tal följt av två eller tre ord.';

  @override
  String get receivePaste => 'Klistra in';

  @override
  String get receiveConnect => 'Anslut';

  @override
  String get receiveConnecting => 'Ansluter säkert…';

  @override
  String get receiveOffer => 'Inkommande filer';

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

    return 'och $countString till';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Sparas i $folder';
  }

  @override
  String get receiveAccept => 'Acceptera';

  @override
  String get receiveDecline => 'Avböj';

  @override
  String get receiveDeclined => 'Du avböjde filerna.';

  @override
  String get receiveReceiving => 'Tar emot';

  @override
  String get receiveDone => 'Mottaget och verifierat';

  @override
  String get verificationTitle => 'Verifieringsord';

  @override
  String get verificationHint =>
      'Den andra personen ser samma fyra ord. Om de skiljer sig åt, avbryt.';

  @override
  String get sameNetworkNote =>
      'Fungerar över internet. När enheterna inte når varandra direkt går den krypterade överföringen via reläservern, som inte kan läsa den.';

  @override
  String get done => 'Klar';

  @override
  String get tryAgain => 'Försök igen';

  @override
  String get cancel => 'Avbryt';

  @override
  String get cancelTransferTitle => 'Stoppa överföringen?';

  @override
  String get cancelTransferBody =>
      'Den andra sidan får veta att överföringen stoppades.';

  @override
  String get keepGoing => 'Fortsätt';

  @override
  String get stop => 'Stoppa';

  @override
  String get moreInfo => 'Detaljer';

  @override
  String get errorWrongCode =>
      'Koden stämde inte. Den skrevs fel, eller så försökte någon fånga upp överföringen. Inget skickades. Börja om med en ny kod.';

  @override
  String get errorPeerLeft =>
      'Den andra sidan lämnade innan överföringen var klar.';

  @override
  String get errorDeclined => 'Mottagaren avböjde filerna.';

  @override
  String get errorCancelled => 'Den andra sidan stoppade överföringen.';

  @override
  String get errorStopped => 'Stoppad på den här enheten.';

  @override
  String get errorInvalidCode => 'Koden är inte giltig.';

  @override
  String get errorServer =>
      'Kunde inte nå TotalDrop-servern. Kontrollera din internetanslutning.';

  @override
  String get errorConnect =>
      'Kunde inte ansluta till den andra enheten. Kontrollera att båda enheterna är online och försök igen.';

  @override
  String get errorInvalidOffer =>
      'Erbjudandet bröt mot en säkerhetsregel och avvisades.';

  @override
  String get errorStorage => 'En fil kunde inte läsas eller sparas.';

  @override
  String get errorOther => 'Något gick fel.';

  @override
  String get settingsServer => 'Mötesserver';

  @override
  String get settingsServerHelp =>
      'Hjälper två enheter med samma kod att hitta varandra. Den ser aldrig dina filer eller nycklar.';

  @override
  String get settingsServerReset => 'Använd standard';

  @override
  String get settingsServerInvalid => 'Måste börja med wss://';

  @override
  String get settingsSave => 'Spara';

  @override
  String get settingsAbout => 'Om';

  @override
  String get settingsAboutBody =>
      'TotalDrop är gratis och öppen källkod, finansierat av donationer. Licens: GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Integritet';

  @override
  String get settingsPrivacyBody =>
      'Mötesservern, reläservern och Cloudflares STUN-server ser din IP-adress, och det gör även den andra enheten när ni ansluter direkt (om inte integritetsläget är på). Ingen av dem ser filnamn, innehåll eller nycklar, och våra servrar sparar inga loggar. Filerna går direkt mellan enheterna när det går, annars via reläservern; de är end-to-end-krypterade i båda fallen.';

  @override
  String get privacyMode => 'Dölj min adress för den andra enheten';

  @override
  String get privacyModeHint =>
      'Integritetsläge: allt går via reläservern, så den andra enheten får aldrig veta din IP-adress. Långsammare, och den andra enheten måste använda samma reläserver (standardservern gör det).';

  @override
  String get privacyModeConfirmTitle => 'Slå på integritetsläget?';

  @override
  String get privacyModeConfirmBody =>
      'Överföringar blir långsammare eftersom inget går direkt mellan enheterna. Mötesservern och reläservern ser fortfarande din IP-adress; de ser aldrig dina filer eller nycklar.';

  @override
  String get privacyModeTurnOn => 'Slå på';

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
      'Den här översättningen granskas. Hjälp till att förbättra den på Weblate.';

  @override
  String get sendDropHere => 'Släpp filer här';

  @override
  String get sendDropHint => 'Du kan också dra filer till det här fönstret.';

  @override
  String get sendQrHint => 'Eller skanna koden med den andra enhetens kamera.';

  @override
  String get sendWaitingAccept =>
      'Kontrollera orden och vänta sedan på att mottagaren accepterar.';

  @override
  String get receiveFromImage => 'Läs QR-kod från bild';

  @override
  String get receiveQrNotFound => 'Det finns ingen TotalDrop-kod i bilden.';

  @override
  String get pickTitle => 'Välj filer att skicka';

  @override
  String get pickImageTitle => 'Välj en bild med en QR-kod';

  @override
  String get openFolder => 'Öppna mapp';

  @override
  String get back => 'Tillbaka';

  @override
  String get codeSuggestions => 'Förslag';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString %';
  }
}
