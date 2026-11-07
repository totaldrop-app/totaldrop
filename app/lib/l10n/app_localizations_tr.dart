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
  String get sendAddFailed =>
      'Dosyalar eklenemedi. Yeniden deneyin. Yine olursa bu cihazda yeterli boş alan olduğuna bakın ya da dosyaları başka bir yerden seçin.';

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
      'Kod eşleşmedi. Ya yanlış yazıldı ya da biri gönderimi ele geçirmeye çalıştı. Hiçbir şey gönderilmedi. Yeni bir kodla yeniden başlayın.';

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
  String get errorCodeNotInUse =>
      'Bu kod kullanımda değil: yanlış yazılmış, süresi dolmuş ya da zaten kullanılmış olabilir. Gönderenden yeni bir kod isteyin.';

  @override
  String get errorCodeExpired =>
      'Kodun süresi kimse kullanmadan doldu. Yeni bir kod almak için yeniden gönderin.';

  @override
  String get errorBadServer =>
      'Sunucu adresi geçerli değil. wss:// ile başlamalı.';

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
  String get causeTimedOut =>
      'Bağlantı artık yanıt vermiyor. Cihazlardan biri ağ bağlantısını kaybetmiş ya da uykuya geçmiş olabilir.';

  @override
  String get causeConnectionLost => 'Diğer cihazla bağlantı koptu.';

  @override
  String get causeNoRoute =>
      'Diğer cihaza ne doğrudan ne de aktarma sunucusu üzerinden ulaşılabildi.';

  @override
  String get causeRelay =>
      'Aktarma sunucusuna ulaşılamadı ya da ayarlı değil, veya iki cihaz aynı aktarma sunucusunu kullanmıyor.';

  @override
  String get causeProtocol =>
      'Diğer cihaz bu sürümün anlamadığı bir şey gönderdi. İki cihazda da TotalDrop\'u güncellemek işe yarayabilir.';

  @override
  String get causeDiskFull => 'Bu cihazda yeterli boş alan yok.';

  @override
  String get causeNoPermission =>
      'TotalDrop\'un gereken bir dosyayı ya da klasörü okuma veya yazma izni yok.';

  @override
  String get causeNotFound =>
      'Bir dosya ya da klasör bulunamadı. Taşınmış ya da silinmiş olabilir.';

  @override
  String get causeReadOnly => 'Klasör, yazılamayan bir diskte.';

  @override
  String get causeFileChanged =>
      'Bir dosya gönderilirken değişti. Kaydettikten sonra yeniden gönderin.';

  @override
  String get technicalDetail =>
      'Sorun bildirirken eklemeniz için İngilizce teknik ileti:';

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
      'Buluşma sunucusu, aktarma sunucusu ve Cloudflare\'in STUN sunucusu IP adresinizi görür; doğrudan bağlandığınızda (gizlilik modu kapalıysa) karşı cihaz da görür. Hiçbiri dosya adlarını, içerikleri ya da anahtarları görmez ve sunucularımız kayıt tutmaz. Dosyalar mümkünse cihazlar arasında doğrudan, değilse aktarma sunucusu üzerinden gider; her iki durumda da uçtan uca şifrelidir.';

  @override
  String get privacyMode => 'Adresimi karşı cihazdan gizle';

  @override
  String get privacyModeHint =>
      'Gizlilik modu: tüm aktarımlar aktarma sunucusu üzerinden geçer, böylece karşı cihaz IP adresinizi hiç öğrenmez. Daha yavaştır ve karşı cihazın da aynı aktarma sunucusunu kullanması gerekir (varsayılan sunucu aynıdır).';

  @override
  String get privacyModeConfirmTitle => 'Gizlilik modu açılsın mı?';

  @override
  String get privacyModeConfirmBody =>
      'Aktarımlar yavaşlar, çünkü cihazlar arasında doğrudan hiçbir şey geçmez. Buluşma sunucusu ve aktarma sunucusu IP adresinizi yine görür; dosyalarınızı ya da anahtarlarınızı hiç görmez.';

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
      'Bu çeviri gözden geçiriliyor. GitHub\'da iyileştirmeye yardım edin.';

  @override
  String get settingsWordsEnglish =>
      'Kodlar ve doğrulama kelimeleri her dilde İngilizcedir; böylece farklı dillere ayarlı iki cihaz aynı kelimeleri gösterir.';

  @override
  String get sendDropHere => 'Dosyaları buraya bırakın';

  @override
  String get sendDropHint =>
      'Dosyaları bu pencereye sürükleyip de ekleyebilirsiniz.';

  @override
  String get keepAwakeReason => 'TotalDrop dosya gönderiyor ya da alıyor';

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

  @override
  String get editUndo => 'Geri al';

  @override
  String get editRedo => 'Yinele';

  @override
  String get editCut => 'Kes';

  @override
  String get editCopy => 'Kopyala';

  @override
  String get editPaste => 'Yapıştır';

  @override
  String get editDelete => 'Sil';

  @override
  String get editSelectAll => 'Tümünü seç';

  @override
  String get transferNoticeChannel => 'Aktarımlar';

  @override
  String get transferNoticeSending => 'TotalDrop ile gönderiliyor';

  @override
  String get transferNoticeReceiving => 'TotalDrop ile alınıyor';

  @override
  String get transferNoticeBody => 'Başka uygulamaları kullanırken de sürer.';

  @override
  String get shareBusy =>
      'Bir aktarım hâlâ sürüyor. Bitmesini bekleyin ya da durdurun, sonra yeniden paylaşın.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Paylaşılan $count öğe okunamadı ve dışarıda bırakıldı.',
      one: 'Paylaşılan bir öğe okunamadı ve dışarıda bırakıldı.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'Dosya aktarımı';

  @override
  String get desktopComment =>
      'Dosyaları başka bir cihaza doğrudan ve uçtan uca şifreli gönderin';

  @override
  String get desktopKeywords => 'gönder;al;paylaş;aktar;dosya;';

  @override
  String get desktopSendWith => 'TotalDrop ile gönder';

  @override
  String get metainfoSummary => 'Dosyaları doğrudan başka bir cihaza gönderin';

  @override
  String get metainfoAbout =>
      'TotalDrop dosya ve klasörleri bir cihazdan diğerine uçtan uca şifreli gönderir. Dosyaları seçin, kısa bir kodu ya da QR kodunu paylaşın, diğer cihaz dosyaları alsın. Hesap, e-posta, telefon numarası yok.';

  @override
  String get metainfoVerified =>
      'Her dosya ulaştığında doğrulanır; bozuk dosyalar silinir';

  @override
  String get metainfoWords =>
      'Dört doğrulama kelimesi araya kimsenin girmediğini gösterir';

  @override
  String get metainfoTheme =>
      'Masaüstünüzün temasını izler: Plasma, Hyprland, Omarchy, GNOME';

  @override
  String get metainfoOnePackage =>
      'Linux\'ta aynı anda tek paket kullanın. Flatpak ile .pkg, .deb ya da .rpm paketi aynı adı paylaşır; biri açıkken ötekini başlatmak ikinci bir pencere açmaz, açık pencereyi öne getirir.';

  @override
  String get otherPackageFlatpak =>
      'TotalDrop zaten Flatpak\'ten açık, o pencere öne getirildi. Aynı anda tek paket kullanın: bunu açmak için o pencereden çıkın.';

  @override
  String get otherPackageSystem =>
      'TotalDrop zaten .pkg, .deb ya da .rpm paketinden açık, o pencere öne getirildi. Aynı anda tek paket kullanın: bunu açmak için o pencereden çıkın.';

  @override
  String get sendText => 'Metin gönder';

  @override
  String get sendTextHint =>
      'Göndermek istediğiniz metni yazın ya da yapıştırın';

  @override
  String sendTextTooLong(String size) {
    return 'Metin en çok $size olabilir. Daha uzununu dosya olarak gönderin.';
  }

  @override
  String get sendTextDone => 'Metin eksiksiz ulaştı.';

  @override
  String get sendClipboard => 'Panodakini gönder';

  @override
  String get sendClipboardEmpty => 'Panoda metin yok.';

  @override
  String get receiveOfferText => 'Gelen metin';

  @override
  String get receiveOfferTextNote => 'Metin kaydedilmez, burada gösterilir.';

  @override
  String get receiveTextDone => 'Metin alındı ve doğrulandı';

  @override
  String get receiveTextDeclined => 'Metni reddettiniz.';

  @override
  String get receiveTextHidden =>
      'Metinde gizli karakterler var, U+… olarak gösteriliyor. Kopyalayınca metin geldiği gibi kopyalanır.';

  @override
  String get sendTextHidden =>
      'Metinde yön işareti gibi gizli karakterler var. Bunlar da metinle birlikte gönderilir.';

  @override
  String get sendClipboardKept => 'Yazılmış metin korundu, pano kullanılmadı.';

  @override
  String get sendTextShortened =>
      'Metnin yalnızca başı gösteriliyor, tamamı gönderilir.';

  @override
  String get receiveTextShortened =>
      'Metnin yalnızca başı gösteriliyor. Metni kopyala, tamamını kopyalar.';

  @override
  String get copyText => 'Metni kopyala';

  @override
  String get textCopied => 'Metin kopyalandı';

  @override
  String get errorTextDamaged =>
      'Metin eksiksiz ulaşmadı, bu yüzden gösterilmiyor.';

  @override
  String get setupBackgroundTitle => 'Aktarımlar arka planda sürsün';

  @override
  String get setupBackgroundBody =>
      'Uygulama değiştirdiğinizde ya da ekranı kilitlediğinizde aktarımın sürmesi için TotalDrop\'un pil kısıtlaması olmadan çalışmasına izin verin. TotalDrop arka planda yalnızca sizin başlattığınız bir aktarım sürerken çalışır.';

  @override
  String get setupBackgroundAllowed =>
      'Android\'in pil optimizasyonu TotalDrop için kapalı.';

  @override
  String get setupBackgroundRestricted =>
      'Android\'in pil optimizasyonu TotalDrop için açık: uygulama değiştirdiğinizde aktarım durabilir.';

  @override
  String get setupBackgroundBattery => 'Pil ayarlarını aç';

  @override
  String get setupBackgroundBatteryHint =>
      'TotalDrop\'un uygulama bilgileri sayfasında Pil\'i açın ve kısıtlamasız kullanıma izin verin. Bu ayarların adları telefondan telefona değişir.';

  @override
  String setupBackgroundMaker(String maker) {
    return '$maker telefonlarının kendi pil tasarrufu var';
  }

  @override
  String get setupBackgroundMakerBody =>
      'TotalDrop\'un pil tasarrufunu da açıp \"Kısıtlama yok\"u seçin. Telefonda Otomatik başlatma varsa TotalDrop için açın.';

  @override
  String get setupBackgroundMakerOpen => 'TotalDrop\'un pil tasarrufunu aç';

  @override
  String get setupBackgroundAutostart => 'Otomatik başlatmayı aç';

  @override
  String get setupBackgroundSkip => 'Şimdi değil';

  @override
  String get settingsBackground => 'Arka planda aktarım';

  @override
  String get settingsBackgroundOpen => 'Pil ayarları';
}
