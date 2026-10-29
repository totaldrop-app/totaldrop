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
      'Esta traducción se está revisando. Ayuda a mejorarla en Weblate.';

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
}
