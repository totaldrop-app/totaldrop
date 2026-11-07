// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Spanish Castilian (`es`).
class AppLocalizationsEs extends AppLocalizations {
  AppLocalizationsEs([String locale = 'es']) : super(locale);

  @override
  String get appTagline =>
      'Cualquier archivo, a cualquier persona, en cualquier lugar.';

  @override
  String get homeSendTitle => 'Enviar';

  @override
  String get homeSendBody => 'Elige archivos y comparte un código corto';

  @override
  String get homeReceiveTitle => 'Recibir';

  @override
  String get homeReceiveBody => 'Introduce un código y guarda los archivos';

  @override
  String get homeFooter =>
      'Cifrado de extremo a extremo. Sin cuenta, sin correo electrónico, sin número de teléfono.';

  @override
  String get settings => 'Ajustes';

  @override
  String get sendTitle => 'Enviar archivos';

  @override
  String get sendAddFiles => 'Añadir archivos';

  @override
  String get sendEmpty => 'Aún no hay nada seleccionado';

  @override
  String get sendEmptyHint =>
      'Añade los archivos que quieres enviar. El destinatario recibe exactamente estos, nada más.';

  @override
  String get sendStrongCode => 'Código más seguro';

  @override
  String get sendStrongCodeHint =>
      'Tres palabras en lugar de dos: mucho más difícil de adivinar';

  @override
  String get sendCreateCode => 'Crear código';

  @override
  String get sendCounting => 'Contando…';

  @override
  String get sendRemove => 'Quitar';

  @override
  String get sendPreparing => 'Preparando tus archivos…';

  @override
  String get sendAddFailed =>
      'No se pudieron añadir los archivos. Vuelve a intentarlo. Si vuelve a pasar, comprueba que haya suficiente espacio libre en este dispositivo o elige los archivos desde otro lugar.';

  @override
  String get sendYourCode => 'Tu código';

  @override
  String get sendCodeHint =>
      'En el otro dispositivo, abre TotalDrop, toca Recibir e introduce este código.';

  @override
  String get sendWaiting => 'Esperando al destinatario…';

  @override
  String get sendCopy => 'Copiar';

  @override
  String get sendCopied => 'Código copiado';

  @override
  String get sendShare => 'Compartir';

  @override
  String sendShareText(String code) {
    return 'Recibe mis archivos con TotalDrop. Código: $code';
  }

  @override
  String get sendSending => 'Enviando';

  @override
  String get sendDone => 'Enviado y verificado';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Los $countString archivos llegaron intactos.',
      many: 'Los $countString de archivos llegaron intactos.',
      one: 'El archivo llegó intacto.',
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
      other: '$countString archivos no llegaron intactos y se descartaron.',
      many: '$countString de archivos no llegaron intactos y se descartaron.',
      one: 'Un archivo no llegó intacto y se descartó.',
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
      other: '$countString elementos',
      many: '$countString de elementos',
      one: '$countString elemento',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Recibir archivos';

  @override
  String get receiveCodeLabel => 'Código';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Pide el código a quien te envía los archivos.';

  @override
  String get receiveCodeInvalid =>
      'Eso no parece un código. Empieza con un número, seguido de dos o tres palabras.';

  @override
  String get receivePaste => 'Pegar';

  @override
  String get receiveConnect => 'Conectar';

  @override
  String get receiveConnecting => 'Conectando de forma segura…';

