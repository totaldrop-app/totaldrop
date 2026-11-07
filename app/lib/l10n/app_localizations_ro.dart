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
  String get sendAddFailed =>
      'Fișierele nu au putut fi adăugate. Încercați din nou. Dacă se întâmplă iar, verificați dacă pe acest dispozitiv este destul spațiu liber sau alegeți fișierele din alt loc.';

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
  String get errorCodeNotInUse =>
      'Codul nu este folosit: a fost tastat greșit, a expirat sau a fost deja folosit. Cereți expeditorului unul nou.';

  @override
  String get errorCodeExpired =>
      'Codul a expirat înainte să îl folosească cineva. Trimiteți din nou pentru a primi un cod nou.';

  @override
  String get errorBadServer =>
      'Adresa serverului nu este validă. Trebuie să înceapă cu wss://.';

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
  String get causeTimedOut =>
      'Conexiunea nu mai răspunde. Unul dintre dispozitive poate a pierdut rețeaua sau a intrat în repaus.';

  @override
  String get causeConnectionLost =>
      'Conexiunea cu celălalt dispozitiv s-a întrerupt.';

  @override
  String get causeNoRoute =>
      'Celălalt dispozitiv nu a putut fi contactat nici direct, nici prin serverul releu.';

  @override
  String get causeRelay =>
      'Serverul releu nu a putut fi contactat sau nu este configurat, ori cele două dispozitive nu folosesc același server.';

  @override
  String get causeProtocol =>
      'Celălalt dispozitiv a trimis ceva ce această versiune nu înțelege. Actualizarea TotalDrop pe ambele dispozitive poate ajuta.';

  @override
  String get causeDiskFull =>
      'Nu există suficient spațiu liber pe acest dispozitiv.';

  @override
  String get causeNoPermission =>
      'TotalDrop nu are permisiunea să citească sau să scrie un fișier ori un dosar de care are nevoie.';

  @override
  String get causeNotFound =>
      'Un fișier sau un dosar nu a fost găsit. Poate a fost mutat sau șters.';

  @override
  String get causeReadOnly =>
      'Dosarul se află pe un disc pe care nu se poate scrie.';

  @override
  String get causeFileChanged =>
      'Un fișier s-a schimbat în timp ce era trimis. Trimite-l din nou după ce este salvat.';

  @override
  String get technicalDetail =>
      'Mesaj tehnic în engleză, de inclus când raportezi o problemă:';

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
      'Această traducere este în curs de revizuire. Ajutați la îmbunătățirea ei pe GitHub.';

  @override
  String get settingsWordsEnglish =>
      'Codurile și cuvintele de verificare sunt mereu în engleză, ca două dispozitive în limbi diferite să arate aceleași cuvinte.';

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

  @override
  String get editUndo => 'Anulează';

  @override
  String get editRedo => 'Refă';

  @override
  String get editCut => 'Decupează';

  @override
  String get editCopy => 'Copiază';

  @override
  String get editPaste => 'Lipește';

  @override
  String get editDelete => 'Șterge';

  @override
  String get editSelectAll => 'Selectează tot';

  @override
  String get transferNoticeChannel => 'Transferuri';

  @override
  String get transferNoticeSending => 'Se trimite cu TotalDrop';

  @override
  String get transferNoticeReceiving => 'Se primește cu TotalDrop';

  @override
  String get transferNoticeBody => 'Continuă cât timp folosiți alte aplicații.';

  @override
  String get shareTextSoon =>
      'TotalDrop nu trimite încă text, doar fișiere. Partajează în schimb un fișier.';

  @override
  String get shareBusy =>
      'Un transfer este încă în desfășurare. Așteaptă să se termine sau oprește-l, apoi partajează din nou.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count de elemente partajate nu au putut fi citite și au fost omise.',
      few: '$count elemente partajate nu au putut fi citite și au fost omise.',
      one: 'Un element partajat nu a putut fi citit și a fost omis.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'Transfer de fișiere';

  @override
  String get desktopComment =>
      'Trimite fișiere direct pe alt dispozitiv, criptate integral';

  @override
  String get desktopKeywords => 'trimite;primește;partajează;transfer;fișiere;';

  @override
  String get desktopSendWith => 'Trimite cu TotalDrop';

  @override
  String get sendText => 'Trimite text';

  @override
  String get sendTextHint => 'Scrieți sau lipiți textul de trimis';

  @override
  String sendTextTooLong(String size) {
    return 'Un text poate avea cel mult $size. Trimiteți un text mai lung ca fișier.';
  }

  @override
  String get sendTextDone => 'Textul a ajuns întreg.';

  @override
  String get sendClipboard => 'Trimite clipboardul';

  @override
  String get sendClipboardEmpty => 'Clipboardul nu conține text.';

  @override
  String get receiveOfferText => 'Text primit';

  @override
  String get receiveOfferTextNote => 'Textul este afișat aici, nu este salvat.';

  @override
  String get receiveTextDone => 'Text primit și verificat';

  @override
  String get receiveTextDeclined => 'Ați refuzat textul.';

  @override
  String get receiveTextHidden =>
      'Textul conține caractere ascunse, afișate ca U+…. La copiere obțineți textul exact așa cum a sosit.';

  @override
  String get sendClipboardKept =>
      'Textul deja scris a fost păstrat; clipboardul nu a fost folosit.';

  @override
  String get sendTextShortened =>
      'Se afișează doar începutul textului; se trimite tot.';

  @override
  String get receiveTextShortened =>
      'Se afișează doar începutul textului. Copiază textul îl copiază pe tot.';

  @override
  String get copyText => 'Copiază textul';

  @override
  String get textCopied => 'Text copiat';

  @override
  String get errorTextDamaged =>
      'Textul nu a ajuns întreg, așa că nu este afișat.';
}
