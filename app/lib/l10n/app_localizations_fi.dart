// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Finnish (`fi`).
class AppLocalizationsFi extends AppLocalizations {
  AppLocalizationsFi([String locale = 'fi']) : super(locale);

  @override
  String get appTagline =>
      'Mikä tahansa tiedosto, kenelle tahansa, minne tahansa.';

  @override
  String get homeSendTitle => 'Lähetä';

  @override
  String get homeSendBody => 'Valitse tiedostot ja jaa lyhyt koodi';

  @override
  String get homeReceiveTitle => 'Vastaanota';

  @override
  String get homeReceiveBody => 'Anna koodi ja tallenna tiedostot';

  @override
  String get homeFooter =>
      'Päästä päähän salattu. Ei tiliä, ei sähköpostia, ei puhelinnumeroa.';

  @override
  String get settings => 'Asetukset';

  @override
  String get sendTitle => 'Lähetä tiedostoja';

  @override
  String get sendAddFiles => 'Lisää tiedostoja';

  @override
  String get sendEmpty => 'Mitään ei ole vielä valittu';

  @override
  String get sendEmptyHint =>
      'Lisää tiedostot, jotka haluat lähettää. Vastaanottaja saa juuri nämä, ei mitään muuta.';

  @override
  String get sendStrongCode => 'Vahvempi koodi';

  @override
  String get sendStrongCodeHint =>
      'Kolme sanaa kahden sijaan: paljon vaikeampi arvata';

  @override
  String get sendCreateCode => 'Luo koodi';

  @override
  String get sendCounting => 'Lasketaan…';

  @override
  String get sendRemove => 'Poista';

  @override
  String get sendPreparing => 'Valmistellaan tiedostoja…';

  @override
  String get sendYourCode => 'Koodisi';

  @override
  String get sendCodeHint =>
      'Avaa toisella laitteella TotalDrop, napauta Vastaanota ja anna tämä koodi.';

  @override
  String get sendWaiting => 'Odotetaan vastaanottajaa…';

  @override
  String get sendCopy => 'Kopioi';

  @override
  String get sendCopied => 'Koodi kopioitu';

  @override
  String get sendShare => 'Jaa';

  @override
  String sendShareText(String code) {
    return 'Hae tiedostoni TotalDropilla. Koodi: $code';
  }

  @override
  String get sendSending => 'Lähetetään';

  @override
  String get sendDone => 'Lähetetty ja tarkistettu';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Kaikki $countString tiedostoa saapuivat ehjinä.',
      one: 'Tiedosto saapui ehjänä.',
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
      other: '$countString tiedostoa ei saapunut ehjinä, ja ne hylättiin.',
      one: 'Yksi tiedosto ei saapunut ehjänä, ja se hylättiin.',
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
      other: '$countString kohdetta',
      one: '$countString kohde',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Vastaanota tiedostoja';

  @override
  String get receiveCodeLabel => 'Koodi';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Pyydä koodi lähettäjältä.';

  @override
  String get receiveCodeInvalid =>
      'Tämä ei näytä koodilta. Koodi alkaa numerolla, jota seuraa kaksi tai kolme sanaa.';

  @override
  String get receivePaste => 'Liitä';

  @override
  String get receiveConnect => 'Yhdistä';

  @override
  String get receiveConnecting => 'Muodostetaan suojattua yhteyttä…';

