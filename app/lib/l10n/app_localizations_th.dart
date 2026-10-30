// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Thai (`th`).
class AppLocalizationsTh extends AppLocalizations {
  AppLocalizationsTh([String locale = 'th']) : super(locale);

  @override
  String get appTagline => 'ทุกไฟล์ ถึงทุกคน ทุกที่';

  @override
  String get homeSendTitle => 'ส่ง';

  @override
  String get homeSendBody => 'เลือกไฟล์แล้วแชร์รหัสสั้นๆ';

  @override
  String get homeReceiveTitle => 'รับ';

  @override
  String get homeReceiveBody => 'ป้อนรหัสแล้วบันทึกไฟล์';

  @override
  String get homeFooter =>
      'เข้ารหัสตั้งแต่ต้นทางถึงปลายทาง ไม่ต้องมีบัญชี อีเมล หรือเบอร์โทรศัพท์';

  @override
  String get settings => 'การตั้งค่า';

  @override
  String get sendTitle => 'ส่งไฟล์';

  @override
  String get sendAddFiles => 'เพิ่มไฟล์';

  @override
  String get sendEmpty => 'ยังไม่ได้เลือกอะไร';

  @override
  String get sendEmptyHint =>
      'เพิ่มไฟล์ที่ต้องการส่ง ผู้รับจะได้รับเฉพาะไฟล์เหล่านี้ ไม่มีอย่างอื่น';

  @override
  String get sendStrongCode => 'รหัสที่รัดกุมขึ้น';

  @override
  String get sendStrongCodeHint => 'ใช้สามคำแทนสองคำ ทำให้เดายากขึ้นมาก';

  @override
  String get sendCreateCode => 'สร้างรหัส';

  @override
  String get sendCounting => 'กำลังนับ…';

  @override
  String get sendRemove => 'นำออก';

  @override
  String get sendPreparing => 'กำลังเตรียมไฟล์ของคุณ…';

  @override
  String get sendYourCode => 'รหัสของคุณ';

  @override
  String get sendCodeHint =>
      'บนอุปกรณ์อีกเครื่อง ให้เปิด TotalDrop แตะ รับ แล้วป้อนรหัสนี้';

  @override
  String get sendWaiting => 'กำลังรอผู้รับ…';

  @override
  String get sendCopy => 'คัดลอก';

  @override
  String get sendCopied => 'คัดลอกรหัสแล้ว';

  @override
  String get sendShare => 'แชร์';

  @override
  String sendShareText(String code) {
    return 'รับไฟล์ของฉันด้วย TotalDrop รหัส: $code';
  }

  @override
  String get sendSending => 'กำลังส่ง';

