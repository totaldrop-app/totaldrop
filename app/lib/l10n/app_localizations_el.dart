// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Modern Greek (`el`).
class AppLocalizationsEl extends AppLocalizations {
  AppLocalizationsEl([String locale = 'el']) : super(locale);

  @override
  String get appTagline => 'Οποιοδήποτε αρχείο, σε οποιονδήποτε, οπουδήποτε.';

  @override
  String get homeSendTitle => 'Αποστολή';

  @override
  String get homeSendBody =>
      'Επιλέξτε αρχεία και μοιραστείτε έναν σύντομο κωδικό';

  @override
  String get homeReceiveTitle => 'Λήψη';

  @override
  String get homeReceiveBody =>
      'Εισαγάγετε έναν κωδικό και αποθηκεύστε τα αρχεία';

  @override
  String get homeFooter =>
      'Κρυπτογράφηση από άκρο σε άκρο. Χωρίς λογαριασμό, email ή αριθμό τηλεφώνου.';

  @override
  String get settings => 'Ρυθμίσεις';

  @override
  String get sendTitle => 'Αποστολή αρχείων';

  @override
  String get sendAddFiles => 'Προσθήκη αρχείων';

  @override
  String get sendEmpty => 'Δεν έχει επιλεγεί τίποτα ακόμη';

  @override
  String get sendEmptyHint =>
      'Προσθέστε τα αρχεία που θέλετε να στείλετε. Ο παραλήπτης λαμβάνει ακριβώς αυτά, τίποτε άλλο.';

  @override
  String get sendStrongCode => 'Ισχυρότερος κωδικός';

  @override
  String get sendStrongCodeHint =>
      'Τρεις λέξεις αντί για δύο: πολύ πιο δύσκολο να μαντευτεί';

  @override
  String get sendCreateCode => 'Δημιουργία κωδικού';

  @override
  String get sendCounting => 'Υπολογισμός…';

  @override
  String get sendRemove => 'Αφαίρεση';

  @override
  String get sendPreparing => 'Προετοιμασία αρχείων…';

  @override
  String get sendYourCode => 'Ο κωδικός σας';

  @override
  String get sendCodeHint =>
      'Στην άλλη συσκευή, ανοίξτε το TotalDrop, πατήστε Λήψη και εισαγάγετε αυτόν τον κωδικό.';

  @override
  String get sendWaiting => 'Αναμονή για τον παραλήπτη…';

  @override
  String get sendCopy => 'Αντιγραφή';

  @override
  String get sendCopied => 'Ο κωδικός αντιγράφηκε';

  @override
  String get sendShare => 'Κοινοποίηση';

  @override
  String sendShareText(String code) {
    return 'Πάρε τα αρχεία μου με το TotalDrop. Κωδικός: $code';
  }

  @override
  String get sendSending => 'Αποστολή';

  @override
  String get sendDone => 'Στάλθηκε και επαληθεύτηκε';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Και τα $countString αρχεία έφτασαν ακέραια.',
      one: 'Το αρχείο έφτασε ακέραιο.',
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
      other: '$countString αρχεία δεν έφτασαν ακέραια και απορρίφθηκαν.',
      one: 'Ένα αρχείο δεν έφτασε ακέραιο και απορρίφθηκε.',
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
      other: '$countString στοιχεία',
      one: '$countString στοιχείο',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Λήψη αρχείων';

  @override
  String get receiveCodeLabel => 'Κωδικός';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Ζητήστε τον κωδικό από τον αποστολέα.';

  @override
  String get receiveCodeInvalid =>
      'Αυτό δεν μοιάζει με κωδικό. Ξεκινά με έναν αριθμό και ακολουθούν δύο ή τρεις λέξεις.';

  @override
  String get receivePaste => 'Επικόλληση';

  @override
  String get receiveConnect => 'Σύνδεση';

  @override
  String get receiveConnecting => 'Ασφαλής σύνδεση…';

