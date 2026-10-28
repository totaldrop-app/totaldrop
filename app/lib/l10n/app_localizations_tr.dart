// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Turkish (`tr`).
class AppLocalizationsTr extends AppLocalizations {
  AppLocalizationsTr([String locale = 'tr']) : super(locale);

  @override
  String get appTagline => 'Her dosyayı, herkese, her yere.';

  @override
  String get homeSendTitle => 'Gönder';

  @override
  String get homeSendBody => 'Dosyaları seç, kısa bir kod paylaş';

  @override
  String get homeReceiveTitle => 'Al';

  @override
  String get homeReceiveBody => 'Kodu gir, dosyaları kaydet';

  @override
  String get homeFooter =>
      'Uçtan uca şifreli. Hesap, e-posta, telefon numarası yok.';

  @override
  String get settings => 'Ayarlar';

  @override
  String get sendTitle => 'Dosya gönder';

  @override
  String get sendAddFiles => 'Dosya ekle';

  @override
  String get sendEmpty => 'Henüz bir şey seçilmedi';

  @override
  String get sendEmptyHint =>
      'Göndermek istediğiniz dosyaları ekleyin. Alıcı yalnızca bunları alır, başka hiçbir şeyi değil.';

  @override
  String get sendStrongCode => 'Daha güçlü kod';

  @override
  String get sendStrongCodeHint =>
      'İki yerine üç kelime: tahmin etmesi çok daha zor';

  @override
  String get sendCreateCode => 'Kod oluştur';

  @override
  String get sendCounting => 'Hesaplanıyor…';

  @override
  String get sendRemove => 'Kaldır';

  @override
  String get sendPreparing => 'Dosyalarınız hazırlanıyor…';

  @override
  String get sendYourCode => 'Kodunuz';

  @override
  String get sendCodeHint =>
      'Diğer cihazda TotalDrop\'u açın, Al\'a dokunun ve bu kodu girin.';

  @override
  String get sendWaiting => 'Alıcı bekleniyor…';

  @override
  String get sendCopy => 'Kopyala';

  @override
  String get sendCopied => 'Kod kopyalandı';

  @override
  String get sendShare => 'Paylaş';

  @override
  String sendShareText(String code) {
    return 'Dosyalarımı TotalDrop ile al. Kod: $code';
  }

  @override
  String get sendSending => 'Gönderiliyor';