  @override
  String get sendDone => 'ส่งและตรวจสอบแล้ว';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'ไฟล์ทั้ง $countString ไฟล์มาถึงครบถ้วน',
      one: 'ไฟล์มาถึงครบถ้วน',
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
      other: 'มี $countString ไฟล์ที่มาถึงไม่ครบถ้วนและถูกทิ้งไป',
      one: 'มีหนึ่งไฟล์ที่มาถึงไม่ครบถ้วนและถูกทิ้งไป',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '$countString รายการ · $size';
  }

  @override
  String get receiveTitle => 'รับไฟล์';

  @override
  String get receiveCodeLabel => 'รหัส';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'ขอรหัสจากผู้ส่ง';

  @override
  String get receiveCodeInvalid =>
      'ดูไม่เหมือนรหัส รหัสขึ้นต้นด้วยตัวเลข ตามด้วยคำสองหรือสามคำ';

  @override
  String get receivePaste => 'วาง';

  @override
  String get receiveConnect => 'เชื่อมต่อ';

  @override
  String get receiveConnecting => 'กำลังเชื่อมต่ออย่างปลอดภัย…';

  @override
  String get receiveOffer => 'ไฟล์ขาเข้า';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesString ไฟล์ · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'และอีก $countString รายการ';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'บันทึกไว้ที่ $folder';
  }

  @override
  String get receiveAccept => 'ยอมรับ';

  @override
  String get receiveDecline => 'ปฏิเสธ';

  @override
  String get receiveDeclined => 'คุณปฏิเสธไฟล์แล้ว';

  @override
  String get receiveReceiving => 'กำลังรับ';

  @override
  String get receiveDone => 'รับและตรวจสอบแล้ว';

  @override
  String get verificationTitle => 'คำยืนยัน';

  @override
  String get verificationHint =>
      'อีกฝ่ายจะเห็นสี่คำเดียวกันนี้ ถ้าไม่ตรงกัน ให้หยุด';

  @override
  String get sameNetworkNote =>
      'ทำงานผ่านอินเทอร์เน็ต เมื่ออุปกรณ์เชื่อมต่อกันโดยตรงไม่ได้ การโอนที่เข้ารหัสจะผ่านเซิร์ฟเวอร์รีเลย์ ซึ่งอ่านข้อมูลนั้นไม่ได้';

  @override
  String get done => 'เสร็จ';

  @override
  String get tryAgain => 'ลองอีกครั้ง';

  @override
  String get cancel => 'ยกเลิก';

  @override
  String get cancelTransferTitle => 'หยุดการโอนนี้ไหม';

  @override
  String get cancelTransferBody => 'อีกฝ่ายจะได้รับแจ้งว่าการโอนหยุดแล้ว';

  @override
  String get keepGoing => 'ทำต่อ';

  @override
  String get stop => 'หยุด';

  @override
  String get moreInfo => 'รายละเอียด';

  @override
  String get errorWrongCode =>
      'รหัสไม่ตรงกัน อาจพิมพ์ผิด หรือมีคนพยายามดักการโอน ยังไม่มีอะไรถูกส่ง เริ่มใหม่ด้วยรหัสใหม่';

  @override
  String get errorPeerLeft => 'อีกฝ่ายออกไปก่อนการโอนจะเสร็จ';

  @override
  String get errorDeclined => 'ผู้รับปฏิเสธไฟล์';

  @override
  String get errorCancelled => 'อีกฝ่ายหยุดการโอน';

  @override
  String get errorStopped => 'หยุดบนอุปกรณ์นี้แล้ว';

  @override
  String get errorInvalidCode => 'รหัสนี้ไม่ถูกต้อง';

  @override
  String get errorServer =>
      'เชื่อมต่อเซิร์ฟเวอร์ TotalDrop ไม่ได้ ตรวจสอบการเชื่อมต่ออินเทอร์เน็ตของคุณ';

  @override
  String get errorCodeNotInUse =>
      'รหัสนี้ไม่ได้ใช้งานอยู่ อาจพิมพ์ผิด หมดอายุ หรือถูกใช้ไปแล้ว ขอรหัสใหม่จากผู้ส่ง';

  @override
  String get errorConnect =>
      'เชื่อมต่อกับอุปกรณ์อีกเครื่องไม่ได้ ตรวจสอบว่าอุปกรณ์ทั้งสองออนไลน์อยู่ แล้วลองอีกครั้ง';

  @override
  String get errorInvalidOffer => 'ข้อเสนอนี้ละเมิดกฎความปลอดภัย จึงถูกปฏิเสธ';

  @override
  String get errorStorage => 'อ่านหรือบันทึกไฟล์ไม่ได้';

  @override
  String get errorOther => 'เกิดข้อผิดพลาด';

  @override
  String get settingsServer => 'เซิร์ฟเวอร์นัดพบ';

  @override
  String get settingsServerHelp =>
      'ช่วยให้อุปกรณ์สองเครื่องที่ใช้รหัสเดียวกันหากันเจอ เซิร์ฟเวอร์นี้ไม่เคยเห็นไฟล์หรือคีย์ของคุณ';

  @override
  String get settingsServerReset => 'ใช้ค่าเริ่มต้น';

  @override
  String get settingsServerInvalid => 'ต้องขึ้นต้นด้วย wss://';

  @override
  String get settingsSave => 'บันทึก';

  @override
  String get settingsAbout => 'เกี่ยวกับ';

  @override
  String get settingsAboutBody =>
      'TotalDrop ใช้ฟรีและเป็นโอเพนซอร์ส ดำเนินงานด้วยเงินบริจาค สัญญาอนุญาต GPL-3.0-or-later';

  @override
  String get settingsPrivacy => 'ความเป็นส่วนตัว';

  @override
  String get settingsPrivacyBody =>
      'เซิร์ฟเวอร์นัดพบ เซิร์ฟเวอร์รีเลย์ และเซิร์ฟเวอร์ STUN ของ Cloudflare จะเห็นที่อยู่ IP ของคุณ และอุปกรณ์อีกเครื่องก็เห็นด้วยเมื่อเชื่อมต่อโดยตรง (เว้นแต่จะเปิดโหมดความเป็นส่วนตัว) ไม่มีฝ่ายใดเห็นชื่อไฟล์ เนื้อหา หรือคีย์ และเซิร์ฟเวอร์ของเราไม่เก็บบันทึก ไฟล์จะส่งระหว่างอุปกรณ์โดยตรงเมื่อทำได้ ไม่เช่นนั้นจะผ่านเซิร์ฟเวอร์รีเลย์ ไม่ว่าทางใดก็เข้ารหัสตั้งแต่ต้นทางถึงปลายทาง';

  @override
  String get privacyMode => 'ซ่อนที่อยู่ของฉันจากอุปกรณ์อีกเครื่อง';

  @override
  String get privacyModeHint =>
      'โหมดความเป็นส่วนตัว: ทุกอย่างผ่านเซิร์ฟเวอร์รีเลย์ อุปกรณ์อีกเครื่องจึงไม่มีทางรู้ที่อยู่ IP ของคุณ ช้ากว่า และอุปกรณ์อีกเครื่องต้องใช้รีเลย์เดียวกัน (รีเลย์เริ่มต้นใช้ได้)';

  @override
  String get privacyModeConfirmTitle => 'เปิดโหมดความเป็นส่วนตัวไหม';

  @override
  String get privacyModeConfirmBody =>
      'การโอนจะช้าลง เพราะไม่มีอะไรส่งระหว่างอุปกรณ์โดยตรง เซิร์ฟเวอร์นัดพบและเซิร์ฟเวอร์รีเลย์ยังคงเห็นที่อยู่ IP ของคุณ แต่ไม่เคยเห็นไฟล์หรือคีย์ของคุณ';

  @override
  String get privacyModeTurnOn => 'เปิด';

  @override
  String settingsVersion(String version) {
    return 'เวอร์ชัน $version';
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
      'คำแปลนี้อยู่ระหว่างการตรวจทาน ช่วยปรับปรุงได้ที่ GitHub';

  @override
  String get sendDropHere => 'วางไฟล์ที่นี่';

  @override
  String get sendDropHint => 'คุณลากไฟล์มาวางบนหน้าต่างนี้ได้ด้วย';

  @override
  String get sendQrHint => 'หรือสแกนรหัสนี้ด้วยกล้องของอุปกรณ์อีกเครื่อง';

  @override
  String get sendWaitingAccept => 'ตรวจสอบคำ แล้วรอให้ผู้รับยอมรับ';

  @override
  String get receiveFromImage => 'อ่าน QR จากรูปภาพ';

  @override
  String get receiveQrNotFound => 'ไม่มีรหัส TotalDrop ในรูปภาพนั้น';

  @override
  String get pickTitle => 'เลือกไฟล์ที่จะส่ง';

  @override
  String get pickImageTitle => 'เลือกรูปภาพที่มีคิวอาร์โค้ด';

  @override
  String get openFolder => 'เปิดโฟลเดอร์';

  @override
  String get back => 'กลับ';

  @override
  String get codeSuggestions => 'คำแนะนำ';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
