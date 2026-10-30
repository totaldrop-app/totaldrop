// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Korean (`ko`).
class AppLocalizationsKo extends AppLocalizations {
  AppLocalizationsKo([String locale = 'ko']) : super(locale);

  @override
  String get appTagline => '어떤 파일이든, 누구에게나, 어디로든.';

  @override
  String get homeSendTitle => '보내기';

  @override
  String get homeSendBody => '파일을 고르고 짧은 코드를 공유하세요';

  @override
  String get homeReceiveTitle => '받기';

  @override
  String get homeReceiveBody => '코드를 입력하고 파일을 저장하세요';

  @override
  String get homeFooter => '종단간 암호화. 계정도, 이메일도, 전화번호도 필요 없습니다.';

  @override
  String get settings => '설정';

  @override
  String get sendTitle => '파일 보내기';

  @override
  String get sendAddFiles => '파일 추가';

  @override
  String get sendEmpty => '아직 선택한 항목이 없습니다';

  @override
  String get sendEmptyHint => '보낼 파일을 추가하세요. 받는 사람은 정확히 이 파일들만 받습니다.';

  @override
  String get sendStrongCode => '더 강력한 코드';

  @override
  String get sendStrongCodeHint => '두 단어 대신 세 단어: 추측하기가 훨씬 어렵습니다';

  @override
  String get sendCreateCode => '코드 만들기';

  @override
  String get sendCounting => '계산 중…';

  @override
  String get sendRemove => '제거';

  @override
  String get sendPreparing => '파일을 준비하는 중…';

  @override
  String get sendYourCode => '내 코드';

  @override
  String get sendCodeHint => '다른 기기에서 TotalDrop을 열고 받기를 누른 다음 이 코드를 입력하세요.';

  @override
  String get sendWaiting => '받는 사람을 기다리는 중…';

  @override
  String get sendCopy => '복사';

  @override
  String get sendCopied => '코드를 복사했습니다';

  @override
  String get sendShare => '공유';

  @override
  String sendShareText(String code) {
    return 'TotalDrop으로 내 파일을 받으세요. 코드: $code';
  }

  @override
  String get sendSending => '보내는 중';