  @override
  String get sendDone => 'Gönderildi ve doğrulandı';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$countString dosyanın hepsi eksiksiz ulaştı.',
      one: 'Dosya eksiksiz ulaştı.',
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
      other: '$countString dosya eksiksiz ulaşmadı ve silindi.',
      one: 'Bir dosya eksiksiz ulaşmadı ve silindi.',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '$countString öğe · $size';
  }

  @override
  String get receiveTitle => 'Dosya al';

  @override
  String get receiveCodeLabel => 'Kod';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Gönderenden kodunu isteyin.';

  @override
  String get receiveCodeInvalid =>
      'Bu bir koda benzemiyor. Kod bir sayıyla başlar, ardından iki ya da üç kelime gelir.';

  @override
  String get receivePaste => 'Yapıştır';

  @override
  String get receiveConnect => 'Bağlan';

  @override
  String get receiveConnecting => 'Güvenli bağlantı kuruluyor…';

  @override
  String get receiveOffer => 'Gelen dosyalar';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesString dosya · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 've $countString tane daha';
  }

  @override
  String receiveSaveTo(String folder) {
    return '$folder klasörüne kaydedildi';
  }

  @override
  String get receiveAccept => 'Kabul et';

  @override
  String get receiveDecline => 'Reddet';

  @override
  String get receiveDeclined => 'Dosyaları reddettiniz.';

  @override
  String get receiveReceiving => 'Alınıyor';

  @override
  String get receiveDone => 'Alındı ve doğrulandı';

  @override
  String get verificationTitle => 'Doğrulama kelimeleri';

  @override
  String get verificationHint =>
      'Karşı taraf da aynı dört kelimeyi görür. Farklıysa durun.';

  @override
  String get sameNetworkNote =>
      'İnternet üzerinden çalışır. Cihazlar birbirine doğrudan ulaşamazsa şifreli aktarım, içeriği okuyamayan aktarma sunucusundan geçer.';

  @override
  String get done => 'Tamam';

  @override
  String get tryAgain => 'Tekrar dene';

  @override
  String get cancel => 'Vazgeç';

  @override
  String get cancelTransferTitle => 'Bu gönderim durdurulsun mu?';

  @override
  String get cancelTransferBody =>
      'Karşı tarafa gönderimin durduğu bildirilir.';

  @override
  String get keepGoing => 'Devam et';

  @override
  String get stop => 'Durdur';

  @override
  String get moreInfo => 'Ayrıntılar';

  @override
  String get errorWrongCode =>
      'Kod eşleşmedi. Ya yanlış yazıldı ya da biri gönderimi araya girerek almaya çalıştı. Hiçbir şey gönderilmedi. Yeni bir kodla yeniden başlayın.';

  @override
  String get errorPeerLeft => 'Karşı taraf gönderim bitmeden ayrıldı.';

  @override
  String get errorDeclined => 'Alıcı dosyaları reddetti.';

  @override
  String get errorCancelled => 'Karşı taraf gönderimi durdurdu.';

  @override
  String get errorStopped => 'Bu cihazda durduruldu.';

  @override
  String get errorInvalidCode => 'Bu kod geçerli değil.';

  @override
  String get errorServer =>
      'TotalDrop sunucusuna ulaşılamadı. İnternet bağlantınızı kontrol edin.';

  @override
  String get errorConnect =>
      'Diğer cihaza bağlanılamadı. İki cihazın da internete bağlı olduğundan emin olup yeniden deneyin.';

  @override
  String get errorInvalidOffer =>
      'Teklif bir güvenlik kuralını çiğnediği için reddedildi.';

  @override
  String get errorStorage => 'Bir dosya okunamadı ya da kaydedilemedi.';

  @override
  String get errorOther => 'Bir şeyler ters gitti.';

  @override
  String get settingsServer => 'Buluşma sunucusu';

  @override
  String get settingsServerHelp =>
      'Aynı kodu bilen iki cihazın birbirini bulmasını sağlar. Dosyalarınızı ve anahtarlarınızı asla görmez.';

  @override
  String get settingsServerReset => 'Varsayılanı kullan';

  @override
  String get settingsServerInvalid => 'wss:// ile başlamalı';

  @override
  String get settingsSave => 'Kaydet';

  @override
  String get settingsAbout => 'Hakkında';

  @override
  String get settingsAboutBody =>
      'TotalDrop ücretsiz ve açık kaynaktır, bağışlarla yürür. Lisansı GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Gizlilik';

  @override
  String get settingsPrivacyBody =>
      'Buluşma sunucusu, aktarma sunucusu ve Cloudflare\'in STUN sunucusu IP adresinizi görür; doğrudan bağlandığınızda karşı cihaz da görür. Hiçbiri dosya adlarını, içerikleri ya da anahtarları görmez ve sunucularımız kayıt tutmaz. Dosyalar mümkünse cihazlar arasında doğrudan, değilse aktarma sunucusu üzerinden gider; her iki durumda da uçtan uca şifrelidir.';

  @override
  String get privacyMode => 'Adresimi karşı cihazdan gizle';

  @override
  String get privacyModeHint =>
      'Gizlilik modu: her şey aktarma sunucusundan geçer, böylece karşı cihaz IP adresinizi hiç öğrenmez. Daha yavaştır ve iki cihazın da aynı aktarma sunucusunu kullanması gerekir.';

  @override
  String get privacyModeConfirmTitle => 'Gizlilik modu açılsın mı?';

  @override
  String get privacyModeConfirmBody =>
      'Aktarımlar yavaşlar, çünkü cihazlar arasında doğrudan hiçbir şey geçmez. Aktarma sunucusu IP adresinizi görür; dosyalarınızı ya da anahtarlarınızı hiç görmez.';

  @override
  String get privacyModeTurnOn => 'Aç';

  @override
  String settingsVersion(String version) {
    return 'Sürüm $version';
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
      'Bu çeviri gözden geçiriliyor. Weblate\'te iyileştirmeye yardım edin.';

  @override
  String get sendDropHere => 'Dosyaları buraya bırakın';

  @override
  String get sendDropHint =>
      'Dosyaları bu pencereye sürükleyip de ekleyebilirsiniz.';

  @override
  String get sendQrHint => 'Ya da bu kodu diğer cihazın kamerasıyla okutun.';

  @override
  String get sendWaitingAccept =>
      'Kelimeleri karşılaştırın, ardından alıcının kabul etmesini bekleyin.';

  @override
  String get receiveFromImage => 'Resimden QR oku';

  @override
  String get receiveQrNotFound => 'Bu resimde TotalDrop kodu yok.';

  @override
  String get pickTitle => 'Gönderilecek dosyaları seçin';

  @override
  String get pickImageTitle => 'QR kodlu bir resim seçin';

  @override
  String get openFolder => 'Klasörü aç';

  @override
  String get back => 'Geri';

  @override
  String get codeSuggestions => 'Öneriler';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '%$percentString';
  }
}