  @override
  String get receiveOffer => 'Archivos entrantes';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString archivos',
      many: '$filesString de archivos',
      one: '$filesString archivo',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'y $countString más';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Guardado en $folder';
  }

  @override
  String get receiveAccept => 'Aceptar';

  @override
  String get receiveDecline => 'Rechazar';

  @override
  String get receiveDeclined => 'Has rechazado los archivos.';

  @override
  String get receiveReceiving => 'Recibiendo';

  @override
  String get receiveDone => 'Recibido y verificado';

  @override
  String get verificationTitle => 'Palabras de verificación';

  @override
  String get verificationHint =>
      'La otra persona ve las mismas cuatro palabras. Si no coinciden, detente.';

  @override
  String get sameNetworkNote =>
      'Funciona a través de internet. Cuando los dispositivos no pueden conectarse directamente, la transferencia cifrada pasa por el servidor de retransmisión, que no puede leerla.';

  @override
  String get done => 'Listo';

  @override
  String get tryAgain => 'Reintentar';

  @override
  String get cancel => 'Cancelar';

  @override
  String get cancelTransferTitle => '¿Detener esta transferencia?';

  @override
  String get cancelTransferBody =>
      'Se avisará a la otra parte de que la transferencia se detuvo.';

  @override
  String get keepGoing => 'Continuar';

  @override
  String get stop => 'Detener';

  @override
  String get moreInfo => 'Detalles';

  @override
  String get errorWrongCode =>
      'El código no coincide. Se escribió mal o alguien intentó interceptar la transferencia. No se envió nada. Empieza de nuevo con un código nuevo.';

  @override
  String get errorPeerLeft =>
      'La otra parte se fue antes de que terminara la transferencia.';

  @override
  String get errorDeclined => 'El destinatario rechazó los archivos.';

  @override
  String get errorCancelled => 'La otra parte detuvo la transferencia.';

  @override
  String get errorStopped => 'Detenida en este dispositivo.';

  @override
  String get errorInvalidCode => 'Ese código no es válido.';

  @override
  String get errorServer =>
      'No se pudo acceder al servidor de TotalDrop. Comprueba tu conexión a internet.';

  @override
  String get errorCodeNotInUse =>
      'Ese código no está en uso: se escribió mal, caducó o ya se usó. Pide uno nuevo a quien te envía los archivos.';

  @override
  String get errorCodeExpired =>
      'El código caducó antes de que alguien lo usara. Vuelve a enviar para obtener un código nuevo.';

  @override
  String get errorBadServer =>
      'La dirección del servidor no es válida. Debe empezar por wss://.';

  @override
  String get errorConnect =>
      'No se pudo conectar con el otro dispositivo. Comprueba que ambos dispositivos estén en línea y vuelve a intentarlo.';

  @override
  String get errorInvalidOffer =>
      'La oferta incumplía una regla de seguridad, así que se rechazó.';

  @override
  String get errorStorage => 'No se pudo leer o guardar un archivo.';

  @override
  String get errorOther => 'Algo salió mal.';

  @override
  String get causeTimedOut =>
      'La conexión dejó de responder. Puede que uno de los dispositivos haya perdido la red o se haya suspendido.';

  @override
  String get causeConnectionLost =>
      'Se cortó la conexión con el otro dispositivo.';

  @override
  String get causeNoRoute =>
      'No se pudo llegar al otro dispositivo, ni directamente ni a través del servidor de retransmisión.';

  @override
  String get causeRelay =>
      'No se pudo llegar al servidor de retransmisión o no está configurado, o los dos dispositivos no usan el mismo.';

  @override
  String get causeProtocol =>
      'El otro dispositivo envió algo que esta versión no entiende. Actualizar TotalDrop en ambos dispositivos puede ayudar.';

  @override
  String get causeDiskFull =>
      'No hay suficiente espacio libre en este dispositivo.';

  @override
  String get causeNoPermission =>
      'TotalDrop no tiene permiso para leer o escribir un archivo o una carpeta que necesita.';

  @override
  String get causeNotFound =>
      'No se encontró un archivo o una carpeta. Puede que se haya movido o eliminado.';

  @override
  String get causeReadOnly =>
      'La carpeta está en un disco en el que no se puede escribir.';

  @override
  String get causeFileChanged =>
      'Un archivo cambió mientras se enviaba. Vuelve a enviarlo cuando esté guardado.';

  @override
  String get technicalDetail =>
      'Mensaje técnico en inglés, para incluirlo al informar de un problema:';

  @override
  String get settingsServer => 'Servidor de encuentro';

  @override
  String get settingsServerHelp =>
      'Ayuda a dos dispositivos con el mismo código a encontrarse. Nunca ve tus archivos ni tus claves.';

  @override
  String get settingsServerReset => 'Usar el predeterminado';

  @override
  String get settingsServerInvalid => 'Debe empezar por wss://';

  @override
  String get settingsSave => 'Guardar';

  @override
  String get settingsAbout => 'Acerca de';

  @override
  String get settingsAboutBody =>
      'TotalDrop es gratuito y de código abierto, financiado con donaciones. Licencia GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Privacidad';

  @override
  String get settingsPrivacyBody =>
      'El servidor de encuentro, el servidor de retransmisión y el servidor STUN de Cloudflare ven tu dirección IP, y también el otro dispositivo cuando la conexión es directa (salvo que el modo privacidad esté activado). Ninguno ve los nombres de los archivos, su contenido ni las claves, y nuestros servidores no guardan registros. Los archivos van directamente entre los dispositivos cuando es posible y, si no, a través del servidor de retransmisión; en ambos casos están cifrados de extremo a extremo.';

  @override
  String get privacyMode => 'Ocultar mi dirección al otro dispositivo';

  @override
  String get privacyModeHint =>
      'Modo privacidad: todo pasa por el servidor de retransmisión, así que el otro dispositivo nunca conoce tu dirección IP. Es más lento, y el otro dispositivo debe usar el mismo servidor de retransmisión (el predeterminado lo hace).';

  @override
  String get privacyModeConfirmTitle => '¿Activar el modo privacidad?';

  @override
  String get privacyModeConfirmBody =>
      'Las transferencias serán más lentas, porque nada pasa directamente entre los dispositivos. El servidor de encuentro y el servidor de retransmisión siguen viendo tu dirección IP; nunca ven tus archivos ni tus claves.';

  @override
  String get privacyModeTurnOn => 'Activar';

  @override
  String settingsVersion(String version) {
    return 'Versión $version';
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
      'Esta traducción se está revisando. Ayuda a mejorarla en GitHub.';

  @override
  String get settingsWordsEnglish =>
      'Los códigos y las palabras de verificación están siempre en inglés, para que dos dispositivos en idiomas distintos muestren las mismas palabras.';

  @override
  String get sendDropHere => 'Suelta los archivos aquí';

  @override
  String get sendDropHint =>
      'También puedes arrastrar archivos a esta ventana.';

  @override
  String get sendQrHint =>
      'O escanea este código con la cámara del otro dispositivo.';

  @override
  String get sendWaitingAccept =>
      'Comprueba las palabras y espera a que el destinatario acepte.';

  @override
  String get receiveFromImage => 'Leer QR de una imagen';

  @override
  String get receiveQrNotFound =>
      'No hay ningún código de TotalDrop en esa imagen.';

  @override
  String get pickTitle => 'Elige los archivos que quieres enviar';

  @override
  String get pickImageTitle => 'Elige una imagen con un código QR';

  @override
  String get openFolder => 'Abrir carpeta';

  @override
  String get back => 'Atrás';

  @override
  String get codeSuggestions => 'Sugerencias';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString %';
  }

  @override
  String get editUndo => 'Deshacer';

  @override
  String get editRedo => 'Rehacer';

  @override
  String get editCut => 'Cortar';

  @override
  String get editCopy => 'Copiar';

  @override
  String get editPaste => 'Pegar';

  @override
  String get editDelete => 'Eliminar';

  @override
  String get editSelectAll => 'Seleccionar todo';

  @override
  String get transferNoticeChannel => 'Transferencias';

  @override
  String get transferNoticeSending => 'Enviando con TotalDrop';

  @override
  String get transferNoticeReceiving => 'Recibiendo con TotalDrop';

  @override
  String get transferNoticeBody => 'Sigue mientras usas otras apps.';

  @override
  String get shareTextSoon =>
      'TotalDrop aún no envía texto, solo archivos. Comparte un archivo.';

  @override
  String get shareBusy =>
      'Todavía hay una transferencia en curso. Espera a que termine o detenla y vuelve a compartir.';

  @override
  String shareSkipped(int count) {
    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'No se pudieron leer $count elementos compartidos y se omitieron.',
      many:
          'No se pudieron leer $count de elementos compartidos y se omitieron.',
      one: 'No se pudo leer un elemento compartido y se omitió.',
    );
    return '$_temp0';
  }

  @override
  String get desktopGenericName => 'Transferencia de archivos';

  @override
  String get desktopComment =>
      'Envía archivos directamente a otro dispositivo, con cifrado de extremo a extremo';

  @override
  String get desktopKeywords => 'enviar;recibir;compartir;transferir;archivos;';

  @override
  String get desktopSendWith => 'Enviar con TotalDrop';

  @override
  String get metainfoSummary =>
      'Envía archivos directamente a otro dispositivo';

  @override
  String get metainfoAbout =>
      'TotalDrop envía archivos y carpetas de un dispositivo a otro, con cifrado de extremo a extremo. Elige los archivos, comparte un código corto o un código QR y el otro dispositivo los recibe. Sin cuenta, sin correo electrónico, sin número de teléfono.';

  @override
  String get metainfoVerified =>
      'Cada archivo se verifica al llegar; los archivos dañados se descartan';

  @override
  String get metainfoWords =>
      'Cuatro palabras de verificación muestran que nadie está en medio';

  @override
  String get metainfoTheme =>
      'Sigue el tema de tu escritorio: Plasma, Hyprland, Omarchy, GNOME';

  @override
  String get sendText => 'Enviar texto';

  @override
  String get sendTextHint => 'Escribe o pega el texto que quieres enviar';

  @override
  String sendTextTooLong(String size) {
    return 'Un texto puede ocupar como máximo $size. Envía uno más largo como archivo.';
  }

  @override
  String get sendTextDone => 'El texto ha llegado íntegro.';

  @override
  String get sendClipboard => 'Enviar portapapeles';

  @override
  String get sendClipboardEmpty => 'El portapapeles no contiene texto.';

  @override
  String get receiveOfferText => 'Texto entrante';

  @override
  String get receiveOfferTextNote => 'El texto se muestra aquí, no se guarda.';

  @override
  String get receiveTextDone => 'Texto recibido y verificado';

  @override
  String get receiveTextDeclined => 'Has rechazado el texto.';

  @override
  String get receiveTextHidden =>
      'El texto contiene caracteres ocultos, que se muestran como U+…. Al copiarlo obtienes el texto exactamente como llegó.';

  @override
  String get sendClipboardKept =>
      'Se conserva el texto ya escrito; no se usó el portapapeles.';

  @override
  String get sendTextShortened =>
      'Solo se muestra el comienzo del texto; se envía completo.';

  @override
  String get receiveTextShortened =>
      'Solo se muestra el comienzo del texto. Copiar texto lo copia completo.';

  @override
  String get copyText => 'Copiar texto';

  @override
  String get textCopied => 'Texto copiado';

  @override
  String get errorTextDamaged =>
      'El texto no ha llegado íntegro, así que no se muestra.';
}
