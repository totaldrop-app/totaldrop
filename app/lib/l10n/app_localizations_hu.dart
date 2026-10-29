// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Hungarian (`hu`).
class AppLocalizationsHu extends AppLocalizations {
  AppLocalizationsHu([String locale = 'hu']) : super(locale);

  @override
  String get appTagline => 'Bármilyen fájlt, bárkinek, bárhová.';

  @override
  String get homeSendTitle => 'Küldés';

  @override
  String get homeSendBody => 'Válassz fájlokat, és ossz meg egy rövid kódot';

  @override
  String get homeReceiveTitle => 'Fogadás';

  @override
  String get homeReceiveBody => 'Írd be a kódot, és mentsd a fájlokat';

  @override
  String get homeFooter =>
      'Végpontok közötti titkosítás. Nincs fiók, e-mail-cím vagy telefonszám.';

  @override
  String get settings => 'Beállítások';

  @override
  String get sendTitle => 'Fájlok küldése';

  @override
  String get sendAddFiles => 'Fájlok hozzáadása';

  @override
  String get sendEmpty => 'Még nincs kiválasztva semmi';

  @override
  String get sendEmptyHint =>
      'Add hozzá a küldendő fájlokat. A címzett pontosan ezeket kapja meg, mást nem.';

  @override
  String get sendStrongCode => 'Erősebb kód';

  @override
  String get sendStrongCodeHint =>
      'Kettő helyett három szó: sokkal nehezebb kitalálni';

  @override
  String get sendCreateCode => 'Kód létrehozása';

  @override
  String get sendCounting => 'Számolás…';

  @override
  String get sendRemove => 'Eltávolítás';

  @override
  String get sendPreparing => 'Fájlok előkészítése…';

  @override
  String get sendYourCode => 'A kódod';

  @override
  String get sendCodeHint =>
      'A másik eszközön nyisd meg a TotalDrop alkalmazást, koppints a Fogadás gombra, és írd be ezt a kódot.';

  @override
  String get sendWaiting => 'Várakozás a címzettre…';

  @override
  String get sendCopy => 'Másolás';

  @override
  String get sendCopied => 'Kód másolva';

  @override
  String get sendShare => 'Megosztás';

  @override
  String sendShareText(String code) {
    return 'Vedd át a fájljaimat a TotalDrop alkalmazással. Kód: $code';
  }

  @override
  String get sendSending => 'Küldés folyamatban';

