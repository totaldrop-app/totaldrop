// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get appTagline => '任何文件，发给任何人，送到任何地方。';

  @override
  String get homeSendTitle => '发送';

  @override
  String get homeSendBody => '选择文件，分享一个简短的代码';

  @override
  String get homeReceiveTitle => '接收';

  @override
  String get homeReceiveBody => '输入代码，保存文件';

  @override
  String get homeFooter => '端到端加密。无需账号、邮箱或手机号码。';

  @override
  String get settings => '设置';

  @override
  String get sendTitle => '发送文件';

  @override
  String get sendAddFiles => '添加文件';

  @override
  String get sendEmpty => '尚未选择任何内容';

  @override
  String get sendEmptyHint => '添加要发送的文件。接收方只会收到这些文件，不会收到其他任何内容。';

  @override
  String get sendStrongCode => '更强的代码';

  @override
  String get sendStrongCodeHint => '用三个单词代替两个：更难被猜中';

  @override
  String get sendCreateCode => '生成代码';

  @override
  String get sendCounting => '正在计算…';

  @override
  String get sendRemove => '移除';

  @override
  String get sendPreparing => '正在准备文件…';

  @override
  String get sendYourCode => '你的代码';

  @override
  String get sendCodeHint => '在另一台设备上打开 TotalDrop，点按“接收”，然后输入此代码。';

  @override
  String get sendWaiting => '正在等待接收方…';

  @override
  String get sendCopy => '复制';

  @override
  String get sendCopied => '已复制代码';

  @override
  String get sendShare => '分享';

  @override
  String sendShareText(String code) {
    return '用 TotalDrop 接收我的文件。代码：$code';
  }

  @override
  String get sendSending => '正在发送';

  @override
  String get sendDone => '已发送并验证';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '全部 $countString 个文件均已完整送达。',
      one: '文件已完整送达。',
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
      other: '有 $countString 个文件未能完整送达，已被丢弃。',
      one: '有 1 个文件未能完整送达，已被丢弃。',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '$countString 项 · $size';
  }

  @override
  String get receiveTitle => '接收文件';

  @override
  String get receiveCodeLabel => '代码';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => '请向发送方索取代码。';

  @override
  String get receiveCodeInvalid => '这看起来不像代码。代码以数字开头，后面跟两个或三个单词。';

  @override
  String get receivePaste => '粘贴';

  @override
  String get receiveConnect => '连接';

  @override
  String get receiveConnecting => '正在建立安全连接…';

  @override
  String get receiveOffer => '待接收的文件';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesString 个文件 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '以及另外 $countString 项';
  }

  @override
  String receiveSaveTo(String folder) {
    return '保存到 $folder';
  }

  @override
  String get receiveAccept => '接受';

  @override
  String get receiveDecline => '拒绝';

  @override
  String get receiveDeclined => '你已拒绝这些文件。';

  @override
  String get receiveReceiving => '正在接收';

  @override
  String get receiveDone => '已接收并验证';

  @override
  String get verificationTitle => '验证词';

  @override
  String get verificationHint => '对方会看到同样的四个单词。如果不一致，请停止。';

  @override
  String get sameNetworkNote =>
      '可通过互联网使用。设备之间无法直接连接时，加密传输会经过中继服务器，中继服务器无法读取内容。';

  @override
  String get done => '完成';

  @override
  String get tryAgain => '重试';

  @override
  String get cancel => '取消';

  @override
  String get cancelTransferTitle => '要停止此次传输吗？';

  @override
  String get cancelTransferBody => '对方会收到传输已停止的通知。';

  @override
  String get keepGoing => '继续';

  @override
  String get stop => '停止';

  @override
  String get moreInfo => '详情';

  @override
  String get errorWrongCode =>
      '代码不匹配。可能是输错了，也可能有人试图拦截此次传输。没有发送任何内容。请使用新代码重新开始。';

  @override
  String get errorPeerLeft => '对方在传输完成前离开了。';

  @override
  String get errorDeclined => '接收方拒绝了这些文件。';

  @override
  String get errorCancelled => '对方停止了传输。';

  @override
  String get errorStopped => '已在此设备上停止。';

  @override
  String get errorInvalidCode => '该代码无效。';

  @override
  String get errorServer => '无法连接到 TotalDrop 服务器。请检查网络连接。';

  @override
  String get errorCodeNotInUse => '该代码未在使用中：可能输错了、已过期或已被使用。请向发送方索取新代码。';

  @override
  String get errorCodeExpired => '代码在有人使用之前就已过期。请重新发送以获取新代码。';

  @override
  String get errorBadServer => '服务器地址无效。必须以 wss:// 开头。';

  @override
  String get errorConnect => '无法连接到另一台设备。请确认两台设备都已联网，然后重试。';

  @override
  String get errorInvalidOffer => '该传输请求违反了安全规则，已被拒绝。';

  @override
  String get errorStorage => '无法读取或保存某个文件。';

  @override
  String get errorOther => '出了点问题。';

  @override
  String get settingsServer => '中介服务器';

  @override
  String get settingsServerHelp => '帮助持有相同代码的两台设备找到彼此。它永远看不到你的文件或密钥。';

  @override
  String get settingsServerReset => '使用默认值';

  @override
  String get settingsServerInvalid => '必须以 wss:// 开头';

  @override
  String get settingsSave => '保存';

  @override
  String get settingsAbout => '关于';

  @override
  String get settingsAboutBody =>
      'TotalDrop 免费且开源，依靠捐赠运作。采用 GPL-3.0-or-later 许可证。';

  @override
  String get settingsPrivacy => '隐私';

  @override
  String get settingsPrivacyBody =>
      '中介服务器、中继服务器和 Cloudflare 的 STUN 服务器会看到你的 IP 地址；直接连接时，对方设备也会看到（除非开启了隐私模式）。它们都看不到文件名、文件内容或密钥，我们的服务器也不保留日志。文件尽可能在设备之间直接传输，否则经由中继服务器；无论哪种方式都是端到端加密的。';

  @override
  String get privacyMode => '向对方设备隐藏我的地址';

  @override
  String get privacyModeHint =>
      '隐私模式：所有数据都经由中继服务器，对方设备永远不会知道你的 IP 地址。速度较慢，且对方设备必须使用同一个中继服务器（默认的中继服务器即可）。';

  @override
  String get privacyModeConfirmTitle => '要开启隐私模式吗？';

  @override
  String get privacyModeConfirmBody =>
      '由于设备之间不再直接传输，传输会变慢。中介服务器和中继服务器仍会看到你的 IP 地址，但永远看不到你的文件或密钥。';

  @override
  String get privacyModeTurnOn => '开启';

  @override
  String settingsVersion(String version) {
    return '版本 $version';
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
  String get draftTranslation => '此翻译正在审核中。欢迎在 GitHub 上帮助改进。';

  @override
  String get sendDropHere => '将文件拖放到此处';

  @override
  String get sendDropHint => '也可以将文件拖到此窗口中。';

  @override
  String get sendQrHint => '或者用另一台设备的相机扫描此二维码。';

  @override
  String get sendWaitingAccept => '核对单词，然后等待接收方接受。';

  @override
  String get receiveFromImage => '从图片读取二维码';

  @override
  String get receiveQrNotFound => '该图片中没有 TotalDrop 代码。';

  @override
  String get pickTitle => '选择要发送的文件';

  @override
  String get pickImageTitle => '选择包含二维码的图片';

  @override
  String get openFolder => '打开文件夹';

  @override
  String get back => '返回';

  @override
  String get codeSuggestions => '建议';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}

