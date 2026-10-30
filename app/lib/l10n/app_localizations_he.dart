// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Hebrew (`he`).
class AppLocalizationsHe extends AppLocalizations {
  AppLocalizationsHe([String locale = 'he']) : super(locale);

  @override
  String get appTagline => 'כל קובץ, לכל אחד, לכל מקום.';

  @override
  String get homeSendTitle => 'שליחה';

  @override
  String get homeSendBody => 'בוחרים קבצים ומשתפים קוד קצר';

  @override
  String get homeReceiveTitle => 'קבלה';

  @override
  String get homeReceiveBody => 'מזינים קוד ושומרים את הקבצים';

  @override
  String get homeFooter =>
      'מוצפן מקצה לקצה. בלי חשבון, בלי אימייל, בלי מספר טלפון.';

  @override
  String get settings => 'הגדרות';

  @override
  String get sendTitle => 'שליחת קבצים';

  @override
  String get sendAddFiles => 'הוספת קבצים';

  @override
  String get sendEmpty => 'עדיין לא נבחר דבר';

  @override
  String get sendEmptyHint =>
      'יש להוסיף את הקבצים שברצונך לשלוח. הנמען יקבל בדיוק אותם, ושום דבר אחר.';

  @override
  String get sendStrongCode => 'קוד חזק יותר';

  @override
  String get sendStrongCodeHint => 'שלוש מילים במקום שתיים: הרבה יותר קשה לנחש';

  @override
  String get sendCreateCode => 'יצירת קוד';

  @override
  String get sendCounting => 'מתבצעת ספירה…';

  @override
  String get sendRemove => 'הסרה';

  @override
  String get sendPreparing => 'הקבצים שלך בהכנה…';

  @override
  String get sendYourCode => 'הקוד שלך';

  @override
  String get sendCodeHint =>
      'במכשיר השני יש לפתוח את TotalDrop, להקיש על קבלה ולהזין את הקוד הזה.';

  @override
  String get sendWaiting => 'בהמתנה לנמען…';

  @override
  String get sendCopy => 'העתקה';

  @override
  String get sendCopied => 'הקוד הועתק';

  @override
  String get sendShare => 'שיתוף';

  @override
  String sendShareText(String code) {
    return 'אפשר לקבל את הקבצים שלי עם TotalDrop. קוד: $code';
  }

  @override
  String get sendSending => 'בתהליך שליחה';

  @override
  String get sendDone => 'נשלח ואומת';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'כל $countString הקבצים הגיעו בשלמותם.',
      two: 'שני הקבצים הגיעו בשלמותם.',
      one: 'הקובץ הגיע בשלמותו.',
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
      other: '$countString קבצים לא הגיעו בשלמותם ונמחקו.',
      two: 'שני קבצים לא הגיעו בשלמותם ונמחקו.',
      one: 'קובץ אחד לא הגיע בשלמותו ונמחק.',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$countString פריטים',
      two: 'שני פריטים',
      one: 'פריט אחד',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'קבלת קבצים';

  @override
  String get receiveCodeLabel => 'קוד';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'יש לבקש מהשולח את הקוד שלו.';

  @override
  String get receiveCodeInvalid =>
      'זה לא נראה כמו קוד. קוד מתחיל במספר, ואחריו שתיים או שלוש מילים.';

  @override
  String get receivePaste => 'הדבקה';

  @override
  String get receiveConnect => 'התחברות';

  @override
  String get receiveConnecting => 'מתבצעת התחברות מאובטחת…';

