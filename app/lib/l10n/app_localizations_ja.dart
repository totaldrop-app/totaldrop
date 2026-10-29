// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Japanese (`ja`).
class AppLocalizationsJa extends AppLocalizations {
  AppLocalizationsJa([String locale = 'ja']) : super(locale);

  @override
  String get appTagline => 'どんなファイルも、誰にでも、どこへでも。';

  @override
  String get homeSendTitle => '送信';

  @override
  String get homeSendBody => 'ファイルを選んで短いコードを共有';

  @override
  String get homeReceiveTitle => '受信';

  @override
  String get homeReceiveBody => 'コードを入力してファイルを保存';

  @override
  String get homeFooter => 'エンドツーエンドで暗号化。アカウントもメールアドレスも電話番号も不要です。';

  @override
  String get settings => '設定';

  @override
  String get sendTitle => 'ファイルを送信';

  @override
  String get sendAddFiles => 'ファイルを追加';

  @override
  String get sendEmpty => 'まだ何も選択されていません';

  @override
  String get sendEmptyHint => '送信するファイルを追加してください。受信者が受け取るのはこれらのファイルだけです。';

  @override
  String get sendStrongCode => 'より強力なコード';

  @override
  String get sendStrongCodeHint => '2語ではなく3語：推測がはるかに難しくなります';

  @override
  String get sendCreateCode => 'コードを作成';

  @override
  String get sendCounting => '計算中…';

  @override
  String get sendRemove => '削除';

  @override
  String get sendPreparing => 'ファイルを準備しています…';

  @override
  String get sendYourCode => 'あなたのコード';

  @override
  String get sendCodeHint => '相手のデバイスで TotalDrop を開き、「受信」をタップしてこのコードを入力してください。';

  @override
  String get sendWaiting => '受信者を待っています…';

  @override
  String get sendCopy => 'コピー';

  @override
  String get sendCopied => 'コードをコピーしました';

  @override
  String get sendShare => '共有';

  @override
  String sendShareText(String code) {
    return 'TotalDrop で私のファイルを受け取ってください。コード：$code';
  }

  @override
  String get sendSending => '送信中';

