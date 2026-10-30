// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Italian (`it`).
class AppLocalizationsIt extends AppLocalizations {
  AppLocalizationsIt([String locale = 'it']) : super(locale);

  @override
  String get appTagline => 'Qualsiasi file, a chiunque, ovunque.';

  @override
  String get homeSendTitle => 'Invia';

  @override
  String get homeSendBody => 'Scegli i file e condividi un codice breve';

  @override
  String get homeReceiveTitle => 'Ricevi';

  @override
  String get homeReceiveBody => 'Inserisci un codice e salva i file';

  @override
  String get homeFooter =>
      'Crittografia end-to-end. Nessun account, nessuna email, nessun numero di telefono.';

  @override
  String get settings => 'Impostazioni';

  @override
  String get sendTitle => 'Invia file';

  @override
  String get sendAddFiles => 'Aggiungi file';

  @override
  String get sendEmpty => 'Ancora nessuna selezione';

  @override
  String get sendEmptyHint =>
      'Aggiungi i file che vuoi inviare. Il destinatario riceve esattamente questi, nient\'altro.';

  @override
  String get sendStrongCode => 'Codice più robusto';

  @override
  String get sendStrongCodeHint =>
      'Tre parole invece di due: molto più difficile da indovinare';

  @override
  String get sendCreateCode => 'Crea codice';

  @override
  String get sendCounting => 'Conteggio in corso…';

  @override
  String get sendRemove => 'Rimuovi';

  @override
  String get sendPreparing => 'Preparazione dei file…';

  @override
  String get sendYourCode => 'Il tuo codice';

  @override
  String get sendCodeHint =>
      'Sull\'altro dispositivo apri TotalDrop, tocca Ricevi e inserisci questo codice.';

  @override
  String get sendWaiting => 'In attesa del destinatario…';

  @override
  String get sendCopy => 'Copia';

  @override
  String get sendCopied => 'Codice copiato';

  @override
  String get sendShare => 'Condividi';

  @override
  String sendShareText(String code) {
    return 'Ricevi i miei file con TotalDrop. Codice: $code';
  }

  @override
  String get sendSending => 'Invio in corso';

  @override
  String get sendDone => 'Inviato e verificato';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Tutti i $countString file sono arrivati integri.',
      many: 'Tutti i $countString di file sono arrivati integri.',
      one: 'Il file è arrivato integro.',
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
          '$countString file non sono arrivati integri e sono stati scartati.',
      many:
          '$countString di file non sono arrivati integri e sono stati scartati.',
      one: 'Un file non è arrivato integro ed è stato scartato.',
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
      other: '$countString elementi',
      many: '$countString di elementi',
      one: '$countString elemento',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Ricevi file';

  @override
  String get receiveCodeLabel => 'Codice';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Chiedi il codice a chi invia.';

  @override
  String get receiveCodeInvalid =>
      'Non sembra un codice. Inizia con un numero, seguito da due o tre parole.';

  @override
  String get receivePaste => 'Incolla';

  @override
  String get receiveConnect => 'Connetti';

  @override
  String get receiveConnecting => 'Connessione sicura in corso…';

