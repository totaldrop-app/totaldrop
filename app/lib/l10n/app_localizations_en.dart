// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTagline => 'Any file, to anyone, anywhere.';

  @override
  String get homeSendTitle => 'Send';

  @override
  String get homeSendBody => 'Pick files and share a short code';

  @override
  String get homeReceiveTitle => 'Receive';

  @override
  String get homeReceiveBody => 'Enter a code and save the files';

  @override
  String get homeFooter =>
      'End-to-end encrypted. No account, no email, no phone number.';

  @override
  String get settings => 'Settings';

  @override
  String get sendTitle => 'Send files';

  @override
  String get sendAddFiles => 'Add files';

  @override
  String get sendEmpty => 'Nothing selected yet';

  @override
  String get sendEmptyHint =>
      'Add the files you want to send. The receiver gets exactly these, nothing else.';

  @override
  String get sendStrongCode => 'Stronger code';

  @override
  String get sendStrongCodeHint =>
      'Three words instead of two: much harder to guess';

  @override
  String get sendCreateCode => 'Create code';

  @override
  String get sendRemove => 'Remove';

  @override
  String get sendPreparing => 'Preparing your files…';

  @override
  String get sendYourCode => 'Your code';

  @override
  String get sendCodeHint =>
      'On the other device, open TotalDrop, tap Receive and enter this code.';

  @override
  String get sendWaiting => 'Waiting for the receiver…';

  @override
  String get sendCopy => 'Copy';

  @override
  String get sendCopied => 'Code copied';

  @override
  String get sendShare => 'Share';

  @override
  String sendShareText(String code) {
    return 'Get my files with TotalDrop. Code: $code';
  }

  @override
  String get sendSending => 'Sending';

  @override
  String get sendDone => 'Sent and verified';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'All $countString files arrived intact.',
      one: 'The file arrived intact.',
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
      other: '$countString files did not arrive intact and were discarded.',
      one: 'One file did not arrive intact and was discarded.',
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
      other: '$countString items',
      one: '$countString item',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Receive files';

  @override
  String get receiveCodeLabel => 'Code';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Ask the sender for their code.';

  @override
  String get receiveCodeInvalid =>
      'That does not look like a code. It starts with a number, then two or three words.';

  @override
  String get receivePaste => 'Paste';

  @override
  String get receiveConnect => 'Connect';

  @override
  String get receiveConnecting => 'Connecting securely…';

  @override
  String get receiveOffer => 'Incoming files';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString files',
      one: '$filesString file',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'and $countString more';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Saved to $folder';
  }

  @override
  String get receiveAccept => 'Accept';

  @override
  String get receiveDecline => 'Decline';

  @override
  String get receiveDeclined => 'You declined the files.';

  @override
  String get receiveReceiving => 'Receiving';

  @override
  String get receiveDone => 'Received and verified';

  @override
  String get verificationTitle => 'Verification words';

  @override
  String get verificationHint =>
      'The other person sees the same four words. If they differ, stop.';

  @override
  String get sameNetworkNote =>
      'Works across the internet. When the devices cannot reach each other directly, the encrypted transfer goes through the relay, which cannot read it.';

  @override
  String get done => 'Done';

  @override
  String get tryAgain => 'Try again';

  @override
  String get cancel => 'Cancel';

  @override
  String get cancelTransferTitle => 'Stop this transfer?';

  @override
  String get cancelTransferBody =>
      'The other side will be told the transfer stopped.';

  @override
  String get keepGoing => 'Keep going';

  @override
  String get stop => 'Stop';

  @override
  String get moreInfo => 'Details';

  @override
  String get errorWrongCode =>
      'The code did not match. It was mistyped, or someone tried to intercept the transfer. Nothing was sent. Start again with a new code.';

  @override
  String get errorPeerLeft =>
      'The other side left before the transfer finished.';

  @override
  String get errorDeclined => 'The receiver declined the files.';

  @override
  String get errorCancelled => 'The other side stopped the transfer.';

  @override
  String get errorInvalidCode => 'That code is not valid.';

  @override
  String get errorServer =>
      'Could not reach the TotalDrop server. Check your internet connection.';

  @override
  String get errorConnect =>
      'Could not connect to the other device. Check that both devices are online and try again.';

  @override
  String get errorInvalidOffer =>
      'The offer broke a safety rule, so it was refused.';

  @override
  String get errorStorage => 'A file could not be read or saved.';

  @override
  String get errorOther => 'Something went wrong.';

  @override
  String get settingsServer => 'Rendezvous server';

  @override
  String get settingsServerHelp =>
      'Helps two devices with the same code find each other. It never sees your files or keys.';

  @override
  String get settingsServerReset => 'Use the default';

  @override
  String get settingsServerInvalid => 'Must start with wss://';

  @override
  String get settingsSave => 'Save';

  @override
  String get settingsAbout => 'About';

  @override
  String get settingsAboutBody =>
      'TotalDrop is free and open source, funded by donations. Licensed GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Privacy';

  @override
  String get settingsPrivacyBody =>
      'The rendezvous server, the relay and Cloudflare\'s STUN server see your IP address, and so does the other device when you connect directly. None of them sees file names, contents or keys, and our servers keep no logs. Files go directly between the devices when they can, otherwise through the relay; they are encrypted end to end either way.';

  @override
  String settingsVersion(String version) {
    return 'Version $version';
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
      'This translation is being reviewed. Help improve it on Weblate.';
}
