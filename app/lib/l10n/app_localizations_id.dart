// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Indonesian (`id`).
class AppLocalizationsId extends AppLocalizations {
  AppLocalizationsId([String locale = 'id']) : super(locale);

  @override
  String get appTagline => 'File apa pun, untuk siapa pun, di mana pun.';

  @override
  String get homeSendTitle => 'Kirim';

  @override
  String get homeSendBody => 'Pilih file dan bagikan kode singkat';

  @override
  String get homeReceiveTitle => 'Terima';

  @override
  String get homeReceiveBody => 'Masukkan kode dan simpan file';

  @override
  String get homeFooter =>
      'Terenkripsi end-to-end. Tanpa akun, email, atau nomor telepon.';

  @override
  String get settings => 'Setelan';

  @override
  String get sendTitle => 'Kirim file';

  @override
  String get sendAddFiles => 'Tambahkan file';

  @override
  String get sendEmpty => 'Belum ada yang dipilih';

  @override
  String get sendEmptyHint =>
      'Tambahkan file yang ingin Anda kirim. Penerima hanya mendapat file ini, tidak ada yang lain.';

  @override
  String get sendStrongCode => 'Kode lebih kuat';

  @override
  String get sendStrongCodeHint =>
      'Tiga kata, bukan dua: jauh lebih sulit ditebak';

  @override
  String get sendCreateCode => 'Buat kode';

  @override
  String get sendCounting => 'Menghitung…';

  @override
  String get sendRemove => 'Hapus';

  @override
  String get sendPreparing => 'Menyiapkan file Anda…';

  @override
  String get sendYourCode => 'Kode Anda';

  @override
  String get sendCodeHint =>
      'Di perangkat lain, buka TotalDrop, ketuk Terima, lalu masukkan kode ini.';

  @override
  String get sendWaiting => 'Menunggu penerima…';

  @override
  String get sendCopy => 'Salin';

  @override
  String get sendCopied => 'Kode disalin';

  @override
  String get sendShare => 'Bagikan';

  @override
  String sendShareText(String code) {
    return 'Ambil file saya dengan TotalDrop. Kode: $code';
  }

  @override
  String get sendSending => 'Mengirim';

  @override
  String get sendDone => 'Terkirim dan terverifikasi';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Semua $countString file tiba dengan utuh.',
      one: 'File tiba dengan utuh.',
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
      other: '$countString file tidak tiba dengan utuh dan dibuang.',
      one: 'Satu file tidak tiba dengan utuh dan dibuang.',
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
  String get receiveTitle => 'Terima file';

  @override
  String get receiveCodeLabel => 'Kode';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Minta kode dari pengirim.';

  @override
  String get receiveCodeInvalid =>
      'Itu tidak terlihat seperti kode. Kode diawali angka, lalu dua atau tiga kata.';

  @override
  String get receivePaste => 'Tempel';

  @override
  String get receiveConnect => 'Hubungkan';

  @override
  String get receiveConnecting => 'Menghubungkan dengan aman…';

  @override
  String get receiveOffer => 'File masuk';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesString file · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'dan $countString lainnya';
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
  String get receiveDeclined => 'Anda menolak file tersebut.';

  @override
  String get receiveReceiving => 'Menerima';

  @override
  String get receiveDone => 'Diterima dan terverifikasi';

  @override
  String get verificationTitle => 'Kata verifikasi';

  @override
  String get verificationHint =>
      'Orang lain melihat empat kata yang sama. Jika berbeda, hentikan.';

  @override
  String get sameNetworkNote =>
      'Berfungsi melalui internet. Jika perangkat tidak dapat saling terhubung langsung, transfer terenkripsi melewati server relai, yang tidak dapat membacanya.';

  @override
  String get done => 'Selesai';

  @override
  String get tryAgain => 'Coba lagi';

  @override
  String get cancel => 'Batal';

  @override
  String get cancelTransferTitle => 'Hentikan transfer ini?';

  @override
  String get cancelTransferBody =>
      'Pihak lain akan diberi tahu bahwa transfer dihentikan.';

  @override
  String get keepGoing => 'Lanjutkan';

  @override
  String get stop => 'Hentikan';