  @override
  String get receiveOffer => 'File in arrivo';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString file',
      many: '$filesString di file',
      one: '$filesString file',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'e altri $countString';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Salvato in $folder';
  }

  @override
  String get receiveAccept => 'Accetta';

  @override
  String get receiveDecline => 'Rifiuta';

  @override
  String get receiveDeclined => 'Hai rifiutato i file.';

  @override
  String get receiveReceiving => 'Ricezione in corso';

  @override
  String get receiveDone => 'Ricevuto e verificato';

  @override
  String get verificationTitle => 'Parole di verifica';

  @override
  String get verificationHint =>
      'L\'altra persona vede le stesse quattro parole. Se sono diverse, interrompi.';

  @override
  String get sameNetworkNote =>
      'Funziona via internet. Quando i dispositivi non riescono a raggiungersi direttamente, il trasferimento crittografato passa dal server di inoltro, che non può leggerlo.';

  @override
  String get done => 'Fatto';

  @override
  String get tryAgain => 'Riprova';

  @override
  String get cancel => 'Annulla';

  @override
  String get cancelTransferTitle => 'Interrompere il trasferimento?';

  @override
  String get cancelTransferBody =>
      'L\'altra parte verrà avvisata che il trasferimento è stato interrotto.';

  @override
  String get keepGoing => 'Continua';

  @override
  String get stop => 'Interrompi';

  @override
  String get moreInfo => 'Dettagli';

  @override
  String get errorWrongCode =>
      'Il codice non corrispondeva. È stato digitato male, oppure qualcuno ha cercato di intercettare il trasferimento. Non è stato inviato nulla. Ricomincia con un nuovo codice.';

  @override
  String get errorPeerLeft =>
      'L\'altra parte se n\'è andata prima della fine del trasferimento.';

  @override
  String get errorDeclined => 'Il destinatario ha rifiutato i file.';

  @override
  String get errorCancelled => 'L\'altra parte ha interrotto il trasferimento.';

  @override
  String get errorStopped => 'Interrotto su questo dispositivo.';

  @override
  String get errorInvalidCode => 'Questo codice non è valido.';

  @override
  String get errorServer =>
      'Impossibile raggiungere il server di TotalDrop. Controlla la connessione a internet.';

  @override
  String get errorCodeNotInUse =>
      'Questo codice non è in uso: è stato digitato male, è scaduto o è già stato usato. Chiedine uno nuovo a chi invia.';

  @override
  String get errorConnect =>
      'Impossibile connettersi all\'altro dispositivo. Controlla che entrambi i dispositivi siano online e riprova.';

  @override
  String get errorInvalidOffer =>
      'L\'offerta violava una regola di sicurezza ed è stata rifiutata.';

  @override
  String get errorStorage => 'Non è stato possibile leggere o salvare un file.';

  @override
  String get errorOther => 'Si è verificato un problema.';

  @override
  String get settingsServer => 'Server di incontro';

  @override
  String get settingsServerHelp =>
      'Aiuta due dispositivi con lo stesso codice a trovarsi. Non vede mai i tuoi file né le tue chiavi.';

  @override
  String get settingsServerReset => 'Usa quello predefinito';

  @override
  String get settingsServerInvalid => 'Deve iniziare con wss://';

  @override
  String get settingsSave => 'Salva';

  @override
  String get settingsAbout => 'Informazioni';

  @override
  String get settingsAboutBody =>
      'TotalDrop è gratuito e open source, finanziato dalle donazioni. Licenza GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Privacy';

  @override
  String get settingsPrivacyBody =>
      'Il server di incontro, il server di inoltro e il server STUN di Cloudflare vedono il tuo indirizzo IP, e così anche l\'altro dispositivo quando vi connettete direttamente (a meno che la modalità privacy sia attiva). Nessuno di loro vede i nomi dei file, i contenuti o le chiavi, e i nostri server non conservano log. I file passano direttamente tra i dispositivi quando possibile, altrimenti tramite il server di inoltro; in entrambi i casi sono crittografati end-to-end.';

  @override
  String get privacyMode => 'Nascondi il mio indirizzo all\'altro dispositivo';

  @override
  String get privacyModeHint =>
      'Modalità privacy: tutto passa dal server di inoltro, così l\'altro dispositivo non conosce mai il tuo indirizzo IP. È più lenta, e l\'altro dispositivo deve usare lo stesso server di inoltro (quello predefinito lo fa).';

  @override
  String get privacyModeConfirmTitle => 'Attivare la modalità privacy?';

  @override
  String get privacyModeConfirmBody =>
      'I trasferimenti saranno più lenti, perché niente passa direttamente tra i dispositivi. Il server di incontro e il server di inoltro vedono comunque il tuo indirizzo IP; non vedono mai i tuoi file né le tue chiavi.';

  @override
  String get privacyModeTurnOn => 'Attiva';

  @override
  String settingsVersion(String version) {
    return 'Versione $version';
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
      'Questa traduzione è in fase di revisione. Aiuta a migliorarla su GitHub.';

  @override
  String get sendDropHere => 'Rilascia i file qui';

  @override
  String get sendDropHint => 'Puoi anche trascinare i file su questa finestra.';

  @override
  String get sendQrHint =>
      'Oppure scansiona questo codice con la fotocamera dell\'altro dispositivo.';

  @override
  String get sendWaitingAccept =>
      'Controlla le parole, poi attendi che il destinatario accetti.';

  @override
  String get receiveFromImage => 'Leggi QR da immagine';

  @override
  String get receiveQrNotFound =>
      'Non c\'è nessun codice TotalDrop in questa immagine.';

  @override
  String get pickTitle => 'Scegli i file da inviare';

  @override
  String get pickImageTitle => 'Scegli un\'immagine con un codice QR';

  @override
  String get openFolder => 'Apri cartella';

  @override
  String get back => 'Indietro';

  @override
  String get codeSuggestions => 'Suggerimenti';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
