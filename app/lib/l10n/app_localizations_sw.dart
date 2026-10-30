// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Swahili (`sw`).
class AppLocalizationsSw extends AppLocalizations {
  AppLocalizationsSw([String locale = 'sw']) : super(locale);

  @override
  String get appTagline => 'Faili yoyote, kwa yeyote, popote.';

  @override
  String get homeSendTitle => 'Tuma';

  @override
  String get homeSendBody => 'Chagua faili na ushiriki msimbo mfupi';

  @override
  String get homeReceiveTitle => 'Pokea';

  @override
  String get homeReceiveBody => 'Weka msimbo na uhifadhi faili';

  @override
  String get homeFooter =>
      'Imesimbwa fiche kutoka mwanzo hadi mwisho. Hakuna akaunti, barua pepe wala nambari ya simu.';

  @override
  String get settings => 'Mipangilio';

  @override
  String get sendTitle => 'Tuma faili';

  @override
  String get sendAddFiles => 'Ongeza faili';

  @override
  String get sendEmpty => 'Bado hujachagua chochote';

  @override
  String get sendEmptyHint =>
      'Ongeza faili unazotaka kutuma. Mpokeaji anapata hizi tu, hakuna kingine.';

  @override
  String get sendStrongCode => 'Msimbo imara zaidi';

  @override
  String get sendStrongCodeHint =>
      'Maneno matatu badala ya mawili: ni vigumu zaidi kukisia';

  @override
  String get sendCreateCode => 'Unda msimbo';

  @override
  String get sendCounting => 'Inahesabu…';

  @override
  String get sendRemove => 'Ondoa';

  @override
  String get sendPreparing => 'Inaandaa faili zako…';

  @override
  String get sendYourCode => 'Msimbo wako';

  @override
  String get sendCodeHint =>
      'Kwenye kifaa kingine, fungua TotalDrop, gusa Pokea kisha uweke msimbo huu.';

  @override
  String get sendWaiting => 'Inamsubiri mpokeaji…';

  @override
  String get sendCopy => 'Nakili';

  @override
  String get sendCopied => 'Msimbo umenakiliwa';

  @override
  String get sendShare => 'Shiriki';

  @override
  String sendShareText(String code) {
    return 'Pokea faili zangu kwa TotalDrop. Msimbo: $code';
  }

  @override
  String get sendSending => 'Inatuma';

  @override
  String get sendDone => 'Imetumwa na kuthibitishwa';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Faili zote $countString zimefika zikiwa salama.',
      one: 'Faili imefika ikiwa salama.',
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
      other: 'Faili $countString hazikufika zikiwa salama na zimetupwa.',
      one: 'Faili moja haikufika ikiwa salama na imetupwa.',
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
      other: 'vipengee $countString',
      one: 'kipengee $countString',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Pokea faili';

  @override
  String get receiveCodeLabel => 'Msimbo';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Mwombe mtumaji msimbo wake.';

  @override
  String get receiveCodeInvalid =>
      'Hii haionekani kama msimbo. Msimbo huanza na nambari, kisha maneno mawili au matatu.';

  @override
  String get receivePaste => 'Bandika';

  @override
  String get receiveConnect => 'Unganisha';

  @override
  String get receiveConnecting => 'Inaunganisha kwa usalama…';