  @override
  String get moreInfo => 'Detail';

  @override
  String get errorWrongCode =>
      'Kode tidak cocok. Kode salah ketik, atau seseorang mencoba menyadap transfer. Tidak ada yang terkirim. Mulai lagi dengan kode baru.';

  @override
  String get errorPeerLeft => 'Pihak lain pergi sebelum transfer selesai.';

  @override
  String get errorDeclined => 'Penerima menolak file tersebut.';

  @override
  String get errorCancelled => 'Pihak lain menghentikan transfer.';

  @override
  String get errorStopped => 'Dihentikan di perangkat ini.';

  @override
  String get errorInvalidCode => 'Kode itu tidak valid.';

  @override
  String get errorServer =>
      'Tidak dapat menjangkau server TotalDrop. Periksa koneksi internet Anda.';

  @override
  String get errorConnect =>
      'Tidak dapat terhubung ke perangkat lain. Pastikan kedua perangkat online, lalu coba lagi.';

  @override
  String get errorInvalidOffer =>
      'Tawaran melanggar aturan keamanan, jadi ditolak.';

  @override
  String get errorStorage => 'Sebuah file tidak dapat dibaca atau disimpan.';

  @override
  String get errorOther => 'Terjadi kesalahan.';

  @override
  String get settingsServer => 'Server pertemuan';

  @override
  String get settingsServerHelp =>
      'Membantu dua perangkat dengan kode yang sama saling menemukan. Server ini tidak pernah melihat file atau kunci Anda.';

  @override
  String get settingsServerReset => 'Gunakan default';

  @override
  String get settingsServerInvalid => 'Harus diawali wss://';

  @override
  String get settingsSave => 'Simpan';

  @override
  String get settingsAbout => 'Tentang';

  @override
  String get settingsAboutBody =>
      'TotalDrop gratis dan open source, didanai oleh donasi. Berlisensi GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Privasi';

  @override
  String get settingsPrivacyBody =>
      'Server pertemuan, server relai, dan server STUN Cloudflare melihat alamat IP Anda, begitu pula perangkat lain saat Anda terhubung langsung (kecuali mode privasi aktif). Tidak satu pun dari mereka melihat nama file, isi, atau kunci, dan server kami tidak menyimpan log. File dikirim langsung antarperangkat jika memungkinkan, jika tidak melalui server relai; keduanya tetap terenkripsi end-to-end.';

  @override
  String get privacyMode => 'Sembunyikan alamat saya dari perangkat lain';

  @override
  String get privacyModeHint =>
      'Mode privasi: semuanya melewati server relai, jadi perangkat lain tidak pernah mengetahui alamat IP Anda. Lebih lambat, dan perangkat lain harus menggunakan relai yang sama (relai default memenuhinya).';

  @override
  String get privacyModeConfirmTitle => 'Aktifkan mode privasi?';

  @override
  String get privacyModeConfirmBody =>
      'Transfer akan lebih lambat, karena tidak ada yang dikirim langsung antarperangkat. Server pertemuan dan server relai tetap melihat alamat IP Anda; keduanya tidak pernah melihat file atau kunci Anda.';

  @override
  String get privacyModeTurnOn => 'Aktifkan';

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
      'Terjemahan ini sedang ditinjau. Bantu perbaiki di Weblate.';

  @override
  String get sendDropHere => 'Letakkan file di sini';

  @override
  String get sendDropHint => 'Anda juga bisa menyeret file ke jendela ini.';

  @override
  String get sendQrHint => 'Atau pindai kode ini dengan kamera perangkat lain.';

  @override
  String get sendWaitingAccept =>
      'Periksa kata-katanya, lalu tunggu penerima menerima.';

  @override
  String get receiveFromImage => 'Baca QR dari gambar';

  @override
  String get receiveQrNotFound => 'Tidak ada kode TotalDrop di gambar itu.';

  @override
  String get pickTitle => 'Pilih file untuk dikirim';

  @override
  String get pickImageTitle => 'Pilih gambar dengan kode QR';

  @override
  String get openFolder => 'Buka folder';

  @override
  String get back => 'Kembali';

  @override
  String get codeSuggestions => 'Saran';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