  @override
  String get sendDone => '送信完了・検証済み';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$countString個のファイルがすべて破損なく届きました。',
      one: 'ファイルは破損なく届きました。',
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
      other: '$countString個のファイルが正しく届かなかったため、破棄されました。',
      one: '1個のファイルが正しく届かなかったため、破棄されました。',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '$countString個の項目 · $size';
  }

  @override
  String get receiveTitle => 'ファイルを受信';

  @override
  String get receiveCodeLabel => 'コード';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => '送信者にコードを教えてもらってください。';

  @override
  String get receiveCodeInvalid =>
      'コードの形式ではありません。コードは数字で始まり、そのあとに2語または3語が続きます。';

  @override
  String get receivePaste => '貼り付け';

  @override
  String get receiveConnect => '接続';

  @override
  String get receiveConnecting => '安全に接続しています…';

  @override
  String get receiveOffer => '受信するファイル';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesString個のファイル · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'ほか$countString件';
  }

  @override
  String receiveSaveTo(String folder) {
    return '保存先：$folder';
  }

  @override
  String get receiveAccept => '受け取る';

  @override
  String get receiveDecline => '拒否';

  @override
  String get receiveDeclined => 'ファイルの受け取りを拒否しました。';

  @override
  String get receiveReceiving => '受信中';

  @override
  String get receiveDone => '受信完了・検証済み';

  @override
  String get verificationTitle => '確認用の単語';

  @override
  String get verificationHint => '相手にも同じ4つの単語が表示されます。異なる場合は中止してください。';

  @override
  String get sameNetworkNote =>
      'インターネット経由で使えます。デバイス同士が直接つながらない場合、暗号化された転送はリレーを経由します。リレーは内容を読めません。';

  @override
  String get done => '完了';

  @override
  String get tryAgain => '再試行';

  @override
  String get cancel => 'キャンセル';

  @override
  String get cancelTransferTitle => 'この転送を中止しますか？';

  @override
  String get cancelTransferBody => '転送を中止したことが相手に通知されます。';

  @override
  String get keepGoing => '続ける';

  @override
  String get stop => '中止';

  @override
  String get moreInfo => '詳細';

  @override
  String get errorWrongCode =>
      'コードが一致しませんでした。入力を間違えたか、誰かが転送を傍受しようとしました。何も送信されていません。新しいコードでやり直してください。';

  @override
  String get errorPeerLeft => '転送が終わる前に相手が離れました。';

  @override
  String get errorDeclined => '受信者がファイルを拒否しました。';

  @override
  String get errorCancelled => '相手が転送を中止しました。';

  @override
  String get errorStopped => 'このデバイスで中止しました。';

  @override
  String get errorInvalidCode => 'このコードは無効です。';

  @override
  String get errorServer => 'TotalDrop サーバーに接続できませんでした。インターネット接続を確認してください。';

  @override
  String get errorConnect =>
      '相手のデバイスに接続できませんでした。両方のデバイスがオンラインであることを確認して、もう一度お試しください。';

  @override
  String get errorInvalidOffer => '送信内容が安全上のルールに違反していたため、拒否されました。';

  @override
  String get errorStorage => 'ファイルを読み込めないか、保存できませんでした。';

  @override
  String get errorOther => '問題が発生しました。';

  @override
  String get settingsServer => '仲介サーバー';

  @override
  String get settingsServerHelp =>
      '同じコードを持つ2台のデバイスが互いを見つけられるようにします。ファイルや鍵を見ることはありません。';

  @override
  String get settingsServerReset => 'デフォルトを使用';

  @override
  String get settingsServerInvalid => '「wss://」で始まる必要があります';

  @override
  String get settingsSave => '保存';

  @override
  String get settingsAbout => 'このアプリについて';

  @override
  String get settingsAboutBody =>
      'TotalDrop は無料のオープンソースソフトウェアで、寄付によって運営されています。ライセンスは GPL-3.0-or-later です。';

  @override
  String get settingsPrivacy => 'プライバシー';

  @override
  String get settingsPrivacyBody =>
      '仲介サーバー、リレー、Cloudflare の STUN サーバーにはあなたの IP アドレスが見えます。直接接続した場合は相手のデバイスにも見えます（プライバシーモードがオンの場合を除く）。いずれもファイル名、内容、鍵は見えず、私たちのサーバーはログを残しません。ファイルは可能であればデバイス間で直接、そうでなければリレーを経由して送られます。どちらの場合もエンドツーエンドで暗号化されます。';

  @override
  String get privacyMode => '相手のデバイスに自分のアドレスを隠す';

  @override
  String get privacyModeHint =>
      'プライバシーモード：すべてがリレーを経由するため、相手のデバイスにあなたの IP アドレスが知られることはありません。速度は遅くなり、相手のデバイスも同じリレーを使う必要があります（デフォルトのリレーなら同じです）。';

  @override
  String get privacyModeConfirmTitle => 'プライバシーモードをオンにしますか？';

  @override
  String get privacyModeConfirmBody =>
      'デバイス間で直接やり取りしなくなるため、転送は遅くなります。仲介サーバーとリレーには引き続き IP アドレスが見えますが、ファイルや鍵が見えることはありません。';

  @override
  String get privacyModeTurnOn => 'オンにする';

  @override
  String settingsVersion(String version) {
    return 'バージョン $version';
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
  String get draftTranslation => 'この翻訳はレビュー中です。Weblate で改善にご協力ください。';

  @override
  String get sendDropHere => 'ここにファイルをドロップ';

  @override
  String get sendDropHint => 'このウィンドウにファイルをドラッグして追加することもできます。';

  @override
  String get sendQrHint => 'または、相手のデバイスのカメラでこのコードを読み取ってください。';

  @override
  String get sendWaitingAccept => '単語を確認してから、受信者が受け取るのを待ってください。';

  @override
  String get receiveFromImage => '画像から QR コードを読み取る';

  @override
  String get receiveQrNotFound => 'この画像には TotalDrop のコードがありません。';

  @override
  String get pickTitle => '送信するファイルを選択';

  @override
  String get pickImageTitle => 'QR コードを含む画像を選択';

  @override
  String get openFolder => 'フォルダを開く';

  @override
  String get back => '戻る';

  @override
  String get codeSuggestions => '候補';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
