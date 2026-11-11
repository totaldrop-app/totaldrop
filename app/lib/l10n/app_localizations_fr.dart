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
  String get sendAddFailed =>
      'Impossible d\'ajouter les fichiers. Réessayez. Si cela se reproduit, vérifiez qu\'il reste assez d\'espace libre sur cet appareil, ou choisissez les fichiers à un autre emplacement.';

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
  String get errorCodeNotInUse =>
      'Ce code n\'est pas utilisé : il a été mal saisi, a expiré ou a déjà servi. Demandez-en un nouveau à l\'expéditeur.';

  @override
  String get errorCodeExpired =>
      'Le code a expiré avant que quelqu\'un ne l\'utilise. Envoyez à nouveau pour obtenir un nouveau code.';

  @override
  String get errorBadServer =>
      'L\'adresse du serveur n\'est pas valide. Elle doit commencer par wss://.';

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
  String get causeTimedOut =>
      'La connexion ne répond plus. L\'un des appareils a peut-être perdu le réseau ou s\'est mis en veille.';

  @override
  String get causeConnectionLost =>
      'La connexion avec l\'autre appareil a été coupée.';

  @override
  String get causeNoRoute =>
      'Impossible de joindre l\'autre appareil, ni directement ni par le serveur relais.';

  @override
  String get causeRelay =>
      'Le serveur relais est injoignable ou n\'est pas configuré, ou les deux appareils n\'utilisent pas le même.';

  @override
  String get causeProtocol =>
      'L\'autre appareil a envoyé quelque chose que cette version ne comprend pas. Mettre à jour TotalDrop sur les deux appareils peut aider.';

  @override
  String get causeDiskFull =>
      'Il n\'y a pas assez d\'espace libre sur cet appareil.';

  @override
  String get causeNoPermission =>
      'TotalDrop n\'a pas le droit de lire ou d\'écrire un fichier ou un dossier dont il a besoin.';

  @override
  String get causeNotFound =>
      'Un fichier ou un dossier est introuvable. Il a peut-être été déplacé ou supprimé.';

  @override
  String get causeReadOnly =>
      'Le dossier se trouve sur un disque où l\'on ne peut pas écrire.';

  @override
  String get causeFileChanged =>
      'Un fichier a changé pendant son envoi. Renvoyez-le une fois enregistré.';

  @override
  String get technicalDetail =>
      'Message technique en anglais, à joindre quand vous signalez un problème :';

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
  String get settingsWordsEnglish =>
      'Les codes et les mots de vérification restent en anglais dans toutes les langues, pour que deux appareils réglés dans des langues différentes affichent les mêmes mots.';

  @override
  String get sendDropHere => 'Déposez les fichiers ici';

  @override
  String get sendDropHint =>
      'Vous pouvez aussi faire glisser des fichiers sur cette fenêtre.';

  @override
  String get keepAwakeReason => 'TotalDrop envoie ou reçoit des fichiers';

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

  @override
  String get editUndo => 'Annuler';

  @override
  String get editRedo => 'Rétablir';

  @override
  String get editCut => 'Couper';

  @override
  String get editCopy => 'Copier';

  @override
  String get editPaste => 'Coller';

  @override
  String get editDelete => 'Supprimer';

  @override
  String get editSelectAll => 'Tout sélectionner';

  @override
  String get transferNoticeChannel => 'Transferts';

  @override
  String get transferNoticeSending => 'Envoi avec TotalDrop';

  @override
  String get transferNoticeReceiving => 'Réception avec TotalDrop';

  @override
  String get transferNoticeBody =>
      'Continue pendant que vous utilisez d’autres apps.';

  @override
  String get shareBusy =>
      'Un transfert est encore en cours. Attendez qu’il se termine ou arrêtez-le, puis partagez à nouveau.';

  @override
  String get linkBusy =>
      'Terminez ou arrêtez ce qui est à l’écran, puis rouvrez le lien.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count éléments partagés n\'ont pas pu être lus et ont été ignorés.',
      many:
          '$count d\'éléments partagés n\'ont pas pu être lus et ont été ignorés.',
      one: '$count élément partagé n\'a pas pu être lu et a été ignoré.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'Transfert de fichiers';

  @override
  String get desktopComment =>
      'Envoyez des fichiers directement vers un autre appareil, chiffrés de bout en bout';

  @override
  String get desktopKeywords =>
      'envoyer;recevoir;partager;transférer;fichiers;';

  @override
  String get desktopSendWith => 'Envoyer avec TotalDrop';

  @override
  String get metainfoSummary =>
      'Envoyez des fichiers directement vers un autre appareil';

  @override
  String get metainfoAbout =>
      'TotalDrop envoie des fichiers et des dossiers d\'un appareil à un autre, chiffrés de bout en bout. Choisissez les fichiers, partagez un code court ou un code QR, et l\'autre appareil les reçoit. Pas de compte, pas d\'e-mail, pas de numéro de téléphone.';

  @override
  String get metainfoVerified =>
      'Chaque fichier est vérifié à l\'arrivée ; les fichiers endommagés sont écartés';

  @override
  String get metainfoWords =>
      'Quatre mots de vérification montrent que personne ne s\'est interposé';

  @override
  String get metainfoTheme =>
      'Suit le thème de votre bureau : Plasma, Hyprland, Omarchy, GNOME';

  @override
  String get metainfoOnePackage =>
      'Utilisez un seul paquet Linux à la fois. Le Flatpak et le paquet .pkg, .deb ou .rpm partagent un même nom : quand l\'un est ouvert, lancer l\'autre ramène la fenêtre ouverte au premier plan au lieu d\'en ouvrir une seconde.';

  @override
  String get otherPackageFlatpak =>
      'TotalDrop est déjà ouvert depuis le Flatpak : cette fenêtre a été ramenée au premier plan. Utilisez un seul paquet à la fois : quittez cette fenêtre pour ouvrir celui-ci.';

  @override
  String get otherPackageSystem =>
      'TotalDrop est déjà ouvert depuis le paquet .pkg, .deb ou .rpm : cette fenêtre a été ramenée au premier plan. Utilisez un seul paquet à la fois : quittez cette fenêtre pour ouvrir celui-ci.';

  @override
  String get sendText => 'Envoyer du texte';

  @override
  String get sendTextHint => 'Saisissez ou collez le texte à envoyer';

  @override
  String sendTextTooLong(String size) {
    return 'Un texte peut faire au plus $size. Envoyez un texte plus long sous forme de fichier.';
  }

  @override
  String get sendTextDone => 'Le texte est arrivé intact.';

  @override
  String get sendClipboard => 'Envoyer le presse-papiers';

  @override
  String get sendClipboardEmpty =>
      'Le presse-papiers ne contient pas de texte.';

  @override
  String get receiveOfferText => 'Texte entrant';

  @override
  String get receiveOfferTextNote =>
      'Le texte s’affiche ici, il n’est pas enregistré.';

  @override
  String get receiveTextDone => 'Texte reçu et vérifié';

  @override
  String get receiveTextDeclined => 'Vous avez refusé le texte.';

  @override
  String get receiveTextHidden =>
      'Le texte contient des caractères cachés, affichés sous la forme U+…. La copie donne le texte exactement tel qu’il est arrivé.';

  @override
  String get sendTextHidden =>
      'Le texte contient des caractères cachés, comme des marques de direction. Ils sont envoyés avec lui.';

  @override
  String get sendClipboardKept =>
      'Le texte déjà saisi est conservé ; le presse-papiers n’a pas été utilisé.';

  @override
  String get sendTextShortened =>
      'Seul le début du texte est affiché ; il est envoyé en entier.';

  @override
  String get receiveTextShortened =>
      'Seul le début du texte est affiché. Copier le texte le copie en entier.';

  @override
  String get copyText => 'Copier le texte';

  @override
  String get textCopied => 'Texte copié';

  @override
  String get errorTextDamaged =>
      'Le texte n’est pas arrivé intact, il n’est donc pas affiché.';

  @override
  String get setupBackgroundTitle => 'Continuer les transferts en arrière-plan';

  @override
  String get setupBackgroundBody =>
      'Pour qu\'un transfert continue quand vous changez d\'appli ou verrouillez l\'écran, autorisez TotalDrop à fonctionner sans restriction de batterie. TotalDrop ne travaille en arrière-plan que pendant un transfert que vous avez lancé.';

  @override
  String get setupBackgroundAllowed =>
      'L\'optimisation de la batterie d\'Android est désactivée pour TotalDrop.';

  @override
  String get setupBackgroundRestricted =>
      'L\'optimisation de la batterie d\'Android est activée pour TotalDrop : un transfert peut s\'arrêter quand vous changez d\'appli.';

  @override
  String get setupBackgroundBattery => 'Ouvrir les réglages de batterie';

  @override
  String get setupBackgroundBatteryHint =>
      'Dans les infos de l\'application TotalDrop, ouvrez « Batterie » et autorisez l\'utilisation sans restriction. Le nom de ces réglages varie d\'un téléphone à l\'autre.';

  @override
  String setupBackgroundMaker(String maker) {
    return 'Les téléphones $maker ont leur propre économiseur de batterie';
  }

  @override
  String get setupBackgroundMakerBody =>
      'Ouvrez aussi l\'économiseur de batterie de TotalDrop et choisissez « Aucune restriction ». Si le téléphone propose le Démarrage automatique, activez-le pour TotalDrop.';

  @override
  String get setupBackgroundMakerOpen =>
      'Ouvrir l\'économiseur de batterie de TotalDrop';

  @override
  String get setupBackgroundAutostart => 'Ouvrir le Démarrage automatique';

  @override
  String get setupBackgroundSkip => 'Pas maintenant';

  @override
  String get settingsBackground => 'Transferts en arrière-plan';

  @override
  String get settingsBackgroundOpen => 'Réglages de batterie';
}