  @override
  String get receiveOffer => 'Εισερχόμενα αρχεία';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString αρχεία',
      one: '$filesString αρχείο',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'και $countString ακόμη';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Αποθήκευση σε $folder';
  }

  @override
  String get receiveAccept => 'Αποδοχή';

  @override
  String get receiveDecline => 'Απόρριψη';

  @override
  String get receiveDeclined => 'Απορρίψατε τα αρχεία.';

  @override
  String get receiveReceiving => 'Λήψη';

  @override
  String get receiveDone => 'Λήφθηκε και επαληθεύτηκε';

  @override
  String get verificationTitle => 'Λέξεις επαλήθευσης';

  @override
  String get verificationHint =>
      'Ο άλλος βλέπει τις ίδιες τέσσερις λέξεις. Αν διαφέρουν, σταματήστε.';

  @override
  String get sameNetworkNote =>
      'Λειτουργεί μέσω διαδικτύου. Όταν οι συσκευές δεν μπορούν να συνδεθούν απευθείας, η κρυπτογραφημένη μεταφορά περνά από τον διακομιστή αναμετάδοσης, που δεν μπορεί να τη διαβάσει.';

  @override
  String get done => 'Τέλος';

  @override
  String get tryAgain => 'Δοκιμή ξανά';

  @override
  String get cancel => 'Ακύρωση';

  @override
  String get cancelTransferTitle => 'Διακοπή της μεταφοράς;';

  @override
  String get cancelTransferBody =>
      'Η άλλη πλευρά θα ενημερωθεί ότι η μεταφορά διακόπηκε.';

  @override
  String get keepGoing => 'Συνέχεια';

  @override
  String get stop => 'Διακοπή';

  @override
  String get moreInfo => 'Λεπτομέρειες';

  @override
  String get errorWrongCode =>
      'Ο κωδικός δεν ταίριαξε. Πληκτρολογήθηκε λάθος ή κάποιος προσπάθησε να υποκλέψει τη μεταφορά. Δεν στάλθηκε τίποτα. Ξεκινήστε ξανά με νέο κωδικό.';

  @override
  String get errorPeerLeft =>
      'Η άλλη πλευρά αποχώρησε πριν ολοκληρωθεί η μεταφορά.';

  @override
  String get errorDeclined => 'Ο παραλήπτης απέρριψε τα αρχεία.';

  @override
  String get errorCancelled => 'Η άλλη πλευρά διέκοψε τη μεταφορά.';

  @override
  String get errorStopped => 'Διακόπηκε σε αυτή τη συσκευή.';

  @override
  String get errorInvalidCode => 'Ο κωδικός δεν είναι έγκυρος.';

  @override
  String get errorServer =>
      'Δεν ήταν δυνατή η σύνδεση με τον διακομιστή του TotalDrop. Ελέγξτε τη σύνδεσή σας στο διαδίκτυο.';

  @override
  String get errorConnect =>
      'Δεν ήταν δυνατή η σύνδεση με την άλλη συσκευή. Βεβαιωθείτε ότι και οι δύο συσκευές είναι συνδεδεμένες και δοκιμάστε ξανά.';

  @override
  String get errorInvalidOffer =>
      'Η προσφορά παραβίασε έναν κανόνα ασφαλείας και απορρίφθηκε.';

  @override
  String get errorStorage =>
      'Δεν ήταν δυνατή η ανάγνωση ή η αποθήκευση ενός αρχείου.';

  @override
  String get errorOther => 'Κάτι πήγε στραβά.';

  @override
  String get settingsServer => 'Διακομιστής συνάντησης';

  @override
  String get settingsServerHelp =>
      'Βοηθά δύο συσκευές με τον ίδιο κωδικό να βρουν η μία την άλλη. Δεν βλέπει ποτέ τα αρχεία ή τα κλειδιά σας.';

  @override
  String get settingsServerReset => 'Χρήση προεπιλογής';

  @override
  String get settingsServerInvalid => 'Πρέπει να ξεκινά με wss://';

  @override
  String get settingsSave => 'Αποθήκευση';

  @override
  String get settingsAbout => 'Πληροφορίες';

  @override
  String get settingsAboutBody =>
      'Το TotalDrop είναι δωρεάν και ανοιχτού κώδικα, με χρηματοδότηση από δωρεές. Άδεια: GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Απόρρητο';

  @override
  String get settingsPrivacyBody =>
      'Ο διακομιστής συνάντησης, ο διακομιστής αναμετάδοσης και ο διακομιστής STUN της Cloudflare βλέπουν τη διεύθυνση IP σας, όπως και η άλλη συσκευή όταν συνδέεστε απευθείας (εκτός αν η λειτουργία απορρήτου είναι ενεργή). Κανένας τους δεν βλέπει ονόματα αρχείων, περιεχόμενο ή κλειδιά, και οι διακομιστές μας δεν κρατούν αρχεία καταγραφής. Τα αρχεία πηγαίνουν απευθείας μεταξύ των συσκευών όταν είναι δυνατό, αλλιώς μέσω του διακομιστή αναμετάδοσης· σε κάθε περίπτωση είναι κρυπτογραφημένα από άκρο σε άκρο.';

  @override
  String get privacyMode => 'Απόκρυψη της διεύθυνσής μου από την άλλη συσκευή';

  @override
  String get privacyModeHint =>
      'Λειτουργία απορρήτου: όλα περνούν από τον διακομιστή αναμετάδοσης, ώστε η άλλη συσκευή να μη μάθει ποτέ τη διεύθυνση IP σας. Πιο αργό, και η άλλη συσκευή πρέπει να χρησιμοποιεί τον ίδιο διακομιστή αναμετάδοσης (ο προεπιλεγμένος το κάνει).';

  @override
  String get privacyModeConfirmTitle => 'Ενεργοποίηση λειτουργίας απορρήτου;';

  @override
  String get privacyModeConfirmBody =>
      'Οι μεταφορές θα είναι πιο αργές, γιατί τίποτα δεν πηγαίνει απευθείας μεταξύ των συσκευών. Ο διακομιστής συνάντησης και ο διακομιστής αναμετάδοσης εξακολουθούν να βλέπουν τη διεύθυνση IP σας· δεν βλέπουν ποτέ τα αρχεία ή τα κλειδιά σας.';

  @override
  String get privacyModeTurnOn => 'Ενεργοποίηση';

  @override
  String settingsVersion(String version) {
    return 'Έκδοση $version';
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
      'Αυτή η μετάφραση βρίσκεται υπό έλεγχο. Βοηθήστε να βελτιωθεί στο GitHub.';

  @override
  String get sendDropHere => 'Αφήστε αρχεία εδώ';

  @override
  String get sendDropHint =>
      'Μπορείτε επίσης να σύρετε αρχεία σε αυτό το παράθυρο.';

  @override
  String get sendQrHint =>
      'Ή σαρώστε αυτόν τον κωδικό με την κάμερα της άλλης συσκευής.';

  @override
  String get sendWaitingAccept =>
      'Ελέγξτε τις λέξεις και περιμένετε να αποδεχτεί ο παραλήπτης.';

  @override
  String get receiveFromImage => 'Ανάγνωση QR από εικόνα';

  @override
  String get receiveQrNotFound =>
      'Δεν υπάρχει κωδικός TotalDrop σε αυτή την εικόνα.';

  @override
  String get pickTitle => 'Επιλέξτε αρχεία για αποστολή';

  @override
  String get pickImageTitle => 'Επιλέξτε εικόνα με κωδικό QR';

  @override
  String get openFolder => 'Άνοιγμα φακέλου';

  @override
  String get back => 'Πίσω';

  @override
  String get codeSuggestions => 'Προτάσεις';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
