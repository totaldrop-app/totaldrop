// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Czech (`cs`).
class AppLocalizationsCs extends AppLocalizations {
  AppLocalizationsCs([String locale = 'cs']) : super(locale);

  @override
  String get appTagline => 'Jakýkoli soubor, komukoli, kamkoli.';

  @override
  String get homeSendTitle => 'Odeslat';

  @override
  String get homeSendBody => 'Vyberte soubory a sdílejte krátký kód';

  @override
  String get homeReceiveTitle => 'Přijmout';

  @override
  String get homeReceiveBody => 'Zadejte kód a uložte soubory';

  @override
  String get homeFooter =>
      'Koncové šifrování. Žádný účet, e-mail ani telefonní číslo.';

  @override
  String get settings => 'Nastavení';

  @override
  String get sendTitle => 'Odeslat soubory';

  @override
  String get sendAddFiles => 'Přidat soubory';

  @override
  String get sendEmpty => 'Zatím nic nevybráno';

  @override
  String get sendEmptyHint =>
      'Přidejte soubory, které chcete odeslat. Příjemce dostane přesně tyto a nic jiného.';

  @override
  String get sendStrongCode => 'Silnější kód';

  @override
  String get sendStrongCodeHint =>
      'Tři slova místo dvou: mnohem těžší uhodnout';

  @override
  String get sendCreateCode => 'Vytvořit kód';

  @override
  String get sendCounting => 'Počítání…';

  @override
  String get sendRemove => 'Odebrat';

  @override
  String get sendPreparing => 'Příprava souborů…';

  @override
  String get sendYourCode => 'Váš kód';

  @override
  String get sendCodeHint =>
      'Na druhém zařízení otevřete TotalDrop, klepněte na Přijmout a zadejte tento kód.';

  @override
  String get sendWaiting => 'Čekání na příjemce…';

  @override
  String get sendCopy => 'Kopírovat';

  @override
  String get sendCopied => 'Kód zkopírován';

  @override
  String get sendShare => 'Sdílet';

  @override
  String sendShareText(String code) {
    return 'Přijmi moje soubory v TotalDrop. Kód: $code';
  }

  @override
  String get sendSending => 'Odesílání';

  @override
  String get sendDone => 'Odesláno a ověřeno';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Všech $countString souborů dorazilo v pořádku.',
      many: 'Všech $countString souboru dorazilo v pořádku.',
      few: 'Všechny $countString soubory dorazily v pořádku.',
      one: 'Soubor dorazil v pořádku.',
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
      other: '$countString souborů nedorazilo v pořádku a bylo zahozeno.',
      many: '$countString souboru nedorazilo v pořádku a bylo zahozeno.',
      few: '$countString soubory nedorazily v pořádku a byly zahozeny.',
      one: 'Jeden soubor nedorazil v pořádku a byl zahozen.',
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
      other: '$countString položek',
      many: '$countString položky',
      few: '$countString položky',
      one: '$countString položka',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Přijmout soubory';

  @override
  String get receiveCodeLabel => 'Kód';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Požádejte odesílatele o kód.';

  @override
  String get receiveCodeInvalid =>
      'Tohle nevypadá jako kód. Kód začíná číslem, za kterým následují dvě nebo tři slova.';

  @override
  String get receivePaste => 'Vložit';

  @override
  String get receiveConnect => 'Připojit';

  @override
  String get receiveConnecting => 'Navazování zabezpečeného spojení…';

