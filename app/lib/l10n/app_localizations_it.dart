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
  String get sendAddFailed =>
      'Impossibile aggiungere i file. Riprova. Se succede di nuovo, controlla che su questo dispositivo ci sia abbastanza spazio libero, oppure scegli i file da un\'altra posizione.';

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
  String get errorCodeExpired =>
      'Il codice è scaduto prima che qualcuno lo usasse. Invia di nuovo per ottenere un nuovo codice.';

  @override
  String get errorBadServer =>
      'L\'indirizzo del server non è valido. Deve iniziare con wss://.';

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
  String get causeTimedOut =>
      'La connessione ha smesso di rispondere. Uno dei dispositivi potrebbe aver perso la rete o essere andato in sospensione.';

  @override
  String get causeConnectionLost =>
      'La connessione con l\'altro dispositivo si è interrotta.';

  @override
  String get causeNoRoute =>
      'Non è stato possibile raggiungere l\'altro dispositivo, né direttamente né tramite il server di inoltro.';

  @override
  String get causeRelay =>
      'Il server di inoltro non è raggiungibile o non è configurato, oppure i due dispositivi non usano lo stesso.';

  @override
  String get causeProtocol =>
      'L\'altro dispositivo ha inviato qualcosa che questa versione non capisce. Aggiornare TotalDrop su entrambi i dispositivi può aiutare.';

  @override
  String get causeDiskFull =>
      'Non c\'è abbastanza spazio libero su questo dispositivo.';

  @override
  String get causeNoPermission =>
      'TotalDrop non ha il permesso di leggere o scrivere un file o una cartella di cui ha bisogno.';

  @override
  String get causeNotFound =>
      'Un file o una cartella non è stato trovato. Forse è stato spostato o eliminato.';

  @override
  String get causeReadOnly =>
      'La cartella si trova su un disco in cui non si può scrivere.';

  @override
  String get causeFileChanged =>
      'Un file è cambiato durante l\'invio. Invialo di nuovo dopo averlo salvato.';

  @override
  String get technicalDetail =>
      'Messaggio tecnico in inglese, da allegare quando segnali un problema:';

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
      'Il server di incontro, il server di inoltro e il server STUN di Cloudflare vedono il tuo indirizzo IP, e così anche l\'altro dispositivo quando vi connettete direttamente (a meno che la modalità privacy sia attiva). Nessuno di loro vede i nomi dei file, i contenuti o le chiavi. I nostri server non conservano log propri, ma Cloudflare, che li gestisce, conserva un campione delle richieste con gli indirizzi IP per almeno 31 giorni. I file passano direttamente tra i dispositivi quando possibile, altrimenti tramite il server di inoltro; in entrambi i casi sono crittografati end-to-end.';

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
  String get settingsWordsEnglish =>
      'I codici e le parole di verifica restano in inglese in ogni lingua, così due dispositivi impostati su lingue diverse mostrano le stesse parole.';

  @override
  String get sendDropHere => 'Rilascia i file qui';

  @override
  String get sendDropHint => 'Puoi anche trascinare i file su questa finestra.';

  @override
  String get keepAwakeReason => 'TotalDrop sta inviando o ricevendo file';

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

  @override
  String get editUndo => 'Annulla';

  @override
  String get editRedo => 'Ripeti';

  @override
  String get editCut => 'Taglia';

  @override
  String get editCopy => 'Copia';

  @override
  String get editPaste => 'Incolla';

  @override
  String get editDelete => 'Elimina';

  @override
  String get editSelectAll => 'Seleziona tutto';

  @override
  String get transferNoticeChannel => 'Trasferimenti';

  @override
  String get transferNoticeSending => 'Invio con TotalDrop';

  @override
  String get transferNoticeReceiving => 'Ricezione con TotalDrop';

  @override
  String get transferNoticeBody => 'Continua mentre usi altre app.';

  @override
  String get shareBusy =>
      'Un trasferimento è ancora in corso. Attendi che finisca o interrompilo, poi condividi di nuovo.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other:
          '$count elementi condivisi non sono stati letti e sono stati esclusi.',
      many:
          '$count di elementi condivisi non sono stati letti e sono stati esclusi.',
      one: 'Un elemento condiviso non è stato letto ed è stato escluso.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'Trasferimento file';

  @override
  String get desktopComment =>
      'Invia file direttamente a un altro dispositivo, con crittografia end-to-end';

  @override
  String get desktopKeywords => 'invia;ricevi;condividi;trasferisci;file;';

  @override
  String get desktopSendWith => 'Invia con TotalDrop';

  @override
  String get metainfoSummary =>
      'Invia file direttamente a un altro dispositivo';

  @override
  String get metainfoAbout =>
      'TotalDrop invia file e cartelle da un dispositivo all\'altro, con crittografia end-to-end. Scegli i file, condividi un codice breve o un codice QR e l\'altro dispositivo li riceve. Nessun account, nessuna email, nessun numero di telefono.';

  @override
  String get metainfoVerified =>
      'Ogni file viene verificato all\'arrivo; i file danneggiati vengono scartati';

  @override
  String get metainfoWords =>
      'Quattro parole di verifica mostrano che nessuno si è messo in mezzo';

  @override
  String get metainfoTheme =>
      'Segue il tema del tuo desktop: Plasma, Hyprland, Omarchy, GNOME';

  @override
  String get metainfoOnePackage =>
      'Usa un solo pacchetto Linux alla volta. Il Flatpak e il pacchetto .pkg, .deb o .rpm condividono un nome, quindi mentre uno è aperto, avviare l\'altro porta in primo piano la finestra aperta invece di aprirne una seconda.';

  @override
  String get otherPackageFlatpak =>
      'TotalDrop è già aperto dal Flatpak, quindi quella finestra è stata portata in primo piano. Usa un solo pacchetto alla volta: chiudi quella finestra per aprire questo.';

  @override
  String get otherPackageSystem =>
      'TotalDrop è già aperto dal pacchetto .pkg, .deb o .rpm, quindi quella finestra è stata portata in primo piano. Usa un solo pacchetto alla volta: chiudi quella finestra per aprire questo.';

  @override
  String get sendText => 'Invia testo';

  @override
  String get sendTextHint => 'Scrivi o incolla il testo da inviare';

  @override
  String sendTextTooLong(String size) {
    return 'Un testo può occupare al massimo $size. Invia un testo più lungo come file.';
  }

  @override
  String get sendTextDone => 'Il testo è arrivato integro.';

  @override
  String get sendClipboard => 'Invia appunti';

  @override
  String get sendClipboardEmpty => 'Gli appunti non contengono testo.';

  @override
  String get receiveOfferText => 'Testo in arrivo';

  @override
  String get receiveOfferTextNote =>
      'Il testo viene mostrato qui, non salvato.';

  @override
  String get receiveTextDone => 'Testo ricevuto e verificato';

  @override
  String get receiveTextDeclined => 'Hai rifiutato il testo.';

  @override
  String get receiveTextHidden =>
      'Il testo contiene caratteri nascosti, mostrati come U+…. Copiandolo ottieni il testo esattamente come è arrivato.';

  @override
  String get sendTextHidden =>
      'Il testo contiene caratteri nascosti, come i segni di direzione. Vengono inviati insieme al testo.';

  @override
  String get sendClipboardKept =>
      'Il testo già scritto è stato mantenuto; gli appunti non sono stati usati.';

  @override
  String get sendTextShortened =>
      'Viene mostrato solo l’inizio del testo; viene inviato per intero.';

  @override
  String get receiveTextShortened =>
      'Viene mostrato solo l’inizio del testo. Copia testo lo copia per intero.';

  @override
  String get copyText => 'Copia testo';

  @override
  String get textCopied => 'Testo copiato';

  @override
  String get errorTextDamaged =>
      'Il testo non è arrivato integro, quindi non viene mostrato.';

  @override
  String get setupBackgroundTitle => 'Trasferimenti attivi in background';

  @override
  String get setupBackgroundBody =>
      'Perché un trasferimento continui quando cambi app o blocchi lo schermo, consenti a TotalDrop di funzionare senza restrizioni della batteria. TotalDrop lavora in background solo mentre è in corso un trasferimento che hai avviato tu.';

  @override
  String get setupBackgroundAllowed =>
      'L\'ottimizzazione della batteria di Android è disattivata per TotalDrop.';

  @override
  String get setupBackgroundRestricted =>
      'L\'ottimizzazione della batteria di Android è attiva per TotalDrop: un trasferimento può fermarsi quando cambi app.';

  @override
  String get setupBackgroundBattery => 'Apri impostazioni batteria';

  @override
  String get setupBackgroundBatteryHint =>
      'Nelle informazioni dell\'app TotalDrop, apri Batteria e consenti l\'uso senza limitazioni. I nomi di queste impostazioni cambiano da telefono a telefono.';

  @override
  String setupBackgroundMaker(String maker) {
    return 'I telefoni $maker hanno un proprio risparmio batteria';
  }

  @override
  String get setupBackgroundMakerBody =>
      'Apri anche il risparmio batteria di TotalDrop e scegli \"Nessuna restrizione\". Se il telefono ha l\'Avvio automatico, attivalo per TotalDrop.';

  @override
  String get setupBackgroundMakerOpen =>
      'Apri il risparmio batteria di TotalDrop';

  @override
  String get setupBackgroundAutostart => 'Apri Avvio automatico';

  @override
  String get setupBackgroundSkip => 'Non ora';

  @override
  String get settingsBackground => 'Trasferimenti in background';

  @override
  String get settingsBackgroundOpen => 'Impostazioni batteria';
}
