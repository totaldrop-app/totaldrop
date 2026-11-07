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
  String get sendAddFailed =>
      'ファイルを追加できませんでした。もう一度お試しください。それでも追加できない場合は、このデバイスに十分な空き容量があるかを確かめるか、別の場所からファイルを選んでください。';

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
  String get errorCodeNotInUse =>
      'このコードは使われていません。入力を間違えたか、有効期限が切れたか、すでに使用済みです。送信者に新しいコードを依頼してください。';

  @override
  String get errorCodeExpired =>
      '誰にも使われないまま、コードの有効期限が切れました。もう一度送信すると新しいコードが発行されます。';

  @override
  String get errorBadServer => 'サーバーのアドレスが無効です。「wss://」で始まる必要があります。';

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
  String get causeTimedOut =>
      '接続が応答しなくなりました。どちらかのデバイスがネットワークから切断されたか、スリープした可能性があります。';

  @override
  String get causeConnectionLost => '相手のデバイスとの接続が切れました。';

  @override
  String get causeNoRoute => '直接でもリレー経由でも、相手のデバイスに届きませんでした。';

  @override
  String get causeRelay => 'リレーに接続できないか、リレーが設定されていないか、2台のデバイスが同じリレーを使っていません。';

  @override
  String get causeProtocol =>
      '相手のデバイスが、このバージョンでは理解できないものを送ってきました。両方のデバイスで TotalDrop を更新すると解決する場合があります。';

  @override
  String get causeDiskFull => 'このデバイスの空き容量が足りません。';

  @override
  String get causeNoPermission =>
      'TotalDrop に、必要なファイルやフォルダーの読み取りまたは書き込みの許可がありません。';

  @override
  String get causeNotFound => 'ファイルまたはフォルダーが見つかりません。移動または削除された可能性があります。';

  @override
  String get causeReadOnly => 'フォルダーが書き込みできないディスク上にあります。';

  @override
  String get causeFileChanged => '送信中にファイルが変更されました。保存してからもう一度送信してください。';

  @override
  String get technicalDetail => '問題を報告するときに添える、英語の技術メッセージ:';

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
  String get draftTranslation => 'この翻訳はレビュー中です。GitHub で改善にご協力ください。';

  @override
  String get settingsWordsEnglish =>
      'コードと確認用の単語はどの言語でも英語のままです。言語の異なるデバイス同士でも同じ単語が表示されるようにするためです。';

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

  @override
  String get editUndo => '元に戻す';

  @override
  String get editRedo => 'やり直す';

  @override
  String get editCut => '切り取り';

  @override
  String get editCopy => 'コピー';

  @override
  String get editPaste => '貼り付け';

  @override
  String get editDelete => '削除';

  @override
  String get editSelectAll => 'すべて選択';

  @override
  String get transferNoticeChannel => '転送';

  @override
  String get transferNoticeSending => 'TotalDrop で送信中';

  @override
  String get transferNoticeReceiving => 'TotalDrop で受信中';

  @override
  String get transferNoticeBody => 'ほかのアプリを使っている間も続きます。';

  @override
  String get shareTextSoon =>
      'TotalDrop はまだテキストを送信できません。ファイルのみです。ファイルを共有してください。';

  @override
  String get shareBusy => '転送がまだ進行中です。完了を待つか停止してから、もう一度共有してください。';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '共有された$count件の項目を読み取れなかったため、除外しました。',
      one: '共有された1件の項目を読み取れなかったため、除外しました。',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'ファイル転送';

  @override
  String get desktopComment => 'ファイルを別のデバイスへ直接送信。エンドツーエンドで暗号化';

  @override
  String get desktopKeywords => '送信;受信;共有;転送;ファイル;';

  @override
  String get desktopSendWith => 'TotalDrop で送信';

  @override
  String get sendText => 'テキストを送信';

  @override
  String get sendTextHint => '送信するテキストを入力するか貼り付けてください';

  @override
  String sendTextTooLong(String size) {
    return 'テキストは最大 $size までです。それより長いものはファイルとして送信してください。';
  }

  @override
  String get sendTextDone => 'テキストは欠けることなく届きました。';

  @override
  String get sendClipboard => 'クリップボードを送信';

  @override
  String get sendClipboardEmpty => 'クリップボードにテキストがありません。';

  @override
  String get receiveOfferText => '受信するテキスト';

  @override
  String get receiveOfferTextNote => 'テキストはここに表示され、保存はされません。';

  @override
  String get receiveTextDone => 'テキスト受信完了・検証済み';

  @override
  String get receiveTextDeclined => 'テキストの受け取りを拒否しました。';

  @override
  String get receiveTextHidden =>
      'テキストに見えない文字が含まれており、U+… の形で表示しています。コピーすると、届いたとおりのテキストがコピーされます。';

  @override
  String get copyText => 'テキストをコピー';

  @override
  String get textCopied => 'テキストをコピーしました';

  @override
  String get errorTextDamaged => 'テキストが欠けることなく届かなかったため、表示しません。';
}