  @override
  String get sendDone => '전송 및 확인 완료';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '파일 $countString개가 모두 손상 없이 도착했습니다.',
      one: '파일이 손상 없이 도착했습니다.',
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
      other: '파일 $countString개가 온전히 도착하지 않아 삭제되었습니다.',
      one: '파일 1개가 온전히 도착하지 않아 삭제되었습니다.',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '항목 $countString개 · $size';
  }

  @override
  String get receiveTitle => '파일 받기';

  @override
  String get receiveCodeLabel => '코드';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => '보내는 사람에게 코드를 요청하세요.';

  @override
  String get receiveCodeInvalid =>
      '코드 형식이 아닙니다. 코드는 숫자로 시작하고 그 뒤에 단어 두세 개가 옵니다.';

  @override
  String get receivePaste => '붙여넣기';

  @override
  String get receiveConnect => '연결';

  @override
  String get receiveConnecting => '안전하게 연결하는 중…';

  @override
  String get receiveOffer => '받을 파일';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '파일 $filesString개 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '외 $countString개';
  }

  @override
  String receiveSaveTo(String folder) {
    return '저장 위치: $folder';
  }

  @override
  String get receiveAccept => '수락';

  @override
  String get receiveDecline => '거절';

  @override
  String get receiveDeclined => '파일을 거절했습니다.';

  @override
  String get receiveReceiving => '받는 중';

  @override
  String get receiveDone => '수신 및 확인 완료';

  @override
  String get verificationTitle => '확인 단어';

  @override
  String get verificationHint => '상대방에게도 같은 네 단어가 표시됩니다. 다르면 중단하세요.';

  @override
  String get sameNetworkNote =>
      '인터넷을 통해 작동합니다. 기기끼리 직접 연결할 수 없으면 암호화된 전송은 릴레이를 거치며, 릴레이는 내용을 읽을 수 없습니다.';

  @override
  String get done => '완료';

  @override
  String get tryAgain => '다시 시도';

  @override
  String get cancel => '취소';

  @override
  String get cancelTransferTitle => '이 전송을 중단할까요?';

  @override
  String get cancelTransferBody => '상대방에게 전송이 중단되었다고 알립니다.';

  @override
  String get keepGoing => '계속하기';

  @override
  String get stop => '중단';

  @override
  String get moreInfo => '자세히';

  @override
  String get errorWrongCode =>
      '코드가 일치하지 않습니다. 잘못 입력했거나 누군가 전송을 가로채려 했습니다. 아무것도 전송되지 않았습니다. 새 코드로 다시 시작하세요.';

  @override
  String get errorPeerLeft => '전송이 끝나기 전에 상대방이 나갔습니다.';

  @override
  String get errorDeclined => '받는 사람이 파일을 거절했습니다.';

  @override
  String get errorCancelled => '상대방이 전송을 중단했습니다.';

  @override
  String get errorStopped => '이 기기에서 중단했습니다.';

  @override
  String get errorInvalidCode => '유효하지 않은 코드입니다.';

  @override
  String get errorServer => 'TotalDrop 서버에 연결할 수 없습니다. 인터넷 연결을 확인하세요.';

  @override
  String get errorCodeNotInUse =>
      '사용 중인 코드가 아닙니다. 잘못 입력했거나, 만료되었거나, 이미 사용된 코드입니다. 보내는 사람에게 새 코드를 요청하세요.';

  @override
  String get errorCodeExpired =>
      '아무도 사용하지 않은 채 코드가 만료되었습니다. 새 코드를 받으려면 다시 보내세요.';

  @override
  String get errorBadServer => '서버 주소가 유효하지 않습니다. \'wss://\'로 시작해야 합니다.';

  @override
  String get errorConnect =>
      '다른 기기에 연결할 수 없습니다. 두 기기가 모두 온라인 상태인지 확인하고 다시 시도하세요.';

  @override
  String get errorInvalidOffer => '전송 요청이 보안 규칙을 위반하여 거부되었습니다.';

  @override
  String get errorStorage => '파일을 읽거나 저장할 수 없습니다.';

  @override
  String get errorOther => '문제가 발생했습니다.';

  @override
  String get settingsServer => '중개 서버';

  @override
  String get settingsServerHelp =>
      '같은 코드를 가진 두 기기가 서로를 찾도록 돕습니다. 파일이나 키는 절대 보지 않습니다.';

  @override
  String get settingsServerReset => '기본값 사용';

  @override
  String get settingsServerInvalid => '\'wss://\'로 시작해야 합니다';

  @override
  String get settingsSave => '저장';

  @override
  String get settingsAbout => '정보';

  @override
  String get settingsAboutBody =>
      'TotalDrop은 무료 오픈 소스이며 후원으로 운영됩니다. 라이선스는 GPL-3.0-or-later입니다.';

  @override
  String get settingsPrivacy => '개인정보 보호';

  @override
  String get settingsPrivacyBody =>
      '중개 서버, 릴레이, Cloudflare의 STUN 서버는 내 IP 주소를 볼 수 있으며, 직접 연결할 때는 상대 기기도 볼 수 있습니다(개인정보 보호 모드가 켜져 있지 않은 경우). 이들 중 어느 것도 파일 이름, 내용, 키를 볼 수 없으며, 저희 서버는 로그를 남기지 않습니다. 파일은 가능하면 기기 간에 직접 전송되고, 그렇지 않으면 릴레이를 거칩니다. 어느 경우든 종단간 암호화됩니다.';

  @override
  String get privacyMode => '상대 기기에 내 주소 숨기기';

  @override
  String get privacyModeHint =>
      '개인정보 보호 모드: 모든 전송이 릴레이를 거치므로 상대 기기는 내 IP 주소를 알 수 없습니다. 속도가 느려지며, 상대 기기도 같은 릴레이를 사용해야 합니다(기본 릴레이는 같습니다).';

  @override
  String get privacyModeConfirmTitle => '개인정보 보호 모드를 켤까요?';

  @override
  String get privacyModeConfirmBody =>
      '기기 간에 직접 전송되는 것이 없으므로 전송 속도가 느려집니다. 중개 서버와 릴레이는 여전히 IP 주소를 볼 수 있지만, 파일이나 키는 절대 볼 수 없습니다.';

  @override
  String get privacyModeTurnOn => '켜기';

  @override
  String settingsVersion(String version) {
    return '버전 $version';
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
  String get draftTranslation => '이 번역은 검토 중입니다. GitHub에서 개선을 도와주세요.';

  @override
  String get sendDropHere => '여기에 파일을 놓으세요';

  @override
  String get sendDropHint => '이 창으로 파일을 끌어다 놓아도 됩니다.';

  @override
  String get sendQrHint => '또는 다른 기기의 카메라로 이 코드를 스캔하세요.';

  @override
  String get sendWaitingAccept => '단어를 확인한 다음 받는 사람이 수락할 때까지 기다리세요.';

  @override
  String get receiveFromImage => '이미지에서 QR 코드 읽기';

  @override
  String get receiveQrNotFound => '이 이미지에 TotalDrop 코드가 없습니다.';

  @override
  String get pickTitle => '보낼 파일 선택';

  @override
  String get pickImageTitle => 'QR 코드가 있는 이미지 선택';

  @override
  String get openFolder => '폴더 열기';

  @override
  String get back => '뒤로';

  @override
  String get codeSuggestions => '추천';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
