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
  String get sendAddFailed =>
      'File tidak dapat ditambahkan. Coba lagi. Jika terjadi lagi, pastikan ruang kosong di perangkat ini cukup, atau pilih file dari tempat lain.';

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
  String get errorCodeNotInUse =>
      'Kode itu tidak sedang digunakan: salah ketik, sudah kedaluwarsa, atau sudah dipakai. Minta kode baru dari pengirim.';

  @override
  String get errorCodeExpired =>
      'Kode kedaluwarsa sebelum ada yang memakainya. Kirim lagi untuk mendapatkan kode baru.';

  @override
  String get errorBadServer =>
      'Alamat server tidak valid. Harus diawali wss://.';

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
  String get causeTimedOut =>
      'Koneksi berhenti merespons. Salah satu perangkat mungkin kehilangan jaringan atau masuk mode tidur.';

  @override
  String get causeConnectionLost => 'Koneksi ke perangkat lain terputus.';

  @override
  String get causeNoRoute =>
      'Perangkat lain tidak dapat dijangkau, baik langsung maupun melalui server relai.';

  @override
  String get causeRelay =>
      'Server relai tidak dapat dijangkau atau belum diatur, atau kedua perangkat tidak memakai server yang sama.';

  @override
  String get causeProtocol =>
      'Perangkat lain mengirim sesuatu yang tidak dipahami versi ini. Memperbarui TotalDrop di kedua perangkat mungkin membantu.';

  @override
  String get causeDiskFull => 'Ruang kosong di perangkat ini tidak cukup.';

  @override
  String get causeNoPermission =>
      'TotalDrop tidak diizinkan membaca atau menulis file atau folder yang dibutuhkannya.';

  @override
  String get causeNotFound =>
      'File atau folder tidak ditemukan. Mungkin sudah dipindahkan atau dihapus.';

  @override
  String get causeReadOnly => 'Folder berada di disk yang tidak bisa ditulisi.';

  @override
  String get causeFileChanged =>
      'Sebuah file berubah saat sedang dikirim. Kirim lagi setelah file disimpan.';

  @override
  String get technicalDetail =>
      'Pesan teknis dalam bahasa Inggris, untuk disertakan saat melaporkan masalah:';

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
      'Terjemahan ini sedang ditinjau. Bantu perbaiki di GitHub.';

  @override
  String get settingsWordsEnglish =>
      'Kode dan kata verifikasi selalu dalam bahasa Inggris, agar dua perangkat dengan bahasa berbeda menampilkan kata yang sama.';

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

  @override
  String get editUndo => 'Urungkan';

  @override
  String get editRedo => 'Ulangi';

  @override
  String get editCut => 'Potong';

  @override
  String get editCopy => 'Salin';

  @override
  String get editPaste => 'Tempel';

  @override
  String get editDelete => 'Hapus';

  @override
  String get editSelectAll => 'Pilih semua';

  @override
  String get transferNoticeChannel => 'Transfer';

  @override
  String get transferNoticeSending => 'Mengirim dengan TotalDrop';

  @override
  String get transferNoticeReceiving => 'Menerima dengan TotalDrop';

  @override
  String get transferNoticeBody =>
      'Tetap berjalan saat Anda memakai aplikasi lain.';

  @override
  String get shareBusy =>
      'Masih ada transfer yang berjalan. Tunggu sampai selesai atau hentikan, lalu bagikan lagi.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count item yang dibagikan tidak dapat dibaca dan dilewati.',
      one: 'Satu item yang dibagikan tidak dapat dibaca dan dilewati.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'Transfer file';

  @override
  String get desktopComment =>
      'Kirim file langsung ke perangkat lain, terenkripsi end-to-end';

  @override
  String get desktopKeywords => 'kirim;terima;bagikan;transfer;file;';

  @override
  String get desktopSendWith => 'Kirim dengan TotalDrop';

  @override
  String get metainfoSummary => 'Kirim file langsung ke perangkat lain';

  @override
  String get metainfoAbout =>
      'TotalDrop mengirim file dan folder dari satu perangkat ke perangkat lain, terenkripsi end-to-end. Pilih file, bagikan kode pendek atau kode QR, dan perangkat lain akan menerimanya. Tanpa akun, tanpa email, tanpa nomor telepon.';

  @override
  String get metainfoVerified =>
      'Setiap file diverifikasi saat tiba; file yang rusak dibuang';

  @override
  String get metainfoWords =>
      'Empat kata verifikasi menunjukkan tidak ada pihak lain di tengah';

  @override
  String get metainfoTheme =>
      'Mengikuti tema desktop Anda: Plasma, Hyprland, Omarchy, GNOME';

  @override
  String get sendText => 'Kirim teks';

  @override
  String get sendTextHint => 'Ketik atau tempel teks yang ingin dikirim';

  @override
  String sendTextTooLong(String size) {
    return 'Teks paling besar $size. Kirim teks yang lebih panjang sebagai file.';
  }

  @override
  String get sendTextDone => 'Teks tiba dengan utuh.';

  @override
  String get sendClipboard => 'Kirim papan klip';

  @override
  String get sendClipboardEmpty => 'Papan klip tidak berisi teks.';

  @override
  String get receiveOfferText => 'Teks masuk';

  @override
  String get receiveOfferTextNote =>
      'Teks ditampilkan di sini, tidak disimpan.';

  @override
  String get receiveTextDone => 'Teks diterima dan terverifikasi';

  @override
  String get receiveTextDeclined => 'Anda menolak teks tersebut.';

  @override
  String get receiveTextHidden =>
      'Teks berisi karakter tersembunyi yang ditampilkan sebagai U+…. Saat disalin, teks didapat persis seperti saat tiba.';

  @override
  String get sendTextHidden =>
      'Teks berisi karakter tersembunyi, seperti tanda arah. Karakter itu ikut terkirim.';

  @override
  String get sendClipboardKept =>
      'Teks yang sudah diketik tetap disimpan; papan klip tidak digunakan.';

  @override
  String get sendTextShortened =>
      'Hanya awal teks yang ditampilkan; seluruhnya dikirim.';

  @override
  String get receiveTextShortened =>
      'Hanya awal teks yang ditampilkan. Salin teks menyalin seluruhnya.';

  @override
  String get copyText => 'Salin teks';

  @override
  String get textCopied => 'Teks disalin';

  @override
  String get errorTextDamaged =>
      'Teks tidak tiba dengan utuh, jadi tidak ditampilkan.';

  @override
  String get setupBackgroundTitle =>
      'Biarkan transfer berjalan di latar belakang';

  @override
  String get setupBackgroundBody =>
      'Agar transfer tetap berjalan saat Anda berpindah aplikasi atau mengunci layar, izinkan TotalDrop berjalan tanpa batasan baterai. TotalDrop hanya bekerja di latar belakang selama transfer yang Anda mulai sedang berjalan.';

  @override
  String get setupBackgroundAllowed =>
      'Pengoptimalan baterai Android nonaktif untuk TotalDrop.';

  @override
  String get setupBackgroundRestricted =>
      'Pengoptimalan baterai Android aktif untuk TotalDrop: transfer bisa berhenti saat Anda berpindah aplikasi.';

  @override
  String get setupBackgroundBattery => 'Buka setelan baterai';

  @override
  String get setupBackgroundBatteryHint =>
      'Cari TotalDrop di daftar dan nonaktifkan pengoptimalan baterainya. Nama setelan ini berbeda di setiap ponsel.';

  @override
  String setupBackgroundMaker(String maker) {
    return 'Ponsel $maker punya penghemat baterai sendiri';
  }

  @override
  String get setupBackgroundMakerBody =>
      'Buka juga penghemat baterai TotalDrop dan pilih \"Tanpa batasan\". Jika ponsel punya Mulai otomatis, aktifkan untuk TotalDrop.';

  @override
  String get setupBackgroundMakerOpen => 'Buka penghemat baterai TotalDrop';

  @override
  String get setupBackgroundAutostart => 'Buka Mulai otomatis';

  @override
  String get setupBackgroundSkip => 'Nanti saja';

  @override
  String get settingsBackground => 'Transfer di latar belakang';

  @override
  String get settingsBackgroundOpen => 'Setelan baterai';
}