/// The translations for Chinese, using the Han script (`zh_Hans`).
class AppLocalizationsZhHans extends AppLocalizationsZh {
  AppLocalizationsZhHans() : super('zh_Hans');

  @override
  String get appTagline => '任何文件，发给任何人，送到任何地方。';

  @override
  String get homeSendTitle => '发送';

  @override
  String get homeSendBody => '选择文件，分享一个简短的代码';

  @override
  String get homeReceiveTitle => '接收';

  @override
  String get homeReceiveBody => '输入代码，保存文件';

  @override
  String get homeFooter => '端到端加密。无需账号、邮箱或手机号码。';

  @override
  String get settings => '设置';

  @override
  String get sendTitle => '发送文件';

  @override
  String get sendAddFiles => '添加文件';

  @override
  String get sendEmpty => '尚未选择任何内容';

  @override
  String get sendEmptyHint => '添加要发送的文件。接收方只会收到这些文件，不会收到其他任何内容。';

  @override
  String get sendStrongCode => '更强的代码';

  @override
  String get sendStrongCodeHint => '用三个单词代替两个：更难被猜中';

  @override
  String get sendCreateCode => '生成代码';

  @override
  String get sendCounting => '正在计算…';

  @override
  String get sendRemove => '移除';

  @override
  String get sendPreparing => '正在准备文件…';

