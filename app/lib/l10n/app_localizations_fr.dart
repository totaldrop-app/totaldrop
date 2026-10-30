// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for French (`fr`).
class AppLocalizationsFr extends AppLocalizations {
  AppLocalizationsFr([String locale = 'fr']) : super(locale);

  @override
  String get appTagline =>
      'N\'importe quel fichier, à n\'importe qui, n\'importe où.';

  @override
  String get homeSendTitle => 'Envoyer';

  @override
  String get homeSendBody =>
      'Choisissez des fichiers et partagez un code court';

  @override
  String get homeReceiveTitle => 'Recevoir';

  @override
  String get homeReceiveBody => 'Saisissez un code et enregistrez les fichiers';

  @override
  String get homeFooter =>
      'Chiffré de bout en bout. Pas de compte, pas d\'e-mail, pas de numéro de téléphone.';

  @override
  String get settings => 'Paramètres';

  @override
  String get sendTitle => 'Envoyer des fichiers';

  @override
  String get sendAddFiles => 'Ajouter des fichiers';

  @override
  String get sendEmpty => 'Rien n\'est encore sélectionné';

  @override
  String get sendEmptyHint =>
      'Ajoutez les fichiers à envoyer. Le destinataire reçoit exactement ceux-ci, rien d\'autre.';

  @override
  String get sendStrongCode => 'Code plus fort';

  @override
  String get sendStrongCodeHint =>
      'Trois mots au lieu de deux : bien plus difficile à deviner';

  @override
  String get sendCreateCode => 'Créer le code';

  @override
  String get sendCounting => 'Calcul en cours…';

  @override
  String get sendRemove => 'Retirer';

  @override
  String get sendPreparing => 'Préparation de vos fichiers…';

  @override
  String get sendYourCode => 'Votre code';

  @override
  String get sendCodeHint =>
      'Sur l\'autre appareil, ouvrez TotalDrop, appuyez sur Recevoir et saisissez ce code.';

  @override
  String get sendWaiting => 'En attente du destinataire…';

  @override
  String get sendCopy => 'Copier';

  @override
  String get sendCopied => 'Code copié';

  @override
  String get sendShare => 'Partager';

  @override
  String sendShareText(String code) {
    return 'Récupérez mes fichiers avec TotalDrop. Code : $code';
  }

  @override
  String get sendSending => 'Envoi en cours';

  @override
  String get sendDone => 'Envoyé et vérifié';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Les $countString fichiers sont arrivés intacts.',
      many: 'Les $countString de fichiers sont arrivés intacts.',
      one: '$countString fichier est arrivé intact.',
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
          '$countString fichiers ne sont pas arrivés intacts et ont été supprimés.',
      many:
          '$countString de fichiers ne sont pas arrivés intacts et ont été supprimés.',
      one: '$countString fichier n\'est pas arrivé intact et a été supprimé.',
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
      other: '$countString éléments',
      many: '$countString d\'éléments',
      one: '$countString élément',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Recevoir des fichiers';

  @override
  String get receiveCodeLabel => 'Code';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Demandez son code à l\'expéditeur.';

  @override
  String get receiveCodeInvalid =>
      'Cela ne ressemble pas à un code. Un code commence par un nombre, suivi de deux ou trois mots.';

  @override
  String get receivePaste => 'Coller';

  @override
  String get receiveConnect => 'Se connecter';

  @override
  String get receiveConnecting => 'Connexion sécurisée…';

