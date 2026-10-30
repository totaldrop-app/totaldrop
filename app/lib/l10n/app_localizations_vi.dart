// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Vietnamese (`vi`).
class AppLocalizationsVi extends AppLocalizations {
  AppLocalizationsVi([String locale = 'vi']) : super(locale);

  @override
  String get appTagline => 'Mọi tệp, cho bất kỳ ai, ở bất cứ đâu.';

  @override
  String get homeSendTitle => 'Gửi';

  @override
  String get homeSendBody => 'Chọn tệp và chia sẻ một mã ngắn';

  @override
  String get homeReceiveTitle => 'Nhận';

  @override
  String get homeReceiveBody => 'Nhập mã và lưu tệp';

  @override
  String get homeFooter =>
      'Mã hóa đầu cuối. Không cần tài khoản, email hay số điện thoại.';

  @override
  String get settings => 'Cài đặt';

  @override
  String get sendTitle => 'Gửi tệp';

  @override
  String get sendAddFiles => 'Thêm tệp';

  @override
  String get sendEmpty => 'Chưa chọn gì';

  @override
  String get sendEmptyHint =>
      'Thêm các tệp bạn muốn gửi. Người nhận chỉ nhận đúng những tệp này, không gì khác.';

  @override
  String get sendStrongCode => 'Mã mạnh hơn';

  @override
  String get sendStrongCodeHint => 'Ba từ thay vì hai: khó đoán hơn nhiều';

  @override
  String get sendCreateCode => 'Tạo mã';

  @override
  String get sendCounting => 'Đang đếm…';

  @override
  String get sendRemove => 'Xóa';

  @override
  String get sendPreparing => 'Đang chuẩn bị tệp của bạn…';

  @override
  String get sendYourCode => 'Mã của bạn';

  @override
  String get sendCodeHint =>
      'Trên thiết bị kia, mở TotalDrop, nhấn Nhận và nhập mã này.';

  @override
  String get sendWaiting => 'Đang chờ người nhận…';

  @override
  String get sendCopy => 'Sao chép';

  @override
  String get sendCopied => 'Đã sao chép mã';

  @override
  String get sendShare => 'Chia sẻ';

  @override
  String sendShareText(String code) {
    return 'Nhận tệp của tôi bằng TotalDrop. Mã: $code';
  }

  @override
  String get sendSending => 'Đang gửi';

