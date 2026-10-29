// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Bengali Bangla (`bn`).
class AppLocalizationsBn extends AppLocalizations {
  AppLocalizationsBn([String locale = 'bn']) : super(locale);

  @override
  String get appTagline => 'যেকোনো ফাইল, যে কাউকে, যেকোনো জায়গায়।';

  @override
  String get homeSendTitle => 'পাঠান';

  @override
  String get homeSendBody => 'ফাইল বেছে নিন আর একটি ছোট কোড শেয়ার করুন';

  @override
  String get homeReceiveTitle => 'গ্রহণ করুন';

  @override
  String get homeReceiveBody => 'কোড লিখুন আর ফাইলগুলো সেভ করুন';

  @override
  String get homeFooter =>
      'এন্ড-টু-এন্ড এনক্রিপ্টেড। কোনো অ্যাকাউন্ট, ইমেল বা ফোন নম্বর লাগে না।';

  @override
  String get settings => 'সেটিংস';

  @override
  String get sendTitle => 'ফাইল পাঠান';

  @override
  String get sendAddFiles => 'ফাইল যোগ করুন';

  @override
  String get sendEmpty => 'এখনও কিছু বেছে নেওয়া হয়নি';

  @override
  String get sendEmptyHint =>
      'যে ফাইলগুলো পাঠাতে চান সেগুলো যোগ করুন। প্রাপক ঠিক এগুলোই পাবেন, আর কিছু নয়।';

  @override
  String get sendStrongCode => 'আরও শক্তিশালী কোড';

  @override
  String get sendStrongCodeHint =>
      'দুটির বদলে তিনটি শব্দ: অনুমান করা অনেক বেশি কঠিন';

  @override
  String get sendCreateCode => 'কোড তৈরি করুন';

  @override
  String get sendCounting => 'গোনা হচ্ছে…';

  @override
  String get sendRemove => 'সরান';

  @override
  String get sendPreparing => 'আপনার ফাইলগুলো প্রস্তুত করা হচ্ছে…';

  @override
  String get sendYourCode => 'আপনার কোড';

  @override
  String get sendCodeHint =>
      'অন্য ডিভাইসে TotalDrop খুলুন, গ্রহণ করুন-এ ট্যাপ করুন আর এই কোডটি লিখুন।';

  @override
  String get sendWaiting => 'প্রাপকের জন্য অপেক্ষা করা হচ্ছে…';

  @override
  String get sendCopy => 'কপি করুন';

  @override
  String get sendCopied => 'কোড কপি হয়েছে';

  @override
  String get sendShare => 'শেয়ার করুন';

  @override
  String sendShareText(String code) {
    return 'TotalDrop দিয়ে আমার ফাইলগুলো নিন। কোড: $code';
  }

  @override
  String get sendSending => 'পাঠানো হচ্ছে';