  @override
  String get sendDone => 'Elküldve és ellenőrizve';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$countString fájl érkezett meg, mind sértetlenül.',
      one: 'A fájl sértetlenül megérkezett.',
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
          '$countString fájl nem érkezett meg sértetlenül, ezért törölve lett.',
      one: 'Egy fájl nem érkezett meg sértetlenül, ezért törölve lett.',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '$countString elem · $size';
  }

  @override
  String get receiveTitle => 'Fájlok fogadása';

  @override
  String get receiveCodeLabel => 'Kód';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Kérd el a kódot a küldőtől.';

  @override
  String get receiveCodeInvalid =>
      'Ez nem tűnik kódnak. A kód egy számmal kezdődik, utána két vagy három szó jön.';

  @override
  String get receivePaste => 'Beillesztés';

  @override
  String get receiveConnect => 'Csatlakozás';

  @override
  String get receiveConnecting => 'Biztonságos kapcsolódás…';

  @override
  String get receiveOffer => 'Bejövő fájlok';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesString fájl · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'és még $countString';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Mentve ide: $folder';
  }

  @override
  String get receiveAccept => 'Elfogadás';

  @override
  String get receiveDecline => 'Elutasítás';

  @override
  String get receiveDeclined => 'Elutasítottad a fájlokat.';

  @override
  String get receiveReceiving => 'Fogadás folyamatban';

  @override
  String get receiveDone => 'Fogadva és ellenőrizve';

  @override
  String get verificationTitle => 'Ellenőrző szavak';

  @override
  String get verificationHint =>
      'A másik fél ugyanazt a négy szót látja. Ha eltérnek, állj le.';

  @override
  String get sameNetworkNote =>
      'Az interneten keresztül működik. Ha az eszközök nem érik el egymást közvetlenül, a titkosított átvitel a továbbító szerveren megy át, amely nem tudja elolvasni.';

  @override
  String get done => 'Kész';

  @override
  String get tryAgain => 'Újra';

  @override
  String get cancel => 'Mégse';

  @override
  String get cancelTransferTitle => 'Leállítod az átvitelt?';

  @override
  String get cancelTransferBody =>
      'A másik fél értesítést kap, hogy az átvitel leállt.';

  @override
  String get keepGoing => 'Folytatás';

  @override
  String get stop => 'Leállítás';

  @override
  String get moreInfo => 'Részletek';

  @override
  String get errorWrongCode =>
      'A kód nem egyezett. Vagy elgépelték, vagy valaki megpróbálta elfogni az átvitelt. Semmi sem lett elküldve. Kezdd újra egy új kóddal.';

  @override
  String get errorPeerLeft =>
      'A másik fél kilépett, mielőtt az átvitel befejeződött.';

  @override
  String get errorDeclined => 'A címzett elutasította a fájlokat.';

  @override
  String get errorCancelled => 'A másik fél leállította az átvitelt.';

  @override
  String get errorStopped => 'Ezen az eszközön leállítva.';

  @override
  String get errorInvalidCode => 'Ez a kód érvénytelen.';

  @override
  String get errorServer =>
      'Nem sikerült elérni a TotalDrop szervert. Ellenőrizd az internetkapcsolatot.';

  @override
  String get errorConnect =>
      'Nem sikerült kapcsolódni a másik eszközhöz. Ellenőrizd, hogy mindkét eszköz online van-e, és próbáld újra.';

  @override
  String get errorInvalidOffer =>
      'Az ajánlat megszegett egy biztonsági szabályt, ezért el lett utasítva.';

  @override
  String get errorStorage => 'Egy fájlt nem sikerült beolvasni vagy menteni.';

  @override
  String get errorOther => 'Hiba történt.';

  @override
  String get settingsServer => 'Találkozási szerver';

  @override
  String get settingsServerHelp =>
      'Segít, hogy két, azonos kódot használó eszköz megtalálja egymást. A fájljaidat és a kulcsaidat soha nem látja.';

  @override
  String get settingsServerReset => 'Alapértelmezett használata';

  @override
  String get settingsServerInvalid => 'wss:// előtaggal kell kezdődnie';

  @override
  String get settingsSave => 'Mentés';

  @override
  String get settingsAbout => 'Névjegy';

  @override
  String get settingsAboutBody =>
      'A TotalDrop ingyenes és nyílt forráskódú, adományokból működik. Licenc: GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Adatvédelem';

  @override
  String get settingsPrivacyBody =>
      'A találkozási szerver, a továbbító szerver és a Cloudflare STUN-szervere látja az IP-címedet, és közvetlen kapcsolatnál a másik eszköz is (hacsak nincs bekapcsolva az adatvédelmi mód). Egyikük sem látja a fájlneveket, a tartalmat vagy a kulcsokat, és a szervereink nem naplóznak. A fájlok, ha lehet, közvetlenül mennek az eszközök között, ha nem, a továbbító szerveren át; végpontok közötti titkosítással mindkét esetben.';

  @override
  String get privacyMode => 'Címem elrejtése a másik eszköz elől';

  @override
  String get privacyModeHint =>
      'Adatvédelmi mód: minden a továbbító szerveren megy át, így a másik eszköz sosem tudja meg az IP-címedet. Lassabb, és a másik eszköznek ugyanazt a továbbító szervert kell használnia (az alapértelmezett ilyen).';

  @override
  String get privacyModeConfirmTitle => 'Bekapcsolod az adatvédelmi módot?';

  @override
  String get privacyModeConfirmBody =>
      'Az átvitel lassabb lesz, mert semmi sem megy közvetlenül az eszközök között. A találkozási szerver és a továbbító szerver továbbra is látja az IP-címedet; a fájljaidat és a kulcsaidat soha.';

  @override
  String get privacyModeTurnOn => 'Bekapcsolás';

  @override
  String settingsVersion(String version) {
    return 'Verzió: $version';
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
      'Ez a fordítás ellenőrzés alatt áll. Segíts javítani a Weblate-en.';

  @override
  String get sendDropHere => 'Húzd ide a fájlokat';

  @override
  String get sendDropHint => 'A fájlokat erre az ablakra is ráhúzhatod.';

  @override
  String get sendQrHint =>
      'Vagy olvasd be ezt a kódot a másik eszköz kamerájával.';

  @override
  String get sendWaitingAccept =>
      'Ellenőrizd a szavakat, aztán várd meg, hogy a címzett elfogadja.';

  @override
  String get receiveFromImage => 'QR-kód beolvasása képről';

  @override
  String get receiveQrNotFound => 'Ezen a képen nincs TotalDrop-kód.';

  @override
  String get pickTitle => 'Válaszd ki a küldendő fájlokat';

  @override
  String get pickImageTitle => 'Válassz egy QR-kódot tartalmazó képet';

  @override
  String get openFolder => 'Mappa megnyitása';

  @override
  String get back => 'Vissza';

  @override
  String get codeSuggestions => 'Javaslatok';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