  @override
  String get sendDone => 'Đã gửi và xác minh';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Cả $countString tệp đều đã đến nguyên vẹn.',
      one: 'Tệp đã đến nguyên vẹn.',
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
      other: '$countString tệp không đến nguyên vẹn và đã bị loại bỏ.',
      one: 'Một tệp không đến nguyên vẹn và đã bị loại bỏ.',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '$countString mục · $size';
  }

  @override
  String get receiveTitle => 'Nhận tệp';

  @override
  String get receiveCodeLabel => 'Mã';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Hãy hỏi người gửi mã của họ.';

  @override
  String get receiveCodeInvalid =>
      'Đây có vẻ không phải là mã. Mã bắt đầu bằng một số, sau đó là hai hoặc ba từ.';

  @override
  String get receivePaste => 'Dán';

  @override
  String get receiveConnect => 'Kết nối';

  @override
  String get receiveConnecting => 'Đang kết nối an toàn…';

  @override
  String get receiveOffer => 'Tệp đến';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesString tệp · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'và $countString mục khác';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Đã lưu vào $folder';
  }

  @override
  String get receiveAccept => 'Chấp nhận';

  @override
  String get receiveDecline => 'Từ chối';

  @override
  String get receiveDeclined => 'Bạn đã từ chối các tệp.';

  @override
  String get receiveReceiving => 'Đang nhận';

  @override
  String get receiveDone => 'Đã nhận và xác minh';

  @override
  String get verificationTitle => 'Từ xác minh';

  @override
  String get verificationHint =>
      'Người kia thấy cùng bốn từ này. Nếu khác nhau, hãy dừng lại.';

  @override
  String get sameNetworkNote =>
      'Hoạt động qua internet. Khi các thiết bị không thể kết nối trực tiếp với nhau, dữ liệu truyền đã mã hóa sẽ đi qua máy chủ chuyển tiếp, nơi không thể đọc được nội dung.';

  @override
  String get done => 'Xong';

  @override
  String get tryAgain => 'Thử lại';

  @override
  String get cancel => 'Hủy';

  @override
  String get cancelTransferTitle => 'Dừng lần truyền này?';

  @override
  String get cancelTransferBody =>
      'Bên kia sẽ được thông báo rằng việc truyền đã dừng.';

  @override
  String get keepGoing => 'Tiếp tục';

  @override
  String get stop => 'Dừng';

  @override
  String get moreInfo => 'Chi tiết';

  @override
  String get errorWrongCode =>
      'Mã không khớp. Có thể mã bị nhập sai, hoặc ai đó đã cố chặn lần truyền. Chưa có gì được gửi. Hãy bắt đầu lại với mã mới.';

  @override
  String get errorPeerLeft => 'Bên kia đã rời đi trước khi truyền xong.';

  @override
  String get errorDeclined => 'Người nhận đã từ chối các tệp.';

  @override
  String get errorCancelled => 'Bên kia đã dừng việc truyền.';

  @override
  String get errorStopped => 'Đã dừng trên thiết bị này.';

  @override
  String get errorInvalidCode => 'Mã này không hợp lệ.';

  @override
  String get errorServer =>
      'Không thể kết nối tới máy chủ TotalDrop. Hãy kiểm tra kết nối internet.';

  @override
  String get errorCodeNotInUse =>
      'Mã này không được sử dụng: có thể đã nhập sai, đã hết hạn hoặc đã được dùng. Hãy xin người gửi một mã mới.';

  @override
  String get errorConnect =>
      'Không thể kết nối với thiết bị kia. Hãy kiểm tra cả hai thiết bị đều đang trực tuyến rồi thử lại.';

  @override
  String get errorInvalidOffer =>
      'Đề nghị gửi đã vi phạm một quy tắc an toàn nên bị từ chối.';

  @override
  String get errorStorage => 'Không thể đọc hoặc lưu một tệp.';

  @override
  String get errorOther => 'Đã xảy ra lỗi.';

  @override
  String get settingsServer => 'Máy chủ điểm hẹn';

  @override
  String get settingsServerHelp =>
      'Giúp hai thiết bị có cùng mã tìm thấy nhau. Máy chủ này không bao giờ thấy tệp hay khóa của bạn.';

  @override
  String get settingsServerReset => 'Dùng mặc định';

  @override
  String get settingsServerInvalid => 'Phải bắt đầu bằng wss://';

  @override
  String get settingsSave => 'Lưu';

  @override
  String get settingsAbout => 'Giới thiệu';

  @override
  String get settingsAboutBody =>
      'TotalDrop miễn phí và mã nguồn mở, hoạt động nhờ quyên góp. Giấy phép GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Quyền riêng tư';

  @override
  String get settingsPrivacyBody =>
      'Máy chủ điểm hẹn, máy chủ chuyển tiếp và máy chủ STUN của Cloudflare thấy địa chỉ IP của bạn, thiết bị kia cũng thấy khi kết nối trực tiếp (trừ khi bật chế độ riêng tư). Không ai trong số đó thấy tên tệp, nội dung hay khóa, và máy chủ của chúng tôi không lưu nhật ký. Tệp đi trực tiếp giữa các thiết bị khi có thể, nếu không thì qua máy chủ chuyển tiếp; dù cách nào cũng được mã hóa đầu cuối.';

  @override
  String get privacyMode => 'Ẩn địa chỉ của tôi với thiết bị kia';

  @override
  String get privacyModeHint =>
      'Chế độ riêng tư: mọi thứ đi qua máy chủ chuyển tiếp, nên thiết bị kia không bao giờ biết địa chỉ IP của bạn. Chậm hơn, và thiết bị kia phải dùng cùng máy chủ chuyển tiếp (máy chủ mặc định đáp ứng điều này).';

  @override
  String get privacyModeConfirmTitle => 'Bật chế độ riêng tư?';

  @override
  String get privacyModeConfirmBody =>
      'Việc truyền sẽ chậm hơn, vì không có gì đi trực tiếp giữa các thiết bị. Máy chủ điểm hẹn và máy chủ chuyển tiếp vẫn thấy địa chỉ IP của bạn; chúng không bao giờ thấy tệp hay khóa của bạn.';

  @override
  String get privacyModeTurnOn => 'Bật';

  @override
  String settingsVersion(String version) {
    return 'Phiên bản $version';
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
      'Bản dịch này đang được xem xét. Hãy giúp cải thiện trên GitHub.';

  @override
  String get sendDropHere => 'Thả tệp vào đây';

  @override
  String get sendDropHint => 'Bạn cũng có thể kéo tệp vào cửa sổ này.';

  @override
  String get sendQrHint => 'Hoặc quét mã này bằng camera của thiết bị kia.';

  @override
  String get sendWaitingAccept =>
      'Kiểm tra các từ, rồi chờ người nhận chấp nhận.';

  @override
  String get receiveFromImage => 'Đọc QR từ ảnh';

  @override
  String get receiveQrNotFound => 'Ảnh này không có mã TotalDrop.';

  @override
  String get pickTitle => 'Chọn tệp để gửi';

  @override
  String get pickImageTitle => 'Chọn ảnh có mã QR';

  @override
  String get openFolder => 'Mở thư mục';

  @override
  String get back => 'Quay lại';

  @override
  String get codeSuggestions => 'Gợi ý';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