  @override
  String get sendYourCode => '你的代码';

  @override
  String get sendCodeHint => '在另一台设备上打开 TotalDrop，点按“接收”，然后输入此代码。';

  @override
  String get sendWaiting => '正在等待接收方…';

  @override
  String get sendCopy => '复制';

  @override
  String get sendCopied => '已复制代码';

  @override
  String get sendShare => '分享';

  @override
  String sendShareText(String code) {
    return '用 TotalDrop 接收我的文件。代码：$code';
  }

  @override
  String get sendSending => '正在发送';

  @override
  String get sendDone => '已发送并验证';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '全部 $countString 个文件均已完整送达。',
      one: '文件已完整送达。',
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
      other: '有 $countString 个文件未能完整送达，已被丢弃。',
      one: '有 1 个文件未能完整送达，已被丢弃。',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '$countString 项 · $size';
  }

  @override
  String get receiveTitle => '接收文件';

  @override
  String get receiveCodeLabel => '代码';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => '请向发送方索取代码。';

  @override
  String get receiveCodeInvalid => '这看起来不像代码。代码以数字开头，后面跟两个或三个单词。';

  @override
  String get receivePaste => '粘贴';

  @override
  String get receiveConnect => '连接';

  @override
  String get receiveConnecting => '正在建立安全连接…';

  @override
  String get receiveOffer => '待接收的文件';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesString 个文件 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '以及另外 $countString 项';
  }

  @override
  String receiveSaveTo(String folder) {
    return '保存到 $folder';
  }

  @override
  String get receiveAccept => '接受';

  @override
  String get receiveDecline => '拒绝';

  @override
  String get receiveDeclined => '你已拒绝这些文件。';

  @override
  String get receiveReceiving => '正在接收';

  @override
  String get receiveDone => '已接收并验证';

  @override
  String get verificationTitle => '验证词';

  @override
  String get verificationHint => '对方会看到同样的四个单词。如果不一致，请停止。';

  @override
  String get sameNetworkNote =>
      '可通过互联网使用。设备之间无法直接连接时，加密传输会经过中继服务器，中继服务器无法读取内容。';

  @override
  String get done => '完成';

  @override
  String get tryAgain => '重试';

  @override
  String get cancel => '取消';

  @override
  String get cancelTransferTitle => '要停止此次传输吗？';

  @override
  String get cancelTransferBody => '对方会收到传输已停止的通知。';

  @override
  String get keepGoing => '继续';

  @override
  String get stop => '停止';

  @override
  String get moreInfo => '详情';

  @override
  String get errorWrongCode =>
      '代码不匹配。可能是输错了，也可能有人试图拦截此次传输。没有发送任何内容。请使用新代码重新开始。';

  @override
  String get errorPeerLeft => '对方在传输完成前离开了。';

  @override
  String get errorDeclined => '接收方拒绝了这些文件。';

  @override
  String get errorCancelled => '对方停止了传输。';

  @override
  String get errorStopped => '已在此设备上停止。';

  @override
  String get errorInvalidCode => '该代码无效。';

  @override
  String get errorServer => '无法连接到 TotalDrop 服务器。请检查网络连接。';

  @override
  String get errorCodeNotInUse => '该代码未在使用中：可能输错了、已过期或已被使用。请向发送方索取新代码。';

  @override
  String get errorCodeExpired => '代码在有人使用之前就已过期。请重新发送以获取新代码。';

  @override
  String get errorBadServer => '服务器地址无效。必须以 wss:// 开头。';

  @override
  String get errorConnect => '无法连接到另一台设备。请确认两台设备都已联网，然后重试。';

  @override
  String get errorInvalidOffer => '该传输请求违反了安全规则，已被拒绝。';

  @override
  String get errorStorage => '无法读取或保存某个文件。';

  @override
  String get errorOther => '出了点问题。';

  @override
  String get settingsServer => '中介服务器';

  @override
  String get settingsServerHelp => '帮助持有相同代码的两台设备找到彼此。它永远看不到你的文件或密钥。';

  @override
  String get settingsServerReset => '使用默认值';

  @override
  String get settingsServerInvalid => '必须以 wss:// 开头';

  @override
  String get settingsSave => '保存';

  @override
  String get settingsAbout => '关于';

  @override
  String get settingsAboutBody =>
      'TotalDrop 免费且开源，依靠捐赠运作。采用 GPL-3.0-or-later 许可证。';

  @override
  String get settingsPrivacy => '隐私';

  @override
  String get settingsPrivacyBody =>
      '中介服务器、中继服务器和 Cloudflare 的 STUN 服务器会看到你的 IP 地址；直接连接时，对方设备也会看到（除非开启了隐私模式）。它们都看不到文件名、文件内容或密钥，我们的服务器也不保留日志。文件尽可能在设备之间直接传输，否则经由中继服务器；无论哪种方式都是端到端加密的。';

  @override
  String get privacyMode => '向对方设备隐藏我的地址';

  @override
  String get privacyModeHint =>
      '隐私模式：所有数据都经由中继服务器，对方设备永远不会知道你的 IP 地址。速度较慢，且对方设备必须使用同一个中继服务器（默认的中继服务器即可）。';

  @override
  String get privacyModeConfirmTitle => '要开启隐私模式吗？';

  @override
  String get privacyModeConfirmBody =>
      '由于设备之间不再直接传输，传输会变慢。中介服务器和中继服务器仍会看到你的 IP 地址，但永远看不到你的文件或密钥。';

  @override
  String get privacyModeTurnOn => '开启';

  @override
  String settingsVersion(String version) {
    return '版本 $version';
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
  String get draftTranslation => '此翻译正在审核中。欢迎在 GitHub 上帮助改进。';

  @override
  String get sendDropHere => '将文件拖放到此处';

  @override
  String get sendDropHint => '也可以将文件拖到此窗口中。';

  @override
  String get sendQrHint => '或者用另一台设备的相机扫描此二维码。';

  @override
  String get sendWaitingAccept => '核对单词，然后等待接收方接受。';

  @override
  String get receiveFromImage => '从图片读取二维码';

  @override
  String get receiveQrNotFound => '该图片中没有 TotalDrop 代码。';

  @override
  String get pickTitle => '选择要发送的文件';

  @override
  String get pickImageTitle => '选择包含二维码的图片';

  @override
  String get openFolder => '打开文件夹';

  @override
  String get back => '返回';

  @override
  String get codeSuggestions => '建议';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}

