import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_tr.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations)!;
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('tr'),
  ];

  /// Short slogan under the app name on the home screen
  ///
  /// In en, this message translates to:
  /// **'Any file, to anyone, anywhere.'**
  String get appTagline;

  /// No description provided for @homeSendTitle.
  ///
  /// In en, this message translates to:
  /// **'Send'**
  String get homeSendTitle;

  /// No description provided for @homeSendBody.
  ///
  /// In en, this message translates to:
  /// **'Pick files and share a short code'**
  String get homeSendBody;

  /// No description provided for @homeReceiveTitle.
  ///
  /// In en, this message translates to:
  /// **'Receive'**
  String get homeReceiveTitle;

  /// No description provided for @homeReceiveBody.
  ///
  /// In en, this message translates to:
  /// **'Enter a code and save the files'**
  String get homeReceiveBody;

  /// No description provided for @homeFooter.
  ///
  /// In en, this message translates to:
  /// **'End-to-end encrypted. No account, no email, no phone number.'**
  String get homeFooter;

  /// No description provided for @settings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settings;

  /// No description provided for @sendTitle.
  ///
  /// In en, this message translates to:
  /// **'Send files'**
  String get sendTitle;

  /// No description provided for @sendAddFiles.
  ///
  /// In en, this message translates to:
  /// **'Add files'**
  String get sendAddFiles;

  /// No description provided for @sendEmpty.
  ///
  /// In en, this message translates to:
  /// **'Nothing selected yet'**
  String get sendEmpty;

  /// No description provided for @sendEmptyHint.
  ///
  /// In en, this message translates to:
  /// **'Add the files you want to send. The receiver gets exactly these, nothing else.'**
  String get sendEmptyHint;

  /// No description provided for @sendStrongCode.
  ///
  /// In en, this message translates to:
  /// **'Stronger code'**
  String get sendStrongCode;

  /// No description provided for @sendStrongCodeHint.
  ///
  /// In en, this message translates to:
  /// **'Three words instead of two: much harder to guess'**
  String get sendStrongCodeHint;

  /// No description provided for @sendCreateCode.
  ///
  /// In en, this message translates to:
  /// **'Create code'**
  String get sendCreateCode;

  /// No description provided for @sendRemove.
  ///
  /// In en, this message translates to:
  /// **'Remove'**
  String get sendRemove;

  /// No description provided for @sendPreparing.
  ///
  /// In en, this message translates to:
  /// **'Preparing your files…'**
  String get sendPreparing;

  /// No description provided for @sendYourCode.
  ///
  /// In en, this message translates to:
  /// **'Your code'**
  String get sendYourCode;

  /// No description provided for @sendCodeHint.
  ///
  /// In en, this message translates to:
  /// **'On the other device, open TotalDrop, tap Receive and enter this code.'**
  String get sendCodeHint;

  /// No description provided for @sendWaiting.
  ///
  /// In en, this message translates to:
  /// **'Waiting for the receiver…'**
  String get sendWaiting;

  /// No description provided for @sendCopy.
  ///
  /// In en, this message translates to:
  /// **'Copy'**
  String get sendCopy;

  /// No description provided for @sendCopied.
  ///
  /// In en, this message translates to:
  /// **'Code copied'**
  String get sendCopied;

  /// No description provided for @sendShare.
  ///
  /// In en, this message translates to:
  /// **'Share'**
  String get sendShare;

  /// No description provided for @sendShareText.
  ///
  /// In en, this message translates to:
  /// **'Get my files with TotalDrop. Code: {code}'**
  String sendShareText(String code);

  /// No description provided for @sendSending.
  ///
  /// In en, this message translates to:
  /// **'Sending'**
  String get sendSending;

  /// No description provided for @sendDone.
  ///
  /// In en, this message translates to:
  /// **'Sent and verified'**
  String get sendDone;

  /// No description provided for @sendDoneBody.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{The file arrived intact.} other{All {count} files arrived intact.}}'**
  String sendDoneBody(int count);

  /// No description provided for @sendSomeFailed.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, =1{One file did not arrive intact and was discarded.} other{{count} files did not arrive intact and were discarded.}}'**
  String sendSomeFailed(int count);

  /// No description provided for @itemsAndSize.
  ///
  /// In en, this message translates to:
  /// **'{count, plural, one{{count} item} other{{count} items}} · {size}'**
  String itemsAndSize(int count, String size);

  /// No description provided for @receiveTitle.
  ///
  /// In en, this message translates to:
  /// **'Receive files'**
  String get receiveTitle;

  /// No description provided for @receiveCodeLabel.
  ///
  /// In en, this message translates to:
  /// **'Code'**
  String get receiveCodeLabel;

  /// No description provided for @receiveCodeHint.
  ///
  /// In en, this message translates to:
  /// **'7-harbor-rocket'**
  String get receiveCodeHint;

  /// No description provided for @receiveCodeHelp.
  ///
  /// In en, this message translates to:
  /// **'Ask the sender for their code.'**
  String get receiveCodeHelp;

  /// No description provided for @receiveCodeInvalid.
  ///
  /// In en, this message translates to:
  /// **'That does not look like a code. It starts with a number, then two or three words.'**
  String get receiveCodeInvalid;

  /// No description provided for @receivePaste.
  ///
  /// In en, this message translates to:
  /// **'Paste'**
  String get receivePaste;

  /// No description provided for @receiveConnect.
  ///
  /// In en, this message translates to:
  /// **'Connect'**
  String get receiveConnect;

  /// No description provided for @receiveConnecting.
  ///
  /// In en, this message translates to:
  /// **'Connecting securely…'**
  String get receiveConnecting;

  /// No description provided for @receiveOffer.
  ///
  /// In en, this message translates to:
  /// **'Incoming files'**
  String get receiveOffer;

  /// No description provided for @receiveOfferSummary.
  ///
  /// In en, this message translates to:
  /// **'{files, plural, one{{files} file} other{{files} files}} · {size}'**
  String receiveOfferSummary(int files, String size);

  /// No description provided for @receiveAndMore.
  ///
  /// In en, this message translates to:
  /// **'and {count} more'**
  String receiveAndMore(int count);

  /// No description provided for @receiveSaveTo.
  ///
  /// In en, this message translates to:
  /// **'Saved to {folder}'**
  String receiveSaveTo(String folder);

  /// No description provided for @receiveAccept.
  ///
  /// In en, this message translates to:
  /// **'Accept'**
  String get receiveAccept;

  /// No description provided for @receiveDecline.
  ///
  /// In en, this message translates to:
  /// **'Decline'**
  String get receiveDecline;

  /// No description provided for @receiveDeclined.
  ///
  /// In en, this message translates to:
  /// **'You declined the files.'**
  String get receiveDeclined;

  /// No description provided for @receiveReceiving.
  ///
  /// In en, this message translates to:
  /// **'Receiving'**
  String get receiveReceiving;

  /// No description provided for @receiveDone.
  ///
  /// In en, this message translates to:
  /// **'Received and verified'**
  String get receiveDone;

  /// No description provided for @verificationTitle.
  ///
  /// In en, this message translates to:
  /// **'Verification words'**
  String get verificationTitle;

  /// No description provided for @verificationHint.
  ///
  /// In en, this message translates to:
  /// **'The other person sees the same four words. If they differ, stop.'**
  String get verificationHint;

  /// No description provided for @sameNetworkNote.
  ///
  /// In en, this message translates to:
  /// **'Works across the internet. When the devices cannot reach each other directly, the encrypted transfer goes through the relay, which cannot read it.'**
  String get sameNetworkNote;

  /// No description provided for @done.
  ///
  /// In en, this message translates to:
  /// **'Done'**
  String get done;

  /// No description provided for @tryAgain.
  ///
  /// In en, this message translates to:
  /// **'Try again'**
  String get tryAgain;

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// No description provided for @cancelTransferTitle.
  ///
  /// In en, this message translates to:
  /// **'Stop this transfer?'**
  String get cancelTransferTitle;

  /// No description provided for @cancelTransferBody.
  ///
  /// In en, this message translates to:
  /// **'The other side will be told the transfer stopped.'**
  String get cancelTransferBody;

  /// No description provided for @keepGoing.
  ///
  /// In en, this message translates to:
  /// **'Keep going'**
  String get keepGoing;

  /// No description provided for @stop.
  ///
  /// In en, this message translates to:
  /// **'Stop'**
  String get stop;

  /// No description provided for @moreInfo.
  ///
  /// In en, this message translates to:
  /// **'Details'**
  String get moreInfo;

  /// No description provided for @errorWrongCode.
  ///
  /// In en, this message translates to:
  /// **'The code did not match. It was mistyped, or someone tried to intercept the transfer. Nothing was sent. Start again with a new code.'**
  String get errorWrongCode;

  /// No description provided for @errorPeerLeft.
  ///
  /// In en, this message translates to:
  /// **'The other side left before the transfer finished.'**
  String get errorPeerLeft;

  /// No description provided for @errorDeclined.
  ///
  /// In en, this message translates to:
  /// **'The receiver declined the files.'**
  String get errorDeclined;

  /// No description provided for @errorCancelled.
  ///
  /// In en, this message translates to:
  /// **'The other side stopped the transfer.'**
  String get errorCancelled;

  /// No description provided for @errorInvalidCode.
  ///
  /// In en, this message translates to:
  /// **'That code is not valid.'**
  String get errorInvalidCode;

  /// No description provided for @errorServer.
  ///
  /// In en, this message translates to:
  /// **'Could not reach the TotalDrop server. Check your internet connection.'**
  String get errorServer;

  /// No description provided for @errorConnect.
  ///
  /// In en, this message translates to:
  /// **'Could not connect to the other device. Check that both devices are online and try again.'**
  String get errorConnect;

  /// No description provided for @errorInvalidOffer.
  ///
  /// In en, this message translates to:
  /// **'The offer broke a safety rule, so it was refused.'**
  String get errorInvalidOffer;

  /// No description provided for @errorStorage.
  ///
  /// In en, this message translates to:
  /// **'A file could not be read or saved.'**
  String get errorStorage;

  /// No description provided for @errorOther.
  ///
  /// In en, this message translates to:
  /// **'Something went wrong.'**
  String get errorOther;

  /// No description provided for @settingsServer.
  ///
  /// In en, this message translates to:
  /// **'Rendezvous server'**
  String get settingsServer;

  /// No description provided for @settingsServerHelp.
  ///
  /// In en, this message translates to:
  /// **'Helps two devices with the same code find each other. It never sees your files or keys.'**
  String get settingsServerHelp;

  /// No description provided for @settingsServerReset.
  ///
  /// In en, this message translates to:
  /// **'Use the default'**
  String get settingsServerReset;

  /// No description provided for @settingsServerInvalid.
  ///
  /// In en, this message translates to:
  /// **'Must start with wss://'**
  String get settingsServerInvalid;

  /// No description provided for @settingsSave.
  ///
  /// In en, this message translates to:
  /// **'Save'**
  String get settingsSave;

  /// No description provided for @settingsAbout.
  ///
  /// In en, this message translates to:
  /// **'About'**
  String get settingsAbout;

  /// No description provided for @settingsAboutBody.
  ///
  /// In en, this message translates to:
  /// **'TotalDrop is free and open source, funded by donations. Licensed GPL-3.0-or-later.'**
  String get settingsAboutBody;

  /// No description provided for @settingsPrivacy.
  ///
  /// In en, this message translates to:
  /// **'Privacy'**
  String get settingsPrivacy;

  /// No description provided for @settingsPrivacyBody.
  ///
  /// In en, this message translates to:
  /// **'The rendezvous server sees your IP address and when you connect, but no file names, contents or keys. It keeps no logs. Files go directly between the two devices, encrypted end to end.'**
  String get settingsPrivacyBody;

  /// No description provided for @settingsVersion.
  ///
  /// In en, this message translates to:
  /// **'Version {version}'**
  String settingsVersion(String version);

  /// A size in bytes. value is already formatted for the region
  ///
  /// In en, this message translates to:
  /// **'{value} B'**
  String sizeBytes(String value);

  /// Kilobytes (1000 bytes)
  ///
  /// In en, this message translates to:
  /// **'{value} kB'**
  String sizeKB(String value);

  /// Megabytes
  ///
  /// In en, this message translates to:
  /// **'{value} MB'**
  String sizeMB(String value);

  /// Gigabytes
  ///
  /// In en, this message translates to:
  /// **'{value} GB'**
  String sizeGB(String value);

  /// Terabytes
  ///
  /// In en, this message translates to:
  /// **'{value} TB'**
  String sizeTB(String value);

  /// Petabytes
  ///
  /// In en, this message translates to:
  /// **'{value} PB'**
  String sizePB(String value);

  /// Shown in Settings when some of the text in this language is an unreviewed draft
  ///
  /// In en, this message translates to:
  /// **'This translation is being reviewed. Help improve it on Weblate.'**
  String get draftTranslation;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'tr'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'tr':
      return AppLocalizationsTr();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