  @override
  String get sendDone => 'পাঠানো ও যাচাই করা হয়েছে';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: '$countStringটি ফাইলই অক্ষত অবস্থায় পৌঁছেছে।',
      one: 'ফাইলটি অক্ষত অবস্থায় পৌঁছেছে।',
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
      other:
          '$countStringটি ফাইল অক্ষত অবস্থায় পৌঁছায়নি, তাই বাতিল করা হয়েছে।',
      one: 'একটি ফাইল অক্ষত অবস্থায় পৌঁছায়নি, তাই বাতিল করা হয়েছে।',
    );
    return '$_temp0';
  }

  @override
  String itemsAndSize(int count, String size) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return '$countStringটি আইটেম · $size';
  }

  @override
  String get receiveTitle => 'ফাইল গ্রহণ করুন';

  @override
  String get receiveCodeLabel => 'কোড';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'প্রেরকের কাছে তাঁর কোড চেয়ে নিন।';

  @override
  String get receiveCodeInvalid =>
      'এটি কোডের মতো দেখাচ্ছে না। কোড একটি সংখ্যা দিয়ে শুরু হয়, তারপর দুটি বা তিনটি শব্দ থাকে।';

  @override
  String get receivePaste => 'পেস্ট করুন';

  @override
  String get receiveConnect => 'সংযোগ করুন';

  @override
  String get receiveConnecting => 'নিরাপদে সংযোগ করা হচ্ছে…';

  @override
  String get receiveOffer => 'আসা ফাইল';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    return '$filesStringটি ফাইল · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'এবং আরও $countStringটি';
  }

  @override
  String receiveSaveTo(String folder) {
    return '$folder-এ সেভ করা হয়েছে';
  }

  @override
  String get receiveAccept => 'গ্রহণ করুন';

  @override
  String get receiveDecline => 'প্রত্যাখ্যান করুন';

  @override
  String get receiveDeclined => 'আপনি ফাইলগুলো প্রত্যাখ্যান করেছেন।';

  @override
  String get receiveReceiving => 'গ্রহণ করা হচ্ছে';

  @override
  String get receiveDone => 'গ্রহণ ও যাচাই করা হয়েছে';

  @override
  String get verificationTitle => 'যাচাইয়ের শব্দ';

  @override
  String get verificationHint =>
      'অন্য ব্যক্তিও এই একই চারটি শব্দ দেখছেন। শব্দগুলো আলাদা হলে থামুন।';

  @override
  String get sameNetworkNote =>
      'ইন্টারনেটের মাধ্যমে কাজ করে। ডিভাইসগুলো সরাসরি একে অপরের কাছে পৌঁছাতে না পারলে এনক্রিপ্ট করা ট্রান্সফার রিলে সার্ভারের মধ্য দিয়ে যায়, যা এটি পড়তে পারে না।';

  @override
  String get done => 'সম্পন্ন';

  @override
  String get tryAgain => 'আবার চেষ্টা করুন';

  @override
  String get cancel => 'বাতিল';

  @override
  String get cancelTransferTitle => 'এই ট্রান্সফার থামাবেন?';

  @override
  String get cancelTransferBody =>
      'অন্য পক্ষকে জানানো হবে যে ট্রান্সফার থেমে গেছে।';

  @override
  String get keepGoing => 'চালিয়ে যান';

  @override
  String get stop => 'থামান';

  @override
  String get moreInfo => 'বিস্তারিত';

  @override
  String get errorWrongCode =>
      'কোড মেলেনি। হয় এটি ভুল টাইপ হয়েছে, নয়তো কেউ ট্রান্সফারটি মাঝপথে ধরার চেষ্টা করেছে। কিছুই পাঠানো হয়নি। নতুন কোড দিয়ে আবার শুরু করুন।';

  @override
  String get errorPeerLeft => 'ট্রান্সফার শেষ হওয়ার আগেই অন্য পক্ষ চলে গেছে।';

  @override
  String get errorDeclined => 'প্রাপক ফাইলগুলো প্রত্যাখ্যান করেছেন।';

  @override
  String get errorCancelled => 'অন্য পক্ষ ট্রান্সফার থামিয়ে দিয়েছে।';

  @override
  String get errorStopped => 'এই ডিভাইসে থামানো হয়েছে।';

  @override
  String get errorInvalidCode => 'এই কোডটি বৈধ নয়।';

  @override
  String get errorServer =>
      'TotalDrop সার্ভারে পৌঁছানো যায়নি। আপনার ইন্টারনেট সংযোগ দেখুন।';

  @override
  String get errorConnect =>
      'অন্য ডিভাইসের সঙ্গে সংযোগ করা যায়নি। দুটি ডিভাইসই অনলাইনে আছে কিনা দেখে আবার চেষ্টা করুন।';

  @override
  String get errorInvalidOffer =>
      'প্রস্তাবটি একটি নিরাপত্তা নিয়ম ভেঙেছে, তাই এটি প্রত্যাখ্যান করা হয়েছে।';

  @override
  String get errorStorage => 'একটি ফাইল পড়া বা সেভ করা যায়নি।';

  @override
  String get errorOther => 'কিছু একটা সমস্যা হয়েছে।';

  @override
  String get settingsServer => 'সংযোগকারী সার্ভার';

  @override
  String get settingsServerHelp =>
      'একই কোডওয়ালা দুটি ডিভাইসকে একে অপরকে খুঁজে পেতে সাহায্য করে। এটি কখনও আপনার ফাইল বা কী দেখে না।';

  @override
  String get settingsServerReset => 'ডিফল্ট ব্যবহার করুন';

  @override
  String get settingsServerInvalid => 'wss:// দিয়ে শুরু হতে হবে';

  @override
  String get settingsSave => 'সেভ করুন';

  @override
  String get settingsAbout => 'সম্পর্কে';

  @override
  String get settingsAboutBody =>
      'TotalDrop বিনামূল্যে ও ওপেন সোর্স, অনুদানে চলে। লাইসেন্স GPL-3.0-or-later।';

  @override
  String get settingsPrivacy => 'গোপনীয়তা';

  @override
  String get settingsPrivacyBody =>
      'সংযোগকারী সার্ভার, রিলে সার্ভার ও Cloudflare-এর STUN সার্ভার আপনার IP ঠিকানা দেখে, আর সরাসরি সংযোগ হলে অন্য ডিভাইসও দেখে (গোপনীয়তা মোড চালু না থাকলে)। এদের কেউই ফাইলের নাম, বিষয়বস্তু বা কী দেখে না, আর আমাদের সার্ভার কোনো লগ রাখে না। সম্ভব হলে ফাইল সরাসরি ডিভাইসগুলোর মধ্যে যায়, না হলে রিলে সার্ভারের মধ্য দিয়ে; দুই ক্ষেত্রেই সেগুলো এন্ড-টু-এন্ড এনক্রিপ্টেড থাকে।';

  @override
  String get privacyMode => 'অন্য ডিভাইস থেকে আমার ঠিকানা লুকান';

  @override
  String get privacyModeHint =>
      'গোপনীয়তা মোড: সবকিছু রিলে সার্ভারের মধ্য দিয়ে যায়, তাই অন্য ডিভাইস কখনও আপনার IP ঠিকানা জানতে পারে না। এটি ধীর, আর অন্য ডিভাইসকেও একই রিলে সার্ভার ব্যবহার করতে হবে (ডিফল্ট সার্ভার তাই করে)।';

  @override
  String get privacyModeConfirmTitle => 'গোপনীয়তা মোড চালু করবেন?';

  @override
  String get privacyModeConfirmBody =>
      'ট্রান্সফার ধীর হবে, কারণ ডিভাইসগুলোর মধ্যে কিছুই সরাসরি যায় না। সংযোগকারী সার্ভার ও রিলে সার্ভার তখনও আপনার IP ঠিকানা দেখে; তারা কখনও আপনার ফাইল বা কী দেখে না।';

  @override
  String get privacyModeTurnOn => 'চালু করুন';

  @override
  String settingsVersion(String version) {
    return 'সংস্করণ $version';
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
      'এই অনুবাদটি পর্যালোচনা করা হচ্ছে। Weblate-এ এটি আরও ভালো করতে সাহায্য করুন।';

  @override
  String get sendDropHere => 'ফাইলগুলো এখানে ছাড়ুন';

  @override
  String get sendDropHint => 'আপনি ফাইলগুলো এই উইন্ডোতে টেনেও আনতে পারেন।';

  @override
  String get sendQrHint =>
      'অথবা অন্য ডিভাইসের ক্যামেরা দিয়ে এই কোডটি স্ক্যান করুন।';

  @override
  String get sendWaitingAccept =>
      'শব্দগুলো মিলিয়ে দেখুন, তারপর প্রাপকের গ্রহণ করার জন্য অপেক্ষা করুন।';

  @override
  String get receiveFromImage => 'ছবি থেকে QR পড়ুন';

  @override
  String get receiveQrNotFound => 'এই ছবিতে কোনো TotalDrop কোড নেই।';

  @override
  String get pickTitle => 'পাঠানোর জন্য ফাইল বেছে নিন';

  @override
  String get pickImageTitle => 'QR কোডসহ একটি ছবি বেছে নিন';

  @override
  String get openFolder => 'ফোল্ডার খুলুন';

  @override
  String get back => 'ফিরে যান';

  @override
  String get codeSuggestions => 'পরামর্শ';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