  @override
  String get receiveOffer => 'Faili zinazoingia';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return 'faili $filesString · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'na $countString zaidi';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Zimehifadhiwa kwenye $folder';
  }

  @override
  String get receiveAccept => 'Kubali';

  @override
  String get receiveDecline => 'Kataa';

  @override
  String get receiveDeclined => 'Umekataa faili.';

  @override
  String get receiveReceiving => 'Inapokea';

  @override
  String get receiveDone => 'Imepokelewa na kuthibitishwa';

  @override
  String get verificationTitle => 'Maneno ya uthibitisho';

  @override
  String get verificationHint =>
      'Mtu mwingine anaona maneno manne yale yale. Yakitofautiana, simama.';

  @override
  String get sameNetworkNote =>
      'Inafanya kazi kupitia intaneti. Vifaa visipoweza kufikiana moja kwa moja, uhamishaji uliosimbwa fiche hupita kupitia seva ya kati, ambayo haiwezi kuusoma.';

  @override
  String get done => 'Nimemaliza';

  @override
  String get tryAgain => 'Jaribu tena';

  @override
  String get cancel => 'Ghairi';

  @override
  String get cancelTransferTitle => 'Simamisha uhamishaji huu?';

  @override
  String get cancelTransferBody =>
      'Upande mwingine utaarifiwa kuwa uhamishaji umesimamishwa.';

  @override
  String get keepGoing => 'Endelea';

  @override
  String get stop => 'Simamisha';

  @override
  String get moreInfo => 'Maelezo';

  @override
  String get errorWrongCode =>
      'Msimbo haukulingana. Ulikosewa kuandikwa, au mtu alijaribu kunasa uhamishaji. Hakuna kilichotumwa. Anza tena kwa msimbo mpya.';

  @override
  String get errorPeerLeft =>
      'Upande mwingine uliondoka kabla uhamishaji haujakamilika.';

  @override
  String get errorDeclined => 'Mpokeaji amekataa faili.';

  @override
  String get errorCancelled => 'Upande mwingine umesimamisha uhamishaji.';

  @override
  String get errorStopped => 'Imesimamishwa kwenye kifaa hiki.';

  @override
  String get errorInvalidCode => 'Msimbo huo si sahihi.';

  @override
  String get errorServer =>
      'Imeshindwa kufikia seva ya TotalDrop. Angalia muunganisho wako wa intaneti.';

  @override
  String get errorCodeNotInUse =>
      'Msimbo huo hautumiki: ulikosewa kuandikwa, umeisha muda au tayari umetumika. Mwombe mtumaji msimbo mpya.';

  @override
  String get errorConnect =>
      'Imeshindwa kuunganisha na kifaa kingine. Hakikisha vifaa vyote viwili viko mtandaoni kisha ujaribu tena.';

  @override
  String get errorInvalidOffer =>
      'Ofa ilivunja kanuni ya usalama, kwa hiyo imekataliwa.';

  @override
  String get errorStorage => 'Faili moja haikuweza kusomwa au kuhifadhiwa.';

  @override
  String get errorOther => 'Hitilafu fulani imetokea.';

  @override
  String get settingsServer => 'Seva ya kukutanisha';

  @override
  String get settingsServerHelp =>
      'Husaidia vifaa viwili vyenye msimbo mmoja kupatana. Haioni kamwe faili au funguo zako.';

  @override
  String get settingsServerReset => 'Tumia chaguomsingi';

  @override
  String get settingsServerInvalid => 'Lazima ianze na wss://';

  @override
  String get settingsSave => 'Hifadhi';

  @override
  String get settingsAbout => 'Kuhusu';

  @override
  String get settingsAboutBody =>
      'TotalDrop ni bure na ya chanzo huria, inayofadhiliwa kwa michango. Leseni GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Faragha';

  @override
  String get settingsPrivacyBody =>
      'Seva ya kukutanisha, seva ya kati na seva ya STUN ya Cloudflare huona anwani yako ya IP, na kifaa kingine pia huiona mnapounganishwa moja kwa moja (isipokuwa hali ya faragha imewashwa). Hakuna kati yao anayeona majina ya faili, maudhui au funguo, na seva zetu hazihifadhi kumbukumbu. Faili husafiri moja kwa moja kati ya vifaa inapowezekana, vinginevyo kupitia seva ya kati; kwa njia zote mbili zimesimbwa fiche kutoka mwanzo hadi mwisho.';

  @override
  String get privacyMode => 'Ficha anwani yangu kwa kifaa kingine';

  @override
  String get privacyModeHint =>
      'Hali ya faragha: kila kitu hupita kupitia seva ya kati, kwa hiyo kifaa kingine hakijui kamwe anwani yako ya IP. Ni polepole zaidi, na kifaa kingine lazima kitumie seva ya kati ile ile (ile ya chaguomsingi inafaa).';

  @override
  String get privacyModeConfirmTitle => 'Washa hali ya faragha?';

  @override
  String get privacyModeConfirmBody =>
      'Uhamishaji utakuwa wa polepole zaidi, kwa sababu hakuna kinachopita moja kwa moja kati ya vifaa. Seva ya kukutanisha na seva ya kati bado huona anwani yako ya IP; hazioni kamwe faili au funguo zako.';

  @override
  String get privacyModeTurnOn => 'Washa';

  @override
  String settingsVersion(String version) {
    return 'Toleo $version';
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
      'Tafsiri hii inakaguliwa. Saidia kuiboresha kwenye GitHub.';

  @override
  String get sendDropHere => 'Dondosha faili hapa';

  @override
  String get sendDropHint =>
      'Unaweza pia kuburuta faili hadi kwenye dirisha hili.';

  @override
  String get sendQrHint =>
      'Au changanua msimbo huu kwa kamera ya kifaa kingine.';

  @override
  String get sendWaitingAccept =>
      'Linganisha maneno, kisha subiri mpokeaji akubali.';

  @override
  String get receiveFromImage => 'Soma QR kutoka kwenye picha';

  @override
  String get receiveQrNotFound =>
      'Hakuna msimbo wa TotalDrop kwenye picha hiyo.';

  @override
  String get pickTitle => 'Chagua faili za kutuma';

  @override
  String get pickImageTitle => 'Chagua picha yenye msimbo wa QR';

  @override
  String get openFolder => 'Fungua folda';

  @override
  String get back => 'Rudi';

  @override
  String get codeSuggestions => 'Mapendekezo';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
