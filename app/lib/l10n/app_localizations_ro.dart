// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Romanian Moldavian Moldovan (`ro`).
class AppLocalizationsRo extends AppLocalizations {
  AppLocalizationsRo([String locale = 'ro']) : super(locale);

  @override
  String get appTagline => 'Orice fișier, oricui, oriunde.';

  @override
  String get homeSendTitle => 'Trimite';

  @override
  String get homeSendBody => 'Alegeți fișiere și împărtășiți un cod scurt';

  @override
  String get homeReceiveTitle => 'Primește';

  @override
  String get homeReceiveBody => 'Introduceți un cod și salvați fișierele';

  @override
  String get homeFooter =>
      'Criptat end-to-end. Fără cont, fără e-mail, fără număr de telefon.';

  @override
  String get settings => 'Setări';

  @override
  String get sendTitle => 'Trimiteți fișiere';

  @override
  String get sendAddFiles => 'Adaugă fișiere';

  @override
  String get sendEmpty => 'Nimic selectat încă';

  @override
  String get sendEmptyHint =>
      'Adăugați fișierele pe care vreți să le trimiteți. Destinatarul primește exact acestea, nimic altceva.';

  @override
  String get sendStrongCode => 'Cod mai puternic';

  @override
  String get sendStrongCodeHint =>
      'Trei cuvinte în loc de două: mult mai greu de ghicit';

  @override
  String get sendCreateCode => 'Creează codul';

  @override
  String get sendCounting => 'Se numără…';

  @override
  String get sendRemove => 'Elimină';

  @override
  String get sendPreparing => 'Se pregătesc fișierele…';

  @override
  String get sendYourCode => 'Codul dvs.';

  @override
  String get sendCodeHint =>
      'Pe celălalt dispozitiv, deschideți TotalDrop, atingeți Primește și introduceți acest cod.';

  @override
  String get sendWaiting => 'Se așteaptă destinatarul…';

  @override
  String get sendCopy => 'Copiază';

  @override
  String get sendCopied => 'Cod copiat';

  @override
  String get sendShare => 'Distribuie';

  @override
  String sendShareText(String code) {
    return 'Primește fișierele mele cu TotalDrop. Cod: $code';
  }

  @override
  String get sendSending => 'Se trimite';

  @override
  String get sendDone => 'Trimis și verificat';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Toate cele $countString de fișiere au ajuns intacte.',
      few: 'Toate cele $countString fișiere au ajuns intacte.',
      one: 'Fișierul a ajuns intact.',
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
          '$countString de fișiere nu au ajuns intacte și au fost eliminate.',
      few: '$countString fișiere nu au ajuns intacte și au fost eliminate.',
      one: 'Un fișier nu a ajuns intact și a fost eliminat.',
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
      other: '$countString de elemente',
      few: '$countString elemente',
      one: '$countString element',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Primiți fișiere';

  @override
  String get receiveCodeLabel => 'Cod';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Cereți codul expeditorului.';

  @override
  String get receiveCodeInvalid =>
      'Nu pare a fi un cod. Codul începe cu un număr, urmat de două sau trei cuvinte.';

  @override
  String get receivePaste => 'Lipește';

  @override
  String get receiveConnect => 'Conectează';

  @override
  String get receiveConnecting => 'Conectare securizată…';

