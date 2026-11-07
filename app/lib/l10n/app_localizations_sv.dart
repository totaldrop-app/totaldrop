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
  String get sendAddFailed =>
      'Filerna kunde inte läggas till. Försök igen. Om det händer igen, kontrollera att det finns tillräckligt med ledigt utrymme på den här enheten, eller välj filerna från en annan plats.';

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
  String get errorCodeNotInUse =>
      'Koden används inte: den skrevs fel, har gått ut eller har redan använts. Be avsändaren om en ny.';

  @override
  String get errorCodeExpired =>
      'Koden gick ut innan någon använde den. Skicka igen för att få en ny kod.';

  @override
  String get errorBadServer =>
      'Serveradressen är inte giltig. Den måste börja med wss://.';

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
  String get causeTimedOut =>
      'Anslutningen slutade svara. En av enheterna kan ha tappat nätverket eller gått i viloläge.';

  @override
  String get causeConnectionLost =>
      'Anslutningen till den andra enheten bröts.';

  @override
  String get causeNoRoute =>
      'Den andra enheten gick inte att nå, varken direkt eller via reläservern.';

  @override
  String get causeRelay =>
      'Reläservern gick inte att nå eller är inte inställd, eller så använder enheterna inte samma reläserver.';

  @override
  String get causeProtocol =>
      'Den andra enheten skickade något som den här versionen inte förstår. Det kan hjälpa att uppdatera TotalDrop på båda enheterna.';

  @override
  String get causeDiskFull =>
      'Det finns inte tillräckligt med ledigt utrymme på den här enheten.';

  @override
  String get causeNoPermission =>
      'TotalDrop får inte läsa eller skriva en fil eller mapp som behövs.';

  @override
  String get causeNotFound =>
      'En fil eller mapp hittades inte. Den kan ha flyttats eller tagits bort.';

  @override
  String get causeReadOnly =>
      'Mappen ligger på en disk som inte går att skriva till.';

  @override
  String get causeFileChanged =>
      'En fil ändrades medan den skickades. Skicka den igen när den är sparad.';

  @override
  String get technicalDetail =>
      'Tekniskt meddelande på engelska, att bifoga när du rapporterar ett problem:';

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
      'Den här översättningen granskas. Hjälp till att förbättra den på GitHub.';

  @override
  String get settingsWordsEnglish =>
      'Koder och verifieringsord är alltid på engelska, så att två enheter med olika språk visar samma ord.';

  @override
  String get sendDropHere => 'Släpp filer här';

  @override
  String get sendDropHint => 'Du kan också dra filer till det här fönstret.';

  @override
  String get keepAwakeReason => 'TotalDrop skickar eller tar emot filer';

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

  @override
  String get editUndo => 'Ångra';

  @override
  String get editRedo => 'Gör om';

  @override
  String get editCut => 'Klipp ut';

  @override
  String get editCopy => 'Kopiera';

  @override
  String get editPaste => 'Klistra in';

  @override
  String get editDelete => 'Ta bort';

  @override
  String get editSelectAll => 'Markera allt';

  @override
  String get transferNoticeChannel => 'Överföringar';

  @override
  String get transferNoticeSending => 'Skickar med TotalDrop';

  @override
  String get transferNoticeReceiving => 'Tar emot med TotalDrop';

  @override
  String get transferNoticeBody => 'Fortsätter medan du använder andra appar.';

  @override
  String get shareBusy =>
      'En överföring pågår fortfarande. Vänta tills den är klar eller stoppa den och dela sedan igen.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count delade objekt kunde inte läsas och utelämnades.',
      one: 'Ett delat objekt kunde inte läsas och utelämnades.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'Filöverföring';

  @override
  String get desktopComment =>
      'Skicka filer direkt till en annan enhet, end-to-end-krypterat';

  @override
  String get desktopKeywords => 'skicka;ta emot;dela;överför;filer;';

  @override
  String get desktopSendWith => 'Skicka med TotalDrop';

  @override
  String get metainfoSummary => 'Skicka filer direkt till en annan enhet';

  @override
  String get metainfoAbout =>
      'TotalDrop skickar filer och mappar från en enhet till en annan, end-to-end-krypterat. Välj filerna, dela en kort kod eller en QR-kod, så tar den andra enheten emot dem. Inget konto, ingen e-post, inget telefonnummer.';

  @override
  String get metainfoVerified =>
      'Varje fil kontrolleras när den kommer fram; skadade filer kasseras';

  @override
  String get metainfoWords =>
      'Fyra verifieringsord visar att ingen sitter emellan';

  @override
  String get metainfoTheme =>
      'Följer skrivbordets tema: Plasma, Hyprland, Omarchy, GNOME';

  @override
  String get metainfoOnePackage =>
      'Använd ett Linux-paket i taget. Flatpak-paketet och .pkg-, .deb- eller .rpm-paketet delar ett namn, så medan det ena är öppet tar en start av det andra fram det öppna fönstret i stället för att öppna ett nytt.';

  @override
  String get otherPackageFlatpak =>
      'TotalDrop är redan öppet från Flatpak, så det fönstret togs fram. Använd ett paket i taget: avsluta det fönstret för att öppna det här.';

  @override
  String get otherPackageSystem =>
      'TotalDrop är redan öppet från .pkg-, .deb- eller .rpm-paketet, så det fönstret togs fram. Använd ett paket i taget: avsluta det fönstret för att öppna det här.';

  @override
  String get sendText => 'Skicka text';

  @override
  String get sendTextHint => 'Skriv eller klistra in texten som ska skickas';

  @override
  String sendTextTooLong(String size) {
    return 'En text får vara högst $size. Skicka en längre text som fil.';
  }

  @override
  String get sendTextDone => 'Texten kom fram oskadd.';

  @override
  String get sendClipboard => 'Skicka urklipp';

  @override
  String get sendClipboardEmpty => 'Urklipp innehåller ingen text.';

  @override
  String get receiveOfferText => 'Inkommande text';

  @override
  String get receiveOfferTextNote => 'Texten visas här och sparas inte.';

  @override
  String get receiveTextDone => 'Text mottagen och verifierad';

  @override
  String get receiveTextDeclined => 'Du avböjde texten.';

  @override
  String get receiveTextHidden =>
      'Texten innehåller dolda tecken, som visas som U+…. När du kopierar får du texten exakt som den kom.';

  @override
  String get sendTextHidden =>
      'Texten innehåller dolda tecken, till exempel riktningsmarkeringar. De skickas med.';

  @override
  String get sendClipboardKept =>
      'Den text som redan skrivits behölls; urklippet användes inte.';

  @override
  String get sendTextShortened =>
      'Bara början av texten visas; hela texten skickas.';

  @override
  String get receiveTextShortened =>
      'Bara början av texten visas. Kopiera text kopierar allt.';

  @override
  String get copyText => 'Kopiera text';

  @override
  String get textCopied => 'Texten kopierades';

  @override
  String get errorTextDamaged =>
      'Texten kom inte fram oskadd, så den visas inte.';

  @override
  String get setupBackgroundTitle => 'Låt överföringar fortsätta i bakgrunden';

  @override
  String get setupBackgroundBody =>
      'För att en överföring ska fortsätta när du byter app eller låser skärmen, låt TotalDrop köras utan batteribegränsningar. TotalDrop arbetar bara i bakgrunden medan en överföring som du har startat pågår.';

  @override
  String get setupBackgroundAllowed =>
      'Androids batterioptimering är av för TotalDrop.';

  @override
  String get setupBackgroundRestricted =>
      'Androids batterioptimering är på för TotalDrop: en överföring kan stanna när du byter app.';

  @override
  String get setupBackgroundBattery => 'Öppna batteriinställningar';

  @override
  String get setupBackgroundBatteryHint =>
      'Öppna Batteri på TotalDrops appinfosida och tillåt obegränsad användning. Namnen på de här inställningarna skiljer sig mellan telefoner.';

  @override
  String setupBackgroundMaker(String maker) {
    return '$maker-telefoner har en egen batterisparfunktion';
  }

  @override
  String get setupBackgroundMakerBody =>
      'Öppna även TotalDrops batterisparfunktion och välj ”Inga begränsningar”. Om telefonen har Autostart, slå på det för TotalDrop.';

  @override
  String get setupBackgroundMakerOpen => 'Öppna TotalDrops batterisparfunktion';

  @override
  String get setupBackgroundAutostart => 'Öppna Autostart';

  @override
  String get setupBackgroundSkip => 'Inte nu';

  @override
  String get settingsBackground => 'Överföringar i bakgrunden';

  @override
  String get settingsBackgroundOpen => 'Batteriinställningar';
}