/// The translations for Chinese, using the Han script (`zh_Hant`).
class AppLocalizationsZhHant extends AppLocalizationsZh {
  AppLocalizationsZhHant() : super('zh_Hant');

  @override
  String get appTagline => '任何檔案，傳給任何人，送到任何地方。';

  @override
  String get homeSendTitle => '傳送';

  @override
  String get homeSendBody => '選擇檔案，分享一組簡短的代碼';

  @override
  String get homeReceiveTitle => '接收';

  @override
  String get homeReceiveBody => '輸入代碼，儲存檔案';

  @override
  String get homeFooter => '端對端加密。不需要帳號、電子郵件或手機號碼。';

  @override
  String get settings => '設定';

  @override
  String get sendTitle => '傳送檔案';

  @override
  String get sendAddFiles => '新增檔案';

  @override
  String get sendEmpty => '尚未選取任何項目';

  @override
  String get sendEmptyHint => '新增要傳送的檔案。接收者只會收到這些檔案，不會收到其他任何內容。';

  @override
  String get sendStrongCode => '更強的代碼';

  @override
  String get sendStrongCodeHint => '用三個單字取代兩個：更難被猜中';

  @override
  String get sendCreateCode => '產生代碼';

  @override
  String get sendCounting => '正在計算…';

  @override
  String get sendRemove => '移除';