  @override
  String get receiveOffer => 'Fișiere primite';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString de fișiere',
      few: '$filesString fișiere',
      one: '$filesString fișier',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'și încă $countString';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Salvat în $folder';
  }

  @override
  String get receiveAccept => 'Acceptă';

  @override
  String get receiveDecline => 'Refuză';

  @override
  String get receiveDeclined => 'Ați refuzat fișierele.';

  @override
  String get receiveReceiving => 'Se primește';

  @override
  String get receiveDone => 'Primit și verificat';

  @override
  String get verificationTitle => 'Cuvinte de verificare';

  @override
  String get verificationHint =>
      'Cealaltă persoană vede aceleași patru cuvinte. Dacă diferă, opriți-vă.';

  @override
  String get sameNetworkNote =>
      'Funcționează prin internet. Când dispozitivele nu se pot conecta direct, transferul criptat trece prin serverul releu, care nu îl poate citi.';

  @override
  String get done => 'Gata';

  @override
  String get tryAgain => 'Reîncearcă';

  @override
  String get cancel => 'Anulează';

  @override
  String get cancelTransferTitle => 'Opriți acest transfer?';

  @override
  String get cancelTransferBody =>
      'Cealaltă parte va fi anunțată că transferul s-a oprit.';

  @override
  String get keepGoing => 'Continuă';

  @override
  String get stop => 'Oprește';

  @override
  String get moreInfo => 'Detalii';

  @override
  String get errorWrongCode =>
      'Codul nu s-a potrivit. A fost tastat greșit sau cineva a încercat să intercepteze transferul. Nu s-a trimis nimic. Începeți din nou cu un cod nou.';

  @override
  String get errorPeerLeft =>
      'Cealaltă parte a plecat înainte de terminarea transferului.';

  @override
  String get errorDeclined => 'Destinatarul a refuzat fișierele.';

  @override
  String get errorCancelled => 'Cealaltă parte a oprit transferul.';

  @override
  String get errorStopped => 'Oprit pe acest dispozitiv.';

  @override
  String get errorInvalidCode => 'Codul nu este valid.';

  @override
  String get errorServer =>
      'Serverul TotalDrop nu a putut fi contactat. Verificați conexiunea la internet.';

  @override
  String get errorConnect =>
      'Nu s-a putut realiza conexiunea cu celălalt dispozitiv. Verificați că ambele dispozitive sunt online și încercați din nou.';

  @override
  String get errorInvalidOffer =>
      'Oferta a încălcat o regulă de siguranță, așa că a fost refuzată.';

  @override
  String get errorStorage => 'Un fișier nu a putut fi citit sau salvat.';

  @override
  String get errorOther => 'Ceva nu a mers bine.';

  @override
  String get settingsServer => 'Server de întâlnire';

  @override
  String get settingsServerHelp =>
      'Ajută două dispozitive cu același cod să se găsească. Nu vede niciodată fișierele sau cheile dvs.';

  @override
  String get settingsServerReset => 'Folosește serverul implicit';

  @override
  String get settingsServerInvalid => 'Trebuie să înceapă cu wss://';

  @override
  String get settingsSave => 'Salvează';

  @override
  String get settingsAbout => 'Despre';

  @override
  String get settingsAboutBody =>
      'TotalDrop este gratuit și open source, finanțat din donații. Licență GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Confidențialitate';

  @override
  String get settingsPrivacyBody =>
      'Serverul de întâlnire, serverul releu și serverul STUN al Cloudflare vă văd adresa IP, la fel și celălalt dispozitiv când vă conectați direct (dacă modul de confidențialitate nu este activat). Niciunul nu vede numele fișierelor, conținutul sau cheile, iar serverele noastre nu păstrează jurnale. Fișierele merg direct între dispozitive când se poate, altfel prin serverul releu; în ambele cazuri sunt criptate end-to-end.';

  @override
  String get privacyMode => 'Ascunde adresa mea de celălalt dispozitiv';

  @override
  String get privacyModeHint =>
      'Modul de confidențialitate: totul trece prin serverul releu, așa că celălalt dispozitiv nu vă află niciodată adresa IP. Este mai lent, iar celălalt dispozitiv trebuie să folosească același server releu (cel implicit o face).';

  @override
  String get privacyModeConfirmTitle => 'Activați modul de confidențialitate?';

  @override
  String get privacyModeConfirmBody =>
      'Transferurile vor fi mai lente, pentru că nimic nu merge direct între dispozitive. Serverul de întâlnire și serverul releu vă văd în continuare adresa IP; nu vă văd niciodată fișierele sau cheile.';

  @override
  String get privacyModeTurnOn => 'Activează';

  @override
  String settingsVersion(String version) {
    return 'Versiunea $version';
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
      'Această traducere este în curs de revizuire. Ajutați la îmbunătățirea ei pe Weblate.';

  @override
  String get sendDropHere => 'Plasați fișierele aici';

  @override
  String get sendDropHint =>
      'Puteți și să trageți fișiere în această fereastră.';

  @override
  String get sendQrHint =>
      'Sau scanați acest cod cu camera celuilalt dispozitiv.';

  @override
  String get sendWaitingAccept =>
      'Verificați cuvintele, apoi așteptați ca destinatarul să accepte.';

  @override
  String get receiveFromImage => 'Citește codul QR din imagine';

  @override
  String get receiveQrNotFound =>
      'Nu există niciun cod TotalDrop în această imagine.';

  @override
  String get pickTitle => 'Alegeți fișierele de trimis';

  @override
  String get pickImageTitle => 'Alegeți o imagine cu un cod QR';

  @override
  String get openFolder => 'Deschide dosarul';

  @override
  String get back => 'Înapoi';

  @override
  String get codeSuggestions => 'Sugestii';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