  @override
  String get receiveOffer => 'קבצים נכנסים';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString קבצים',
      two: 'שני קבצים',
      one: 'קובץ אחד',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'ועוד $countString';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'נשמר בתיקייה $folder';
  }

  @override
  String get receiveAccept => 'אישור';

  @override
  String get receiveDecline => 'דחייה';

  @override
  String get receiveDeclined => 'דחית את הקבצים.';

  @override
  String get receiveReceiving => 'בתהליך קבלה';

  @override
  String get receiveDone => 'התקבל ואומת';

  @override
  String get verificationTitle => 'מילות אימות';

  @override
  String get verificationHint =>
      'הצד השני רואה את אותן ארבע מילים. אם הן שונות, יש לעצור.';

  @override
  String get sameNetworkNote =>
      'עובד דרך האינטרנט. כשהמכשירים לא יכולים להגיע זה לזה ישירות, ההעברה המוצפנת עוברת דרך שרת הממסר, שאינו יכול לקרוא אותה.';

  @override
  String get done => 'סיום';

  @override
  String get tryAgain => 'ניסיון חוזר';

  @override
  String get cancel => 'ביטול';

  @override
  String get cancelTransferTitle => 'לעצור את ההעברה הזו?';

  @override
  String get cancelTransferBody => 'הצד השני יקבל הודעה שההעברה נעצרה.';

  @override
  String get keepGoing => 'המשך';

  @override
  String get stop => 'עצירה';

  @override
  String get moreInfo => 'פרטים';

  @override
  String get errorWrongCode =>
      'הקוד לא תאם. הוא הוקלד בטעות, או שמישהו ניסה ליירט את ההעברה. דבר לא נשלח. יש להתחיל מחדש עם קוד חדש.';

  @override
  String get errorPeerLeft => 'הצד השני עזב לפני שההעברה הסתיימה.';

  @override
  String get errorDeclined => 'הנמען דחה את הקבצים.';

  @override
  String get errorCancelled => 'הצד השני עצר את ההעברה.';

  @override
  String get errorStopped => 'נעצר במכשיר הזה.';

  @override
  String get errorInvalidCode => 'הקוד הזה אינו תקף.';

  @override
  String get errorServer =>
      'לא ניתן להגיע לשרת של TotalDrop. יש לבדוק את החיבור לאינטרנט.';

  @override
  String get errorConnect =>
      'לא ניתן להתחבר למכשיר השני. יש לוודא ששני המכשירים מחוברים לאינטרנט ולנסות שוב.';

  @override
  String get errorInvalidOffer => 'ההצעה הפרה כלל אבטחה, ולכן נדחתה.';

  @override
  String get errorStorage => 'לא ניתן היה לקרוא או לשמור קובץ.';

  @override
  String get errorOther => 'משהו השתבש.';

  @override
  String get settingsServer => 'שרת מפגש';

  @override
  String get settingsServerHelp =>
      'עוזר לשני מכשירים עם אותו קוד למצוא זה את זה. הוא לעולם לא רואה את הקבצים או המפתחות שלך.';

  @override
  String get settingsServerReset => 'שימוש בברירת המחדל';

  @override
  String get settingsServerInvalid => 'הכתובת חייבת להתחיל ב־wss://';

  @override
  String get settingsSave => 'שמירה';

  @override
  String get settingsAbout => 'אודות';

  @override
  String get settingsAboutBody =>
      'TotalDrop חינמי וקוד פתוח, וממומן מתרומות. ברישיון GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'פרטיות';

  @override
  String get settingsPrivacyBody =>
      'שרת המפגש, שרת הממסר ושרת ה־STUN של Cloudflare רואים את כתובת ה־IP שלך, וכך גם המכשיר השני כשמתחברים ישירות (אלא אם מצב פרטיות פועל). אף אחד מהם לא רואה שמות קבצים, תוכן או מפתחות, והשרתים שלנו לא שומרים יומנים. הקבצים עוברים ישירות בין המכשירים כשאפשר, ואחרת דרך שרת הממסר; בכל מקרה הם מוצפנים מקצה לקצה.';

  @override
  String get privacyMode => 'הסתרת הכתובת שלי מהמכשיר השני';

  @override
  String get privacyModeHint =>
      'מצב פרטיות: הכול עובר דרך שרת הממסר, כך שהמכשיר השני לעולם לא יודע את כתובת ה־IP שלך. איטי יותר, והמכשיר השני חייב להשתמש באותו שרת ממסר (שרת ברירת המחדל עונה על כך).';

  @override
  String get privacyModeConfirmTitle => 'להפעיל מצב פרטיות?';

  @override
  String get privacyModeConfirmBody =>
      'ההעברות יהיו איטיות יותר, כי שום דבר לא עובר ישירות בין המכשירים. שרת המפגש ושרת הממסר עדיין רואים את כתובת ה־IP שלך; הם לעולם לא רואים את הקבצים או המפתחות שלך.';

  @override
  String get privacyModeTurnOn => 'הפעלה';

  @override
  String settingsVersion(String version) {
    return 'גרסה $version';
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
      'התרגום הזה נמצא בבדיקה. אפשר לעזור לשפר אותו ב־GitHub.';

  @override
  String get sendDropHere => 'אפשר לשחרר כאן קבצים';

  @override
  String get sendDropHint => 'אפשר גם לגרור קבצים אל החלון הזה.';

  @override
  String get sendQrHint => 'או לסרוק את הקוד הזה במצלמה של המכשיר השני.';

  @override
  String get sendWaitingAccept => 'יש לבדוק את המילים, ואז להמתין שהנמען יאשר.';

  @override
  String get receiveFromImage => 'קריאת QR מתמונה';

  @override
  String get receiveQrNotFound => 'אין בתמונה הזו קוד TotalDrop.';

  @override
  String get pickTitle => 'בחירת קבצים לשליחה';

  @override
  String get pickImageTitle => 'בחירת תמונה עם קוד QR';

  @override
  String get openFolder => 'פתיחת התיקייה';

  @override
  String get back => 'חזרה';

  @override
  String get codeSuggestions => 'הצעות';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
