// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Malay (`ms`).
class AppLocalizationsMs extends AppLocalizations {
  AppLocalizationsMs([String locale = 'ms']) : super(locale);

  @override
  String get appTagline => 'Sebarang fail, kepada sesiapa, di mana-mana.';

  @override
  String get homeSendTitle => 'Hantar';

  @override
  String get homeSendBody => 'Pilih fail dan kongsi kod pendek';

  @override
  String get homeReceiveTitle => 'Terima';

  @override
  String get homeReceiveBody => 'Masukkan kod dan simpan fail';

  @override
  String get homeFooter =>
      'Disulitkan hujung ke hujung. Tiada akaun, e-mel atau nombor telefon.';

  @override
  String get settings => 'Tetapan';

  @override
  String get sendTitle => 'Hantar fail';

  @override
  String get sendAddFiles => 'Tambah fail';

  @override
  String get sendEmpty => 'Belum ada yang dipilih';

  @override
  String get sendEmptyHint =>
      'Tambah fail yang anda mahu hantar. Penerima hanya mendapat fail ini, tiada yang lain.';

  @override
  String get sendStrongCode => 'Kod lebih kukuh';

  @override
  String get sendStrongCodeHint =>
      'Tiga perkataan, bukan dua: jauh lebih sukar diteka';

  @override
  String get sendCreateCode => 'Cipta kod';

  @override
  String get sendCounting => 'Mengira…';

  @override
  String get sendRemove => 'Alih keluar';

  @override
  String get sendPreparing => 'Menyediakan fail anda…';

  @override
  String get sendAddFailed =>
      'Fail tidak dapat ditambah. Cuba lagi. Jika berlaku lagi, pastikan ruang kosong pada peranti ini mencukupi, atau pilih fail dari tempat lain.';

  @override
  String get sendYourCode => 'Kod anda';

  @override
  String get sendCodeHint =>
      'Pada peranti lain, buka TotalDrop, ketik Terima dan masukkan kod ini.';

  @override
  String get sendWaiting => 'Menunggu penerima…';

  @override
  String get sendCopy => 'Salin';

  @override
  String get sendCopied => 'Kod disalin';

  @override
  String get sendShare => 'Kongsi';

  @override
  String sendShareText(String code) {
    return 'Dapatkan fail saya dengan TotalDrop. Kod: $code';
  }

  @override
  String get sendSending => 'Menghantar';

