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
  String get sendCounting => 'Counting…';

  @override
  String get sendRemove => 'Remove';

  @override
  String get sendPreparing => 'Preparing your files…';

  @override
  String get sendAddFailed =>
      'The files could not be added. Try again. If it happens again, check that this device has enough free space, or choose the files from another place.';

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
  String get errorStopped => 'Stopped on this device.';

  @override
  String get errorInvalidCode => 'That code is not valid.';

  @override
  String get errorServer =>
      'Could not reach the TotalDrop server. Check your internet connection.';

  @override
  String get errorCodeNotInUse =>
      'That code is not in use: it was mistyped, has expired or was already used. Ask the sender for a new one.';

  @override
  String get errorCodeExpired =>
      'Nobody used the code before it expired. Send again to get a new code.';

  @override
  String get errorBadServer =>
      'The server address is not valid. It must start with wss://.';

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
  String get causeTimedOut =>
      'The connection stopped responding. One of the devices may have lost its network or gone to sleep.';

  @override
  String get causeConnectionLost =>
      'The connection to the other device was cut.';

  @override
  String get causeNoRoute =>
      'No way of reaching the other device worked, directly or through the relay.';

  @override
  String get causeRelay =>
      'The relay server could not be reached or is not set up, or the two devices do not use the same one.';

  @override
  String get causeProtocol =>
      'The other device sent something this version does not understand. Updating TotalDrop on both devices may help.';

  @override
  String get causeDiskFull => 'There is not enough free space on this device.';

  @override
  String get causeNoPermission =>
      'TotalDrop is not allowed to read or write a file or folder it needs.';

  @override
  String get causeNotFound =>
      'A file or folder could not be found. It may have been moved or deleted.';

  @override
  String get causeReadOnly =>
      'The folder is on a disk that cannot be written to.';

  @override
  String get causeFileChanged =>
      'A file changed while it was being sent. Send it again once it is saved.';

  @override
  String get technicalDetail =>
      'Technical message in English, to include when you report a problem:';

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
      'The rendezvous server, the relay and Cloudflare\'s STUN server see your IP address, and so does the other device when you connect directly (unless privacy mode is on). None of them sees file names, contents or keys. Our servers keep no logs of their own, but Cloudflare, which runs them, keeps a sample of requests with IP addresses for 31 days. Files go directly between the devices when they can, otherwise through the relay; they are encrypted end to end either way.';

  @override
  String get privacyMode => 'Hide my address from the other device';

  @override
  String get privacyModeHint =>
      'Privacy mode: everything goes through the relay, so the other device never learns your IP address. Slower, and the other device must use the same relay (the default one does).';

  @override
  String get privacyModeConfirmTitle => 'Turn on privacy mode?';

  @override
  String get privacyModeConfirmBody =>
      'Transfers will be slower, because nothing goes directly between the devices. The rendezvous server and the relay still see your IP address; they never see your files or keys.';

  @override
  String get privacyModeTurnOn => 'Turn on';

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
      'This translation is being reviewed. Help improve it on GitHub.';

  @override
  String get settingsWordsEnglish =>
      'Codes and verification words are in English in every language, so two devices set to different languages show the same words.';

  @override
  String get sendDropHere => 'Drop files here';

  @override
  String get sendDropHint => 'You can also drag files onto this window.';

  @override
  String get keepAwakeReason => 'TotalDrop is sending or receiving files';

  @override
  String get sendQrHint => 'Or scan this code with the other device\'s camera.';

  @override
  String get sendWaitingAccept =>
      'Check the words, then wait for the receiver to accept.';

  @override
  String get receiveFromImage => 'Read QR from image';

  @override
  String get receiveQrNotFound => 'There is no TotalDrop code in that image.';

  @override
  String get pickTitle => 'Choose files to send';

  @override
  String get pickImageTitle => 'Choose an image with a QR code';

  @override
  String get openFolder => 'Open folder';

  @override
  String get back => 'Back';

  @override
  String get codeSuggestions => 'Suggestions';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }

  @override
  String get editUndo => 'Undo';

  @override
  String get editRedo => 'Redo';

  @override
  String get editCut => 'Cut';

  @override
  String get editCopy => 'Copy';

  @override
  String get editPaste => 'Paste';

  @override
  String get editDelete => 'Delete';

  @override
  String get editSelectAll => 'Select All';

  @override
  String get transferNoticeChannel => 'Transfers';

  @override
  String get transferNoticeSending => 'Sending with TotalDrop';

  @override
  String get transferNoticeReceiving => 'Receiving with TotalDrop';

  @override
  String get transferNoticeBody => 'Keeps going while you use other apps.';

  @override
  String get shareBusy =>
      'A transfer is still running. Wait for it to finish or stop it, then share again.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$count shared items could not be read and were left out.',
      one: 'One shared item could not be read and was left out.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'File transfer';

  @override
  String get desktopComment =>
      'Send files straight to another device, end-to-end encrypted';

  @override
  String get desktopKeywords => 'send;receive;share;transfer;files;';

  @override
  String get desktopSendWith => 'Send with TotalDrop';

  @override
  String get metainfoSummary => 'Send files straight to another device';

  @override
  String get metainfoAbout =>
      'TotalDrop sends files and folders from one device to another, end-to-end encrypted. Pick the files, share a short code or a QR code, and the other device receives them. No account, no email, no phone number.';

  @override
  String get metainfoVerified =>
      'Every file is verified on arrival; damaged files are discarded';

  @override
  String get metainfoWords =>
      'Four verification words show that nobody is in the middle';

  @override
  String get metainfoTheme =>
      'Follows your desktop\'s theme: Plasma, Hyprland, Omarchy, GNOME';

  @override
  String get metainfoOnePackage =>
      'Use one Linux package at a time. The Flatpak and the .pkg, .deb or .rpm package share one name, so while one of them is open, starting the other brings the open window forward instead of opening a second one.';

  @override
  String get otherPackageFlatpak =>
      'TotalDrop is already open from the Flatpak, so that window was brought forward. Use one package at a time: quit that window to open this one.';

  @override
  String get otherPackageSystem =>
      'TotalDrop is already open from the .pkg, .deb or .rpm package, so that window was brought forward. Use one package at a time: quit that window to open this one.';

  @override
  String get sendText => 'Send text';

  @override
  String get sendTextHint => 'Type or paste the text to send';

  @override
  String sendTextTooLong(String size) {
    return 'A text can be at most $size. Send a longer one as a file.';
  }

  @override
  String get sendTextDone => 'The text arrived intact.';

  @override
  String get sendClipboard => 'Send clipboard';

  @override
  String get sendClipboardEmpty => 'The clipboard holds no text.';

  @override
  String get receiveOfferText => 'Incoming text';

  @override
  String get receiveOfferTextNote => 'The text is shown here, not saved.';

  @override
  String get receiveTextDone => 'Text received and verified';

  @override
  String get receiveTextDeclined => 'You declined the text.';

  @override
  String get receiveTextHidden =>
      'The text holds hidden characters, shown as U+…. Copying gives the text exactly as it came.';

  @override
  String get sendTextHidden =>
      'The text holds hidden characters, such as direction marks. They are sent with it.';

  @override
  String get sendClipboardKept =>
      'The text already typed is kept; the clipboard was not used.';

  @override
  String get sendTextShortened =>
      'Only the beginning of the text is shown; all of it is sent.';

  @override
  String get receiveTextShortened =>
      'Only the beginning of the text is shown. Copy text copies all of it.';

  @override
  String get copyText => 'Copy text';

  @override
  String get textCopied => 'Text copied';

  @override
  String get errorTextDamaged =>
      'The text did not arrive intact, so it is not shown.';

  @override
  String get setupBackgroundTitle => 'Keep transfers going in the background';

  @override
  String get setupBackgroundBody =>
      'So that a transfer keeps going when you switch apps or lock the screen, let TotalDrop run without battery restrictions. TotalDrop works in the background only while a transfer you started is running.';

  @override
  String get setupBackgroundAllowed =>
      'Android\'s battery optimisation is off for TotalDrop.';

  @override
  String get setupBackgroundRestricted =>
      'Android\'s battery optimisation is on for TotalDrop: a transfer may stop when you switch apps.';

  @override
  String get setupBackgroundBattery => 'Open battery settings';

  @override
  String get setupBackgroundBatteryHint =>
      'On TotalDrop\'s app info page, open Battery and allow unrestricted use. The names of these settings differ from phone to phone.';

  @override
  String setupBackgroundMaker(String maker) {
    return '$maker phones have a battery saver of their own';
  }

  @override
  String get setupBackgroundMakerBody =>
      'Open TotalDrop\'s battery saver too and choose \"No restrictions\". If the phone has Autostart, turn it on for TotalDrop.';

  @override
  String get setupBackgroundMakerOpen => 'Open TotalDrop\'s battery saver';

  @override
  String get setupBackgroundAutostart => 'Open Autostart';

  @override
  String get setupBackgroundSkip => 'Not now';

  @override
  String get settingsBackground => 'Background transfers';

  @override
  String get settingsBackgroundOpen => 'Battery settings';
}