  @override
  String get receiveOffer => 'Fichiers entrants';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString fichiers',
      many: '$filesString de fichiers',
      one: '$filesString fichier',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'et $countString de plus';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Enregistré dans $folder';
  }

  @override
  String get receiveAccept => 'Accepter';

  @override
  String get receiveDecline => 'Refuser';

  @override
  String get receiveDeclined => 'Vous avez refusé les fichiers.';

  @override
  String get receiveReceiving => 'Réception en cours';

  @override
  String get receiveDone => 'Reçu et vérifié';

  @override
  String get verificationTitle => 'Mots de vérification';

  @override
  String get verificationHint =>
      'L\'autre personne voit les quatre mêmes mots. S\'ils diffèrent, arrêtez.';

  @override
  String get sameNetworkNote =>
      'Fonctionne via Internet. Quand les appareils ne peuvent pas se joindre directement, le transfert chiffré passe par le serveur relais, qui ne peut pas le lire.';

  @override
  String get done => 'Terminé';

  @override
  String get tryAgain => 'Réessayer';

  @override
  String get cancel => 'Annuler';

  @override
  String get cancelTransferTitle => 'Arrêter ce transfert ?';

  @override
  String get cancelTransferBody =>
      'L\'autre appareil sera informé de l\'arrêt du transfert.';

  @override
  String get keepGoing => 'Continuer';

  @override
  String get stop => 'Arrêter';

  @override
  String get moreInfo => 'Détails';

  @override
  String get errorWrongCode =>
      'Le code ne correspondait pas. Il a été mal saisi, ou quelqu\'un a tenté d\'intercepter le transfert. Rien n\'a été envoyé. Recommencez avec un nouveau code.';

  @override
  String get errorPeerLeft =>
      'L\'autre appareil s\'est déconnecté avant la fin du transfert.';

  @override
  String get errorDeclined => 'Le destinataire a refusé les fichiers.';

  @override
  String get errorCancelled => 'L\'autre appareil a arrêté le transfert.';

  @override
  String get errorStopped => 'Arrêté sur cet appareil.';

  @override
  String get errorInvalidCode => 'Ce code n\'est pas valide.';

  @override
  String get errorServer =>
      'Impossible de joindre le serveur TotalDrop. Vérifiez votre connexion Internet.';

  @override
  String get errorConnect =>
      'Impossible de se connecter à l\'autre appareil. Vérifiez que les deux appareils sont en ligne et réessayez.';

  @override
  String get errorInvalidOffer =>
      'L\'offre enfreignait une règle de sécurité, elle a donc été refusée.';

  @override
  String get errorStorage => 'Un fichier n\'a pas pu être lu ou enregistré.';

  @override
  String get errorOther => 'Un problème est survenu.';

  @override
  String get settingsServer => 'Serveur de rendez-vous';

  @override
  String get settingsServerHelp =>
      'Aide deux appareils ayant le même code à se trouver. Il ne voit jamais vos fichiers ni vos clés.';

  @override
  String get settingsServerReset => 'Utiliser le serveur par défaut';

  @override
  String get settingsServerInvalid => 'Doit commencer par wss://';

  @override
  String get settingsSave => 'Enregistrer';

  @override
  String get settingsAbout => 'À propos';

  @override
  String get settingsAboutBody =>
      'TotalDrop est gratuit et open source, financé par des dons. Sous licence GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Confidentialité';

  @override
  String get settingsPrivacyBody =>
      'Le serveur de rendez-vous, le serveur relais et le serveur STUN de Cloudflare voient votre adresse IP, tout comme l\'autre appareil lors d\'une connexion directe (sauf si le mode confidentialité est activé). Aucun d\'eux ne voit les noms de fichiers, leur contenu ni les clés, et nos serveurs ne conservent aucun journal. Les fichiers passent directement d\'un appareil à l\'autre quand c\'est possible, sinon par le serveur relais ; ils sont chiffrés de bout en bout dans les deux cas.';

  @override
  String get privacyMode => 'Masquer mon adresse à l\'autre appareil';

  @override
  String get privacyModeHint =>
      'Mode confidentialité : tout passe par le serveur relais, si bien que l\'autre appareil ne connaît jamais votre adresse IP. Plus lent, et l\'autre appareil doit utiliser le même serveur relais (c\'est le cas de celui par défaut).';

  @override
  String get privacyModeConfirmTitle => 'Activer le mode confidentialité ?';

  @override
  String get privacyModeConfirmBody =>
      'Les transferts seront plus lents, car rien ne passe directement entre les appareils. Le serveur de rendez-vous et le serveur relais voient toujours votre adresse IP ; ils ne voient jamais vos fichiers ni vos clés.';

  @override
  String get privacyModeTurnOn => 'Activer';

  @override
  String settingsVersion(String version) {
    return 'Version $version';
  }

  @override
  String sizeBytes(String value) {
    return '$value o';
  }

  @override
  String sizeKB(String value) {
    return '$value ko';
  }

  @override
  String sizeMB(String value) {
    return '$value Mo';
  }

  @override
  String sizeGB(String value) {
    return '$value Go';
  }

  @override
  String sizeTB(String value) {
    return '$value To';
  }

  @override
  String sizePB(String value) {
    return '$value Po';
  }

  @override
  String get draftTranslation =>
      'Cette traduction est en cours de relecture. Aidez à l\'améliorer sur GitHub.';

  @override
  String get sendDropHere => 'Déposez les fichiers ici';

  @override
  String get sendDropHint =>
      'Vous pouvez aussi faire glisser des fichiers sur cette fenêtre.';

  @override
  String get sendQrHint =>
      'Ou scannez ce code avec l\'appareil photo de l\'autre appareil.';

  @override
  String get sendWaitingAccept =>
      'Vérifiez les mots, puis attendez que le destinataire accepte.';

  @override
  String get receiveFromImage => 'Lire un QR code depuis une image';

  @override
  String get receiveQrNotFound =>
      'Cette image ne contient pas de code TotalDrop.';

  @override
  String get pickTitle => 'Choisir les fichiers à envoyer';

  @override
  String get pickImageTitle => 'Choisir une image contenant un QR code';

  @override
  String get openFolder => 'Ouvrir le dossier';

  @override
  String get back => 'Retour';

  @override
  String get codeSuggestions => 'Suggestions';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString %';
  }
}