  @override
  String get sendDone => 'Dihantar dan disahkan';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Kesemua $countString fail tiba dengan sempurna.',
      one: 'Fail tiba dengan sempurna.',
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
      other: '$countString fail tidak tiba dengan sempurna dan telah dibuang.',
      one: 'Satu fail tidak tiba dengan sempurna dan telah dibuang.',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '$countString item · $size';
  }

  @override
  String get receiveTitle => 'Terima fail';

  @override
  String get receiveCodeLabel => 'Kod';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Minta kod daripada penghantar.';

  @override
  String get receiveCodeInvalid =>
      'Itu tidak kelihatan seperti kod. Kod bermula dengan nombor, diikuti dua atau tiga perkataan.';

  @override
  String get receivePaste => 'Tampal';

  @override
  String get receiveConnect => 'Sambung';

  @override
  String get receiveConnecting => 'Menyambung dengan selamat…';

  @override
  String get receiveOffer => 'Fail masuk';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesString fail · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'dan $countString lagi';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Disimpan ke $folder';
  }

  @override
  String get receiveAccept => 'Terima';

  @override
  String get receiveDecline => 'Tolak';

  @override
  String get receiveDeclined => 'Anda telah menolak fail tersebut.';

  @override
  String get receiveReceiving => 'Menerima';

  @override
  String get receiveDone => 'Diterima dan disahkan';

  @override
  String get verificationTitle => 'Perkataan pengesahan';

  @override
  String get verificationHint =>
      'Pihak satu lagi melihat empat perkataan yang sama. Jika berbeza, berhenti.';

  @override
  String get sameNetworkNote =>
      'Berfungsi melalui internet. Apabila peranti tidak dapat berhubung secara terus, pemindahan yang disulitkan melalui pelayan geganti, yang tidak dapat membacanya.';

  @override
  String get done => 'Selesai';

  @override
  String get tryAgain => 'Cuba lagi';

  @override
  String get cancel => 'Batal';

  @override
  String get cancelTransferTitle => 'Hentikan pemindahan ini?';

  @override
  String get cancelTransferBody =>
      'Pihak satu lagi akan dimaklumkan bahawa pemindahan telah dihentikan.';

  @override
  String get keepGoing => 'Teruskan';

  @override
  String get stop => 'Hentikan';

  @override
  String get moreInfo => 'Butiran';

  @override
  String get errorWrongCode =>
      'Kod tidak sepadan. Kod tersalah taip, atau seseorang cuba memintas pemindahan. Tiada apa-apa yang dihantar. Mulakan semula dengan kod baharu.';

  @override
  String get errorPeerLeft =>
      'Pihak satu lagi keluar sebelum pemindahan selesai.';

  @override
  String get errorDeclined => 'Penerima menolak fail tersebut.';

  @override
  String get errorCancelled => 'Pihak satu lagi menghentikan pemindahan.';

  @override
  String get errorStopped => 'Dihentikan pada peranti ini.';

  @override
  String get errorInvalidCode => 'Kod itu tidak sah.';

  @override
  String get errorServer =>
      'Tidak dapat mencapai pelayan TotalDrop. Semak sambungan internet anda.';

  @override
  String get errorCodeNotInUse =>
      'Kod itu tidak digunakan: tersalah taip, telah tamat tempoh atau sudah digunakan. Minta kod baharu daripada penghantar.';

  @override
  String get errorCodeExpired =>
      'Kod tamat tempoh sebelum sesiapa menggunakannya. Hantar semula untuk mendapatkan kod baharu.';

  @override
  String get errorBadServer =>
      'Alamat pelayan tidak sah. Ia mesti bermula dengan wss://.';

  @override
  String get errorConnect =>
      'Tidak dapat menyambung ke peranti lain. Pastikan kedua-dua peranti dalam talian dan cuba lagi.';

  @override
  String get errorInvalidOffer =>
      'Tawaran melanggar peraturan keselamatan, jadi ia ditolak.';

  @override
  String get errorStorage => 'Satu fail tidak dapat dibaca atau disimpan.';

  @override
  String get errorOther => 'Ada sesuatu yang tidak kena.';

  @override
  String get causeTimedOut =>
      'Sambungan berhenti memberi respons. Salah satu peranti mungkin kehilangan rangkaian atau masuk mod tidur.';

  @override
  String get causeConnectionLost => 'Sambungan ke peranti lain terputus.';

  @override
  String get causeNoRoute =>
      'Peranti lain tidak dapat dicapai, sama ada secara terus atau melalui pelayan geganti.';

  @override
  String get causeRelay =>
      'Pelayan geganti tidak dapat dicapai atau belum ditetapkan, atau kedua-dua peranti tidak menggunakan pelayan yang sama.';

  @override
  String get causeProtocol =>
      'Peranti lain menghantar sesuatu yang tidak difahami oleh versi ini. Mengemas kini TotalDrop pada kedua-dua peranti mungkin membantu.';

  @override
  String get causeDiskFull => 'Ruang kosong pada peranti ini tidak mencukupi.';

  @override
  String get causeNoPermission =>
      'TotalDrop tidak dibenarkan membaca atau menulis fail atau folder yang diperlukannya.';

  @override
  String get causeNotFound =>
      'Fail atau folder tidak ditemui. Ia mungkin telah dialihkan atau dipadam.';

  @override
  String get causeReadOnly =>
      'Folder berada pada cakera yang tidak boleh ditulis.';

  @override
  String get causeFileChanged =>
      'Satu fail berubah semasa dihantar. Hantar semula selepas fail disimpan.';

  @override
  String get technicalDetail =>
      'Mesej teknikal dalam bahasa Inggeris, untuk disertakan semasa melaporkan masalah:';

  @override
  String get settingsServer => 'Pelayan pertemuan';

  @override
  String get settingsServerHelp =>
      'Membantu dua peranti dengan kod yang sama mencari satu sama lain. Ia tidak pernah melihat fail atau kunci anda.';

  @override
  String get settingsServerReset => 'Guna lalai';

  @override
  String get settingsServerInvalid => 'Mesti bermula dengan wss://';

  @override
  String get settingsSave => 'Simpan';

  @override
  String get settingsAbout => 'Perihal';

  @override
  String get settingsAboutBody =>
      'TotalDrop percuma dan sumber terbuka, dibiayai oleh derma. Berlesen GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Privasi';

  @override
  String get settingsPrivacyBody =>
      'Pelayan pertemuan, pelayan geganti dan pelayan STUN Cloudflare melihat alamat IP anda, begitu juga peranti lain apabila anda bersambung secara terus (kecuali mod privasi dihidupkan). Tiada satu pun daripadanya melihat nama fail, kandungan atau kunci, dan pelayan kami tidak menyimpan log. Fail bergerak terus antara peranti apabila boleh, jika tidak melalui pelayan geganti; kedua-duanya tetap disulitkan hujung ke hujung.';

  @override
  String get privacyMode => 'Sembunyikan alamat saya daripada peranti lain';

  @override
  String get privacyModeHint =>
      'Mod privasi: semuanya melalui pelayan geganti, jadi peranti lain tidak pernah mengetahui alamat IP anda. Lebih perlahan, dan peranti lain mesti menggunakan geganti yang sama (geganti lalai memenuhinya).';

  @override
  String get privacyModeConfirmTitle => 'Hidupkan mod privasi?';

  @override
  String get privacyModeConfirmBody =>
      'Pemindahan akan menjadi lebih perlahan, kerana tiada apa-apa yang bergerak terus antara peranti. Pelayan pertemuan dan pelayan geganti masih melihat alamat IP anda; kedua-duanya tidak pernah melihat fail atau kunci anda.';

  @override
  String get privacyModeTurnOn => 'Hidupkan';

  @override
  String settingsVersion(String version) {
    return 'Versi $version';
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
      'Terjemahan ini sedang disemak. Bantu memperbaikinya di GitHub.';

  @override
  String get settingsWordsEnglish =>
      'Kod dan perkataan pengesahan sentiasa dalam bahasa Inggeris, supaya dua peranti dalam bahasa berbeza menunjukkan perkataan yang sama.';

  @override
  String get sendDropHere => 'Lepaskan fail di sini';

  @override
  String get sendDropHint => 'Anda juga boleh seret fail ke tetingkap ini.';

  @override
  String get sendQrHint => 'Atau imbas kod ini dengan kamera peranti lain.';

  @override
  String get sendWaitingAccept =>
      'Semak perkataan, kemudian tunggu penerima menerimanya.';

  @override
  String get receiveFromImage => 'Baca QR daripada imej';

  @override
  String get receiveQrNotFound => 'Tiada kod TotalDrop dalam imej itu.';

  @override
  String get pickTitle => 'Pilih fail untuk dihantar';

  @override
  String get pickImageTitle => 'Pilih imej yang mempunyai kod QR';

  @override
  String get openFolder => 'Buka folder';

  @override
  String get back => 'Kembali';

  @override
  String get codeSuggestions => 'Cadangan';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }

  @override
  String get editUndo => 'Buat asal';

  @override
  String get editRedo => 'Buat semula';

  @override
  String get editCut => 'Potong';

  @override
  String get editCopy => 'Salin';

  @override
  String get editPaste => 'Tampal';

  @override
  String get editDelete => 'Padam';

  @override
  String get editSelectAll => 'Pilih semua';

  @override
  String get transferNoticeChannel => 'Pemindahan';

  @override
  String get transferNoticeSending => 'Menghantar dengan TotalDrop';

  @override
  String get transferNoticeReceiving => 'Menerima dengan TotalDrop';

  @override
  String get transferNoticeBody =>
      'Terus berjalan semasa anda menggunakan apl lain.';

  @override
  String get shareBusy =>
      'Pemindahan masih berjalan. Tunggu sehingga selesai atau hentikannya, kemudian kongsi semula.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count item yang dikongsi tidak dapat dibaca dan telah ditinggalkan.',
      one: 'Satu item yang dikongsi tidak dapat dibaca dan telah ditinggalkan.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'Pemindahan fail';

  @override
  String get desktopComment =>
      'Hantar fail terus ke peranti lain, disulitkan hujung ke hujung';

  @override
  String get desktopKeywords => 'hantar;terima;kongsi;pindah;fail;';

  @override
  String get desktopSendWith => 'Hantar dengan TotalDrop';

  @override
  String get metainfoSummary => 'Hantar fail terus ke peranti lain';

  @override
  String get metainfoAbout =>
      'TotalDrop menghantar fail dan folder dari satu peranti ke peranti lain, disulitkan hujung ke hujung. Pilih fail, kongsi kod pendek atau kod QR, dan peranti lain akan menerimanya. Tiada akaun, tiada e-mel, tiada nombor telefon.';

  @override
  String get metainfoVerified =>
      'Setiap fail disahkan apabila tiba; fail yang rosak dibuang';

  @override
  String get metainfoWords =>
      'Empat perkataan pengesahan menunjukkan tiada sesiapa di tengah';

  @override
  String get metainfoTheme =>
      'Mengikut tema desktop anda: Plasma, Hyprland, Omarchy, GNOME';

  @override
  String get sendText => 'Hantar teks';

  @override
  String get sendTextHint => 'Taip atau tampal teks yang hendak dihantar';

  @override
  String sendTextTooLong(String size) {
    return 'Teks boleh sebesar $size sahaja. Hantar teks yang lebih panjang sebagai fail.';
  }

  @override
  String get sendTextDone => 'Teks tiba dengan lengkap.';

  @override
  String get sendClipboard => 'Hantar papan keratan';

  @override
  String get sendClipboardEmpty => 'Papan keratan tidak mengandungi teks.';

  @override
  String get receiveOfferText => 'Teks masuk';

  @override
  String get receiveOfferTextNote => 'Teks dipaparkan di sini, tidak disimpan.';

  @override
  String get receiveTextDone => 'Teks diterima dan disahkan';

  @override
  String get receiveTextDeclined => 'Anda telah menolak teks tersebut.';

  @override
  String get receiveTextHidden =>
      'Teks mengandungi aksara tersembunyi yang dipaparkan sebagai U+…. Apabila disalin, teks diperoleh tepat seperti ia tiba.';

  @override
  String get sendTextHidden =>
      'Teks mengandungi aksara tersembunyi, seperti tanda arah. Aksara itu turut dihantar.';

  @override
  String get sendClipboardKept =>
      'Teks yang sudah ditaip dikekalkan; papan keratan tidak digunakan.';

  @override
  String get sendTextShortened =>
      'Hanya permulaan teks ditunjukkan; keseluruhannya dihantar.';

  @override
  String get receiveTextShortened =>
      'Hanya permulaan teks ditunjukkan. Salin teks menyalin keseluruhannya.';

  @override
  String get copyText => 'Salin teks';

  @override
  String get textCopied => 'Teks disalin';

  @override
  String get errorTextDamaged =>
      'Teks tidak tiba dengan lengkap, jadi ia tidak dipaparkan.';

  @override
  String get setupBackgroundTitle => 'Teruskan pemindahan di latar belakang';

  @override
  String get setupBackgroundBody =>
      'Supaya pemindahan diteruskan apabila anda menukar apl atau mengunci skrin, benarkan TotalDrop berjalan tanpa sekatan bateri. TotalDrop hanya berfungsi di latar belakang semasa pemindahan yang anda mulakan sedang berjalan.';

  @override
  String get setupBackgroundAllowed =>
      'Pengoptimuman bateri Android dimatikan untuk TotalDrop.';

  @override
  String get setupBackgroundRestricted =>
      'Pengoptimuman bateri Android dihidupkan untuk TotalDrop: pemindahan mungkin berhenti apabila anda menukar apl.';

  @override
  String get setupBackgroundBattery => 'Buka tetapan bateri';

  @override
  String get setupBackgroundBatteryHint =>
      'Pada halaman maklumat apl TotalDrop, buka Bateri dan benarkan penggunaan tanpa had. Nama tetapan ini berbeza antara telefon.';

  @override
  String setupBackgroundMaker(String maker) {
    return 'Telefon $maker mempunyai penjimat bateri sendiri';
  }

  @override
  String get setupBackgroundMakerBody =>
      'Buka juga penjimat bateri TotalDrop dan pilih \"Tiada sekatan\". Jika telefon mempunyai Automula, hidupkannya untuk TotalDrop.';

  @override
  String get setupBackgroundMakerOpen => 'Buka penjimat bateri TotalDrop';

  @override
  String get setupBackgroundAutostart => 'Buka Automula';

  @override
  String get setupBackgroundSkip => 'Bukan sekarang';

  @override
  String get settingsBackground => 'Pemindahan di latar belakang';

  @override
  String get settingsBackgroundOpen => 'Tetapan bateri';
}