  @override
  String get receiveOffer => 'Příchozí soubory';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString souborů',
      many: '$filesString souboru',
      few: '$filesString soubory',
      one: '$filesString soubor',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'a $countString dalších';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Uloženo do $folder';
  }

  @override
  String get receiveAccept => 'Přijmout';

  @override
  String get receiveDecline => 'Odmítnout';

  @override
  String get receiveDeclined => 'Soubory jste odmítli.';

  @override
  String get receiveReceiving => 'Přijímání';

  @override
  String get receiveDone => 'Přijato a ověřeno';

  @override
  String get verificationTitle => 'Ověřovací slova';

  @override
  String get verificationHint =>
      'Druhá osoba vidí stejná čtyři slova. Pokud se liší, přestaňte.';

  @override
  String get sameNetworkNote =>
      'Funguje přes internet. Když se zařízení nemohou spojit přímo, šifrovaný přenos jde přes zprostředkující server, který ho nemůže přečíst.';

  @override
  String get done => 'Hotovo';

  @override
  String get tryAgain => 'Zkusit znovu';

  @override
  String get cancel => 'Zrušit';

  @override
  String get cancelTransferTitle => 'Zastavit tento přenos?';

  @override
  String get cancelTransferBody =>
      'Druhá strana se dozví, že byl přenos zastaven.';

  @override
  String get keepGoing => 'Pokračovat';

  @override
  String get stop => 'Zastavit';

  @override
  String get moreInfo => 'Podrobnosti';

  @override
  String get errorWrongCode =>
      'Kód nesouhlasil. Byl chybně zadán, nebo se někdo pokusil přenos zachytit. Nic nebylo odesláno. Začněte znovu s novým kódem.';

  @override
  String get errorPeerLeft => 'Druhá strana odešla před dokončením přenosu.';

  @override
  String get errorDeclined => 'Příjemce soubory odmítl.';

  @override
  String get errorCancelled => 'Druhá strana přenos zastavila.';

  @override
  String get errorStopped => 'Zastaveno na tomto zařízení.';

  @override
  String get errorInvalidCode => 'Tento kód není platný.';

  @override
  String get errorServer =>
      'Nelze se spojit se serverem TotalDrop. Zkontrolujte připojení k internetu.';

  @override
  String get errorCodeNotInUse =>
      'Tento kód se nepoužívá: byl chybně zadán, vypršel nebo už byl použit. Požádejte odesílatele o nový.';

  @override
  String get errorCodeExpired =>
      'Kód vypršel dřív, než ho někdo použil. Pro nový kód odešlete soubory znovu.';

  @override
  String get errorBadServer =>
      'Adresa serveru není platná. Musí začínat wss://.';

  @override
  String get errorConnect =>
      'Nelze se připojit k druhému zařízení. Zkontrolujte, že jsou obě zařízení online, a zkuste to znovu.';

  @override
  String get errorInvalidOffer =>
      'Nabídka porušila bezpečnostní pravidlo, a proto byla odmítnuta.';

  @override
  String get errorStorage => 'Soubor nelze přečíst nebo uložit.';

  @override
  String get errorOther => 'Něco se pokazilo.';

  @override
  String get settingsServer => 'Server pro navázání spojení';

  @override
  String get settingsServerHelp =>
      'Pomáhá dvěma zařízením se stejným kódem najít se navzájem. Nikdy nevidí vaše soubory ani klíče.';

  @override
  String get settingsServerReset => 'Použít výchozí';

  @override
  String get settingsServerInvalid => 'Musí začínat wss://';

  @override
  String get settingsSave => 'Uložit';

  @override
  String get settingsAbout => 'O aplikaci';

  @override
  String get settingsAboutBody =>
      'TotalDrop je zdarma a má otevřený zdrojový kód, financují ho dary. Licence GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Soukromí';

  @override
  String get settingsPrivacyBody =>
      'Server pro navázání spojení, zprostředkující server a STUN server Cloudflare vidí vaši IP adresu, stejně jako druhé zařízení při přímém spojení (pokud není zapnutý režim soukromí). Žádný z nich nevidí názvy souborů, jejich obsah ani klíče a naše servery nevedou záznamy. Soubory jdou přímo mezi zařízeními, když to jde, jinak přes zprostředkující server; v obou případech jsou koncově šifrované.';

  @override
  String get privacyMode => 'Skrýt mou adresu před druhým zařízením';

  @override
  String get privacyModeHint =>
      'Režim soukromí: vše jde přes zprostředkující server, takže druhé zařízení se vaši IP adresu nikdy nedozví. Je to pomalejší a druhé zařízení musí používat stejný zprostředkující server (výchozí to splňuje).';

  @override
  String get privacyModeConfirmTitle => 'Zapnout režim soukromí?';

  @override
  String get privacyModeConfirmBody =>
      'Přenosy budou pomalejší, protože nic nepůjde přímo mezi zařízeními. Server pro navázání spojení a zprostředkující server stále vidí vaši IP adresu; vaše soubory ani klíče nikdy nevidí.';

  @override
  String get privacyModeTurnOn => 'Zapnout';

  @override
  String settingsVersion(String version) {
    return 'Verze $version';
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
      'Tento překlad se kontroluje. Pomozte ho vylepšit na GitHubu.';

  @override
  String get sendDropHere => 'Přetáhněte soubory sem';

  @override
  String get sendDropHint => 'Soubory můžete také přetáhnout do tohoto okna.';

  @override
  String get sendQrHint =>
      'Nebo tento kód naskenujte fotoaparátem druhého zařízení.';

  @override
  String get sendWaitingAccept =>
      'Zkontrolujte slova a počkejte, až příjemce soubory přijme.';

  @override
  String get receiveFromImage => 'Načíst QR kód z obrázku';

  @override
  String get receiveQrNotFound => 'V tomto obrázku není kód TotalDrop.';

  @override
  String get pickTitle => 'Vyberte soubory k odeslání';

  @override
  String get pickImageTitle => 'Vyberte obrázek s QR kódem';

  @override
  String get openFolder => 'Otevřít složku';

  @override
  String get back => 'Zpět';

  @override
  String get codeSuggestions => 'Návrhy';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString %';
  }
}