  @override
  String get receiveOffer => 'Saapuvat tiedostot';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString tiedostoa',
      one: '$filesString tiedosto',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'ja $countString muuta';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Tallennetaan kansioon $folder';
  }

  @override
  String get receiveAccept => 'Hyväksy';

  @override
  String get receiveDecline => 'Hylkää';

  @override
  String get receiveDeclined => 'Hylkäsit tiedostot.';

  @override
  String get receiveReceiving => 'Vastaanotetaan';

  @override
  String get receiveDone => 'Vastaanotettu ja tarkistettu';

  @override
  String get verificationTitle => 'Tarkistussanat';

  @override
  String get verificationHint =>
      'Toinen henkilö näkee samat neljä sanaa. Jos ne eroavat, keskeytä.';

  @override
  String get sameNetworkNote =>
      'Toimii internetin yli. Kun laitteet eivät tavoita toisiaan suoraan, salattu siirto kulkee välityspalvelimen kautta, joka ei voi lukea sitä.';

  @override
  String get done => 'Valmis';

  @override
  String get tryAgain => 'Yritä uudelleen';

  @override
  String get cancel => 'Peruuta';

  @override
  String get cancelTransferTitle => 'Lopetetaanko siirto?';

  @override
  String get cancelTransferBody =>
      'Toiselle osapuolelle kerrotaan, että siirto lopetettiin.';

  @override
  String get keepGoing => 'Jatka';

  @override
  String get stop => 'Lopeta';

  @override
  String get moreInfo => 'Lisätiedot';

  @override
  String get errorWrongCode =>
      'Koodi ei täsmännyt. Se kirjoitettiin väärin, tai joku yritti kaapata siirron. Mitään ei lähetetty. Aloita alusta uudella koodilla.';

  @override
  String get errorPeerLeft =>
      'Toinen osapuoli poistui ennen kuin siirto valmistui.';

  @override
  String get errorDeclined => 'Vastaanottaja hylkäsi tiedostot.';

  @override
  String get errorCancelled => 'Toinen osapuoli lopetti siirron.';

  @override
  String get errorStopped => 'Lopetettu tällä laitteella.';

  @override
  String get errorInvalidCode => 'Koodi ei ole kelvollinen.';

  @override
  String get errorServer =>
      'TotalDrop-palvelimeen ei saatu yhteyttä. Tarkista internetyhteys.';

  @override
  String get errorConnect =>
      'Toiseen laitteeseen ei saatu yhteyttä. Varmista, että molemmat laitteet ovat verkossa, ja yritä uudelleen.';

  @override
  String get errorInvalidOffer =>
      'Tarjous rikkoi turvasääntöä, joten se hylättiin.';

  @override
  String get errorStorage => 'Tiedostoa ei voitu lukea tai tallentaa.';

  @override
  String get errorOther => 'Jokin meni vikaan.';

  @override
  String get settingsServer => 'Kohtauspalvelin';

  @override
  String get settingsServerHelp =>
      'Auttaa kahta saman koodin laitetta löytämään toisensa. Se ei koskaan näe tiedostojasi tai avaimiasi.';

  @override
  String get settingsServerReset => 'Käytä oletusta';

  @override
  String get settingsServerInvalid => 'Alun on oltava wss://';

  @override
  String get settingsSave => 'Tallenna';

  @override
  String get settingsAbout => 'Tietoja';

  @override
  String get settingsAboutBody =>
      'TotalDrop on ilmainen ja avointa lähdekoodia, ja se rahoitetaan lahjoituksilla. Lisenssi: GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Yksityisyys';

  @override
  String get settingsPrivacyBody =>
      'Kohtauspalvelin, välityspalvelin ja Cloudflaren STUN-palvelin näkevät IP-osoitteesi, ja niin näkee myös toinen laite, kun yhdistätte suoraan (ellei yksityisyystila ole käytössä). Mikään niistä ei näe tiedostojen nimiä, sisältöä tai avaimia, eivätkä palvelimemme tallenna lokeja. Tiedostot kulkevat suoraan laitteiden välillä aina kun mahdollista, muuten välityspalvelimen kautta; ne ovat päästä päähän salattuja kummassakin tapauksessa.';

  @override
  String get privacyMode => 'Piilota osoitteeni toiselta laitteelta';

  @override
  String get privacyModeHint =>
      'Yksityisyystila: kaikki kulkee välityspalvelimen kautta, joten toinen laite ei koskaan saa tietää IP-osoitettasi. Hitaampi, ja toisen laitteen on käytettävä samaa välityspalvelinta (oletuspalvelin käyttää).';

  @override
  String get privacyModeConfirmTitle => 'Otetaanko yksityisyystila käyttöön?';

  @override
  String get privacyModeConfirmBody =>
      'Siirrot hidastuvat, koska mikään ei kulje suoraan laitteiden välillä. Kohtauspalvelin ja välityspalvelin näkevät edelleen IP-osoitteesi; ne eivät koskaan näe tiedostojasi tai avaimiasi.';

  @override
  String get privacyModeTurnOn => 'Ota käyttöön';

  @override
  String settingsVersion(String version) {
    return 'Versio $version';
  }

  @override
  String sizeBytes(String value) {
    return '$value t';
  }

  @override
  String sizeKB(String value) {
    return '$value kt';
  }

  @override
  String sizeMB(String value) {
    return '$value Mt';
  }

  @override
  String sizeGB(String value) {
    return '$value Gt';
  }

  @override
  String sizeTB(String value) {
    return '$value Tt';
  }

  @override
  String sizePB(String value) {
    return '$value Pt';
  }

  @override
  String get draftTranslation =>
      'Tätä käännöstä tarkistetaan. Auta parantamaan sitä GitHubissa.';

  @override
  String get sendDropHere => 'Pudota tiedostot tähän';

  @override
  String get sendDropHint => 'Voit myös vetää tiedostoja tähän ikkunaan.';

  @override
  String get sendQrHint => 'Tai skannaa koodi toisen laitteen kameralla.';

  @override
  String get sendWaitingAccept =>
      'Tarkista sanat ja odota, että vastaanottaja hyväksyy.';

  @override
  String get receiveFromImage => 'Lue QR-koodi kuvasta';

  @override
  String get receiveQrNotFound => 'Kuvassa ei ole TotalDrop-koodia.';

  @override
  String get pickTitle => 'Valitse lähetettävät tiedostot';

  @override
  String get pickImageTitle => 'Valitse kuva, jossa on QR-koodi';

  @override
  String get openFolder => 'Avaa kansio';

  @override
  String get back => 'Takaisin';

  @override
  String get codeSuggestions => 'Ehdotukset';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString %';
  }
}