  @override
  String get sendPreparing => '正在準備檔案…';

  @override
  String get sendYourCode => '你的代碼';

  @override
  String get sendCodeHint => '在另一部裝置上開啟 TotalDrop，輕觸「接收」，然後輸入這組代碼。';

  @override
  String get sendWaiting => '正在等待接收者…';

  @override
  String get sendCopy => '複製';

  @override
  String get sendCopied => '已複製代碼';

  @override
  String get sendShare => '分享';

  @override
  String sendShareText(String code) {
    return '用 TotalDrop 接收我的檔案。代碼：$code';
  }

  @override
  String get sendSending => '正在傳送';

  @override
  String get sendDone => '已傳送並驗證';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '全部 $countString 個檔案均已完整送達。',
      one: '檔案已完整送達。',
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
      other: '有 $countString 個檔案未能完整送達，已被捨棄。',
      one: '有 1 個檔案未能完整送達，已被捨棄。',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '$countString 個項目 · $size';
  }

  @override
  String get receiveTitle => '接收檔案';

  @override
  String get receiveCodeLabel => '代碼';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => '請向傳送者索取代碼。';

  @override
  String get receiveCodeInvalid => '這看起來不像代碼。代碼以數字開頭，後面接兩個或三個單字。';

  @override
  String get receivePaste => '貼上';

  @override
  String get receiveConnect => '連線';

  @override
  String get receiveConnecting => '正在建立安全連線…';

  @override
  String get receiveOffer => '待接收的檔案';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesString 個檔案 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '以及另外 $countString 項';
  }

  @override
  String receiveSaveTo(String folder) {
    return '儲存至 $folder';
  }

  @override
  String get receiveAccept => '接受';

  @override
  String get receiveDecline => '拒絕';

  @override
  String get receiveDeclined => '你已拒絕這些檔案。';

  @override
  String get receiveReceiving => '正在接收';

  @override
  String get receiveDone => '已接收並驗證';

  @override
  String get verificationTitle => '驗證單字';

  @override
  String get verificationHint => '對方會看到同樣的四個單字。如果不一致，請停止。';

  @override
  String get sameNetworkNote =>
      '可透過網際網路使用。裝置之間無法直接連線時，加密傳輸會經過中繼伺服器，中繼伺服器無法讀取內容。';

  @override
  String get done => '完成';

  @override
  String get tryAgain => '再試一次';

  @override
  String get cancel => '取消';

  @override
  String get cancelTransferTitle => '要停止這次傳輸嗎？';

  @override
  String get cancelTransferBody => '對方會收到傳輸已停止的通知。';

  @override
  String get keepGoing => '繼續';

  @override
  String get stop => '停止';

  @override
  String get moreInfo => '詳細資訊';

  @override
  String get errorWrongCode =>
      '代碼不相符。可能是輸入錯誤，也可能有人試圖攔截這次傳輸。沒有傳送任何內容。請使用新的代碼重新開始。';

  @override
  String get errorPeerLeft => '對方在傳輸完成前離開了。';

  @override
  String get errorDeclined => '接收者拒絕了這些檔案。';

  @override
  String get errorCancelled => '對方停止了傳輸。';

  @override
  String get errorStopped => '已在這部裝置上停止。';

  @override
  String get errorInvalidCode => '這組代碼無效。';

  @override
  String get errorServer => '無法連線到 TotalDrop 伺服器。請檢查網路連線。';

  @override
  String get errorCodeNotInUse => '這組代碼目前未使用：可能輸入錯誤、已過期或已被使用。請向傳送者索取新的代碼。';

  @override
  String get errorCodeExpired => '代碼在有人使用之前就已過期。請重新傳送以取得新的代碼。';

  @override
  String get errorBadServer => '伺服器位址無效。必須以 wss:// 開頭。';

  @override
  String get errorConnect => '無法連線到另一部裝置。請確認兩部裝置都已連上網路，然後再試一次。';

  @override
  String get errorInvalidOffer => '這項傳輸要求違反了安全規則，已被拒絕。';

  @override
  String get errorStorage => '無法讀取或儲存某個檔案。';

  @override
  String get errorOther => '發生錯誤。';

  @override
  String get settingsServer => '中介伺服器';

  @override
  String get settingsServerHelp => '協助持有相同代碼的兩部裝置找到彼此。它絕對看不到你的檔案或金鑰。';

  @override
  String get settingsServerReset => '使用預設值';

  @override
  String get settingsServerInvalid => '必須以 wss:// 開頭';

  @override
  String get settingsSave => '儲存';

  @override
  String get settingsAbout => '關於';

  @override
  String get settingsAboutBody =>
      'TotalDrop 免費且開放原始碼，依靠捐款營運。採用 GPL-3.0-or-later 授權。';

  @override
  String get settingsPrivacy => '隱私權';

  @override
  String get settingsPrivacyBody =>
      '中介伺服器、中繼伺服器和 Cloudflare 的 STUN 伺服器會看到你的 IP 位址；直接連線時，對方裝置也會看到（除非開啟了隱私模式）。它們都看不到檔案名稱、檔案內容或金鑰，我們的伺服器也不保留記錄。檔案會盡可能在裝置之間直接傳輸，否則經由中繼伺服器；無論哪種方式都是端對端加密。';

  @override
  String get privacyMode => '對另一部裝置隱藏我的位址';

  @override
  String get privacyModeHint =>
      '隱私模式：所有資料都經由中繼伺服器，對方裝置永遠不會知道你的 IP 位址。速度較慢，且對方裝置必須使用同一個中繼伺服器（預設的中繼伺服器即可）。';

  @override
  String get privacyModeConfirmTitle => '要開啟隱私模式嗎？';

  @override
  String get privacyModeConfirmBody =>
      '由於裝置之間不再直接傳輸，傳輸會變慢。中介伺服器和中繼伺服器仍會看到你的 IP 位址，但絕對看不到你的檔案或金鑰。';

  @override
  String get privacyModeTurnOn => '開啟';

  @override
  String settingsVersion(String version) {
    return '版本 $version';
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
  String get draftTranslation => '這份翻譯正在審閱中。歡迎到 GitHub 協助改進。';

  @override
  String get sendDropHere => '將檔案拖放到這裡';

  @override
  String get sendDropHint => '也可以將檔案拖曳到這個視窗中。';

  @override
  String get sendQrHint => '或者用另一部裝置的相機掃描這個 QR 碼。';

  @override
  String get sendWaitingAccept => '核對單字，然後等待接收者接受。';

  @override
  String get receiveFromImage => '從圖片讀取 QR 碼';

  @override
  String get receiveQrNotFound => '這張圖片中沒有 TotalDrop 代碼。';

  @override
  String get pickTitle => '選擇要傳送的檔案';

  @override
  String get pickImageTitle => '選擇含有 QR 碼的圖片';

  @override
  String get openFolder => '開啟資料夾';

  @override
  String get back => '返回';

  @override
  String get codeSuggestions => '建議';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
