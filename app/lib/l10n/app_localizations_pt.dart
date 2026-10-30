// ignore: unused_import
import 'package:intl/intl.dart' as intl;

import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Portuguese (`pt`).
class AppLocalizationsPt extends AppLocalizations {
  AppLocalizationsPt([String locale = 'pt']) : super(locale);

  @override
  String get appTagline =>
      'Qualquer arquivo, para qualquer pessoa, em qualquer lugar.';

  @override
  String get homeSendTitle => 'Enviar';

  @override
  String get homeSendBody => 'Escolha arquivos e compartilhe um código curto';

  @override
  String get homeReceiveTitle => 'Receber';

  @override
  String get homeReceiveBody => 'Digite um código e salve os arquivos';

  @override
  String get homeFooter =>
      'Criptografia de ponta a ponta. Sem conta, sem e-mail, sem número de telefone.';

  @override
  String get settings => 'Configurações';

  @override
  String get sendTitle => 'Enviar arquivos';

  @override
  String get sendAddFiles => 'Adicionar arquivos';

  @override
  String get sendEmpty => 'Nada selecionado ainda';

  @override
  String get sendEmptyHint =>
      'Adicione os arquivos que você quer enviar. O destinatário recebe exatamente esses, mais nada.';

  @override
  String get sendStrongCode => 'Código mais forte';

  @override
  String get sendStrongCodeHint =>
      'Três palavras em vez de duas: muito mais difícil de adivinhar';

  @override
  String get sendCreateCode => 'Criar código';

  @override
  String get sendCounting => 'Contando…';

  @override
  String get sendRemove => 'Remover';

  @override
  String get sendPreparing => 'Preparando seus arquivos…';

  @override
  String get sendYourCode => 'Seu código';

  @override
  String get sendCodeHint =>
      'No outro dispositivo, abra o TotalDrop, toque em Receber e digite este código.';

  @override
  String get sendWaiting => 'Aguardando o destinatário…';

  @override
  String get sendCopy => 'Copiar';

  @override
  String get sendCopied => 'Código copiado';

  @override
  String get sendShare => 'Compartilhar';

  @override
  String sendShareText(String code) {
    return 'Receba meus arquivos com o TotalDrop. Código: $code';
  }

  @override
  String get sendSending => 'Enviando';

  @override
  String get sendDone => 'Enviado e verificado';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Todos os $countString arquivos chegaram intactos.',
      many: 'Todos os $countString de arquivos chegaram intactos.',
      one: '$countString arquivo chegou intacto.',
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
      other: '$countString arquivos não chegaram intactos e foram descartados.',
      many:
          '$countString de arquivos não chegaram intactos e foram descartados.',
      one: '$countString arquivo não chegou intacto e foi descartado.',
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
      other: '$countString itens',
      many: '$countString de itens',
      one: '$countString item',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Receber arquivos';

  @override
  String get receiveCodeLabel => 'Código';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Peça o código a quem está enviando.';

  @override
  String get receiveCodeInvalid =>
      'Isso não parece um código. Ele começa com um número, seguido de duas ou três palavras.';

  @override
  String get receivePaste => 'Colar';

  @override
  String get receiveConnect => 'Conectar';

  @override
  String get receiveConnecting => 'Conectando com segurança…';

  @override
  String get receiveOffer => 'Arquivos recebidos';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString arquivos',
      many: '$filesString de arquivos',
      one: '$filesString arquivo',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'e mais $countString';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Salvo em $folder';
  }

  @override
  String get receiveAccept => 'Aceitar';

  @override
  String get receiveDecline => 'Recusar';

  @override
  String get receiveDeclined => 'Você recusou os arquivos.';

  @override
  String get receiveReceiving => 'Recebendo';

  @override
  String get receiveDone => 'Recebido e verificado';

  @override
  String get verificationTitle => 'Palavras de verificação';

  @override
  String get verificationHint =>
      'A outra pessoa vê as mesmas quatro palavras. Se forem diferentes, pare.';

  @override
  String get sameNetworkNote =>
      'Funciona pela internet. Quando os dispositivos não conseguem se alcançar diretamente, a transferência criptografada passa pelo servidor de retransmissão, que não consegue lê-la.';

  @override
  String get done => 'Concluído';

  @override
  String get tryAgain => 'Tentar de novo';

  @override
  String get cancel => 'Cancelar';

  @override
  String get cancelTransferTitle => 'Parar esta transferência?';

  @override
  String get cancelTransferBody =>
      'O outro lado será avisado de que a transferência parou.';

  @override
  String get keepGoing => 'Continuar';

  @override
  String get stop => 'Parar';

  @override
  String get moreInfo => 'Detalhes';

  @override
  String get errorWrongCode =>
      'O código não correspondeu. Ele foi digitado errado ou alguém tentou interceptar a transferência. Nada foi enviado. Comece de novo com um código novo.';

  @override
  String get errorPeerLeft =>
      'O outro lado saiu antes de a transferência terminar.';

  @override
  String get errorDeclined => 'O destinatário recusou os arquivos.';

  @override
  String get errorCancelled => 'O outro lado parou a transferência.';

  @override
  String get errorStopped => 'Parada neste dispositivo.';

  @override
  String get errorInvalidCode => 'Esse código não é válido.';

  @override
  String get errorServer =>
      'Não foi possível acessar o servidor do TotalDrop. Verifique sua conexão com a internet.';

  @override
  String get errorConnect =>
      'Não foi possível conectar ao outro dispositivo. Verifique se os dois dispositivos estão on-line e tente de novo.';

  @override
  String get errorInvalidOffer =>
      'A oferta violou uma regra de segurança e por isso foi recusada.';

  @override
  String get errorStorage => 'Não foi possível ler ou salvar um arquivo.';

  @override
  String get errorOther => 'Algo deu errado.';

  @override
  String get settingsServer => 'Servidor de encontro';

  @override
  String get settingsServerHelp =>
      'Ajuda dois dispositivos com o mesmo código a se encontrarem. Ele nunca vê seus arquivos nem suas chaves.';

  @override
  String get settingsServerReset => 'Usar o padrão';

  @override
  String get settingsServerInvalid => 'Precisa começar com wss://';

  @override
  String get settingsSave => 'Salvar';

  @override
  String get settingsAbout => 'Sobre';

  @override
  String get settingsAboutBody =>
      'O TotalDrop é gratuito e de código aberto, mantido por doações. Licença GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Privacidade';

  @override
  String get settingsPrivacyBody =>
      'O servidor de encontro, o servidor de retransmissão e o servidor STUN da Cloudflare veem seu endereço IP, e o outro dispositivo também vê quando vocês se conectam diretamente (a menos que o modo de privacidade esteja ativado). Nenhum deles vê nomes de arquivos, conteúdos ou chaves, e nossos servidores não guardam registros. Os arquivos vão diretamente entre os dispositivos quando possível; caso contrário, passam pelo servidor de retransmissão. Nos dois casos, têm criptografia de ponta a ponta.';

  @override
  String get privacyMode => 'Ocultar meu endereço do outro dispositivo';

  @override
  String get privacyModeHint =>
      'Modo de privacidade: tudo passa pelo servidor de retransmissão, então o outro dispositivo nunca descobre seu endereço IP. É mais lento, e o outro dispositivo precisa usar o mesmo servidor de retransmissão (o padrão usa).';

  @override
  String get privacyModeConfirmTitle => 'Ativar o modo de privacidade?';

  @override
  String get privacyModeConfirmBody =>
      'As transferências ficarão mais lentas, porque nada passa diretamente entre os dispositivos. O servidor de encontro e o servidor de retransmissão continuam vendo seu endereço IP; eles nunca veem seus arquivos nem suas chaves.';

  @override
  String get privacyModeTurnOn => 'Ativar';

  @override
  String settingsVersion(String version) {
    return 'Versão $version';
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
      'Esta tradução está em revisão. Ajude a melhorá-la no GitHub.';

  @override
  String get sendDropHere => 'Solte os arquivos aqui';

  @override
  String get sendDropHint =>
      'Você também pode arrastar arquivos para esta janela.';

  @override
  String get sendQrHint =>
      'Ou escaneie este código com a câmera do outro dispositivo.';

  @override
  String get sendWaitingAccept =>
      'Confira as palavras e aguarde o destinatário aceitar.';

  @override
  String get receiveFromImage => 'Ler QR de uma imagem';

  @override
  String get receiveQrNotFound =>
      'Não há nenhum código do TotalDrop nessa imagem.';

  @override
  String get pickTitle => 'Escolha os arquivos para enviar';

  @override
  String get pickImageTitle => 'Escolha uma imagem com um QR code';

  @override
  String get openFolder => 'Abrir pasta';

  @override
  String get back => 'Voltar';

  @override
  String get codeSuggestions => 'Sugestões';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}

/// The translations for Portuguese, as used in Brazil (`pt_BR`).
class AppLocalizationsPtBr extends AppLocalizationsPt {
  AppLocalizationsPtBr() : super('pt_BR');

  @override
  String get appTagline =>
      'Qualquer arquivo, para qualquer pessoa, em qualquer lugar.';

  @override
  String get homeSendTitle => 'Enviar';

  @override
  String get homeSendBody => 'Escolha arquivos e compartilhe um código curto';

  @override
  String get homeReceiveTitle => 'Receber';

  @override
  String get homeReceiveBody => 'Digite um código e salve os arquivos';

  @override
  String get homeFooter =>
      'Criptografia de ponta a ponta. Sem conta, sem e-mail, sem número de telefone.';

  @override
  String get settings => 'Configurações';

  @override
  String get sendTitle => 'Enviar arquivos';

  @override
  String get sendAddFiles => 'Adicionar arquivos';

  @override
  String get sendEmpty => 'Nada selecionado ainda';

  @override
  String get sendEmptyHint =>
      'Adicione os arquivos que você quer enviar. O destinatário recebe exatamente esses, mais nada.';

  @override
  String get sendStrongCode => 'Código mais forte';

  @override
  String get sendStrongCodeHint =>
      'Três palavras em vez de duas: muito mais difícil de adivinhar';

  @override
  String get sendCreateCode => 'Criar código';

  @override
  String get sendCounting => 'Contando…';

  @override
  String get sendRemove => 'Remover';

  @override
  String get sendPreparing => 'Preparando seus arquivos…';

  @override
  String get sendYourCode => 'Seu código';

  @override
  String get sendCodeHint =>
      'No outro dispositivo, abra o TotalDrop, toque em Receber e digite este código.';

  @override
  String get sendWaiting => 'Aguardando o destinatário…';

  @override
  String get sendCopy => 'Copiar';

  @override
  String get sendCopied => 'Código copiado';

  @override
  String get sendShare => 'Compartilhar';

  @override
  String sendShareText(String code) {
    return 'Receba meus arquivos com o TotalDrop. Código: $code';
  }

  @override
  String get sendSending => 'Enviando';

  @override
  String get sendDone => 'Enviado e verificado';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Todos os $countString arquivos chegaram intactos.',
      many: 'Todos os $countString de arquivos chegaram intactos.',
      one: '$countString arquivo chegou intacto.',
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
      other: '$countString arquivos não chegaram intactos e foram descartados.',
      many:
          '$countString de arquivos não chegaram intactos e foram descartados.',
      one: '$countString arquivo não chegou intacto e foi descartado.',
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
      other: '$countString itens',
      many: '$countString de itens',
      one: '$countString item',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Receber arquivos';

  @override
  String get receiveCodeLabel => 'Código';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Peça o código a quem está enviando.';

  @override
  String get receiveCodeInvalid =>
      'Isso não parece um código. Ele começa com um número, seguido de duas ou três palavras.';

  @override
  String get receivePaste => 'Colar';

  @override
  String get receiveConnect => 'Conectar';

  @override
  String get receiveConnecting => 'Conectando com segurança…';

  @override
  String get receiveOffer => 'Arquivos recebidos';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString arquivos',
      many: '$filesString de arquivos',
      one: '$filesString arquivo',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'e mais $countString';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Salvo em $folder';
  }

  @override
  String get receiveAccept => 'Aceitar';

  @override
  String get receiveDecline => 'Recusar';

  @override
  String get receiveDeclined => 'Você recusou os arquivos.';

  @override
  String get receiveReceiving => 'Recebendo';

  @override
  String get receiveDone => 'Recebido e verificado';

  @override
  String get verificationTitle => 'Palavras de verificação';

  @override
  String get verificationHint =>
      'A outra pessoa vê as mesmas quatro palavras. Se forem diferentes, pare.';

  @override
  String get sameNetworkNote =>
      'Funciona pela internet. Quando os dispositivos não conseguem se alcançar diretamente, a transferência criptografada passa pelo servidor de retransmissão, que não consegue lê-la.';

  @override
  String get done => 'Concluído';

  @override
  String get tryAgain => 'Tentar de novo';

  @override
  String get cancel => 'Cancelar';

  @override
  String get cancelTransferTitle => 'Parar esta transferência?';

  @override
  String get cancelTransferBody =>
      'O outro lado será avisado de que a transferência parou.';

  @override
  String get keepGoing => 'Continuar';

  @override
  String get stop => 'Parar';

  @override
  String get moreInfo => 'Detalhes';

  @override
  String get errorWrongCode =>
      'O código não correspondeu. Ele foi digitado errado ou alguém tentou interceptar a transferência. Nada foi enviado. Comece de novo com um código novo.';

  @override
  String get errorPeerLeft =>
      'O outro lado saiu antes de a transferência terminar.';

  @override
  String get errorDeclined => 'O destinatário recusou os arquivos.';

  @override
  String get errorCancelled => 'O outro lado parou a transferência.';

  @override
  String get errorStopped => 'Parada neste dispositivo.';

  @override
  String get errorInvalidCode => 'Esse código não é válido.';

  @override
  String get errorServer =>
      'Não foi possível acessar o servidor do TotalDrop. Verifique sua conexão com a internet.';

  @override
  String get errorConnect =>
      'Não foi possível conectar ao outro dispositivo. Verifique se os dois dispositivos estão on-line e tente de novo.';

  @override
  String get errorInvalidOffer =>
      'A oferta violou uma regra de segurança e por isso foi recusada.';

  @override
  String get errorStorage => 'Não foi possível ler ou salvar um arquivo.';

  @override
  String get errorOther => 'Algo deu errado.';

  @override
  String get settingsServer => 'Servidor de encontro';

  @override
  String get settingsServerHelp =>
      'Ajuda dois dispositivos com o mesmo código a se encontrarem. Ele nunca vê seus arquivos nem suas chaves.';

  @override
  String get settingsServerReset => 'Usar o padrão';

  @override
  String get settingsServerInvalid => 'Precisa começar com wss://';

  @override
  String get settingsSave => 'Salvar';

  @override
  String get settingsAbout => 'Sobre';

  @override
  String get settingsAboutBody =>
      'O TotalDrop é gratuito e de código aberto, mantido por doações. Licença GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Privacidade';

  @override
  String get settingsPrivacyBody =>
      'O servidor de encontro, o servidor de retransmissão e o servidor STUN da Cloudflare veem seu endereço IP, e o outro dispositivo também vê quando vocês se conectam diretamente (a menos que o modo de privacidade esteja ativado). Nenhum deles vê nomes de arquivos, conteúdos ou chaves, e nossos servidores não guardam registros. Os arquivos vão diretamente entre os dispositivos quando possível; caso contrário, passam pelo servidor de retransmissão. Nos dois casos, têm criptografia de ponta a ponta.';

  @override
  String get privacyMode => 'Ocultar meu endereço do outro dispositivo';

  @override
  String get privacyModeHint =>
      'Modo de privacidade: tudo passa pelo servidor de retransmissão, então o outro dispositivo nunca descobre seu endereço IP. É mais lento, e o outro dispositivo precisa usar o mesmo servidor de retransmissão (o padrão usa).';

  @override
  String get privacyModeConfirmTitle => 'Ativar o modo de privacidade?';

  @override
  String get privacyModeConfirmBody =>
      'As transferências ficarão mais lentas, porque nada passa diretamente entre os dispositivos. O servidor de encontro e o servidor de retransmissão continuam vendo seu endereço IP; eles nunca veem seus arquivos nem suas chaves.';

  @override
  String get privacyModeTurnOn => 'Ativar';

  @override
  String settingsVersion(String version) {
    return 'Versão $version';
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
      'Esta tradução está em revisão. Ajude a melhorá-la no GitHub.';

  @override
  String get sendDropHere => 'Solte os arquivos aqui';

  @override
  String get sendDropHint =>
      'Você também pode arrastar arquivos para esta janela.';

  @override
  String get sendQrHint =>
      'Ou escaneie este código com a câmera do outro dispositivo.';

  @override
  String get sendWaitingAccept =>
      'Confira as palavras e aguarde o destinatário aceitar.';

  @override
  String get receiveFromImage => 'Ler QR de uma imagem';

  @override
  String get receiveQrNotFound =>
      'Não há nenhum código do TotalDrop nessa imagem.';

  @override
  String get pickTitle => 'Escolha os arquivos para enviar';

  @override
  String get pickImageTitle => 'Escolha uma imagem com um QR code';

  @override
  String get openFolder => 'Abrir pasta';

  @override
  String get back => 'Voltar';

  @override
  String get codeSuggestions => 'Sugestões';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}

/// The translations for Portuguese, as used in Portugal (`pt_PT`).
class AppLocalizationsPtPt extends AppLocalizationsPt {
  AppLocalizationsPtPt() : super('pt_PT');

  @override
  String get appTagline =>
      'Qualquer ficheiro, para qualquer pessoa, em qualquer lugar.';

  @override
  String get homeSendTitle => 'Enviar';

  @override
  String get homeSendBody => 'Escolha ficheiros e partilhe um código curto';

  @override
  String get homeReceiveTitle => 'Receber';

  @override
  String get homeReceiveBody => 'Introduza um código e guarde os ficheiros';

  @override
  String get homeFooter =>
      'Encriptação ponto a ponto. Sem conta, sem e-mail, sem número de telefone.';

  @override
  String get settings => 'Definições';

  @override
  String get sendTitle => 'Enviar ficheiros';

  @override
  String get sendAddFiles => 'Adicionar ficheiros';

  @override
  String get sendEmpty => 'Ainda não há nada selecionado';

  @override
  String get sendEmptyHint =>
      'Adicione os ficheiros que pretende enviar. O destinatário recebe exatamente estes, mais nada.';

  @override
  String get sendStrongCode => 'Código mais forte';

  @override
  String get sendStrongCodeHint =>
      'Três palavras em vez de duas: muito mais difícil de adivinhar';

  @override
  String get sendCreateCode => 'Criar código';

  @override
  String get sendCounting => 'A contar…';

  @override
  String get sendRemove => 'Remover';

  @override
  String get sendPreparing => 'A preparar os seus ficheiros…';

  @override
  String get sendYourCode => 'O seu código';

  @override
  String get sendCodeHint =>
      'No outro dispositivo, abra o TotalDrop, toque em Receber e introduza este código.';

  @override
  String get sendWaiting => 'A aguardar o destinatário…';

  @override
  String get sendCopy => 'Copiar';

  @override
  String get sendCopied => 'Código copiado';

  @override
  String get sendShare => 'Partilhar';

  @override
  String sendShareText(String code) {
    return 'Receba os meus ficheiros com o TotalDrop. Código: $code';
  }

  @override
  String get sendSending => 'A enviar';

  @override
  String get sendDone => 'Enviado e verificado';

  @override
  String sendDoneBody(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    String _temp0 = intl.Intl.pluralLogic(
      count,
      locale: localeName,
      other: 'Todos os $countString ficheiros chegaram intactos.',
      many: 'Todos os $countString de ficheiros chegaram intactos.',
      one: 'O ficheiro chegou intacto.',
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
          '$countString ficheiros não chegaram intactos e foram descartados.',
      many:
          '$countString de ficheiros não chegaram intactos e foram descartados.',
      one: 'Um ficheiro não chegou intacto e foi descartado.',
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
      other: '$countString itens',
      many: '$countString de itens',
      one: '$countString item',
    );
    return '$_temp0 · $size';
  }

  @override
  String get receiveTitle => 'Receber ficheiros';

  @override
  String get receiveCodeLabel => 'Código';

  @override
  String get receiveCodeHint => '7-harbor-rocket';

  @override
  String get receiveCodeHelp => 'Peça o código a quem está a enviar.';

  @override
  String get receiveCodeInvalid =>
      'Isto não parece um código. Começa com um número, seguido de duas ou três palavras.';

  @override
  String get receivePaste => 'Colar';

  @override
  String get receiveConnect => 'Ligar';

  @override
  String get receiveConnecting => 'A estabelecer ligação segura…';

  @override
  String get receiveOffer => 'Ficheiros recebidos';

  @override
  String receiveOfferSummary(int files, String size) {
    final intl.NumberFormat filesNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String filesString = filesNumberFormat.format(files);

    String _temp0 = intl.Intl.pluralLogic(
      files,
      locale: localeName,
      other: '$filesString ficheiros',
      many: '$filesString de ficheiros',
      one: '$filesString ficheiro',
    );
    return '$_temp0 · $size';
  }

  @override
  String receiveAndMore(int count) {
    final intl.NumberFormat countNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String countString = countNumberFormat.format(count);

    return 'e mais $countString';
  }

  @override
  String receiveSaveTo(String folder) {
    return 'Guardado em $folder';
  }

  @override
  String get receiveAccept => 'Aceitar';

  @override
  String get receiveDecline => 'Recusar';

  @override
  String get receiveDeclined => 'Recusou os ficheiros.';

  @override
  String get receiveReceiving => 'A receber';

  @override
  String get receiveDone => 'Recebido e verificado';

  @override
  String get verificationTitle => 'Palavras de verificação';

  @override
  String get verificationHint =>
      'A outra pessoa vê as mesmas quatro palavras. Se forem diferentes, pare.';

  @override
  String get sameNetworkNote =>
      'Funciona através da Internet. Quando os dispositivos não conseguem comunicar diretamente, a transferência encriptada passa pelo servidor de retransmissão, que não a consegue ler.';

  @override
  String get done => 'Concluído';

  @override
  String get tryAgain => 'Tentar novamente';

  @override
  String get cancel => 'Cancelar';

  @override
  String get cancelTransferTitle => 'Parar esta transferência?';

  @override
  String get cancelTransferBody =>
      'O outro lado será informado de que a transferência parou.';

  @override
  String get keepGoing => 'Continuar';

  @override
  String get stop => 'Parar';

  @override
  String get moreInfo => 'Detalhes';

  @override
  String get errorWrongCode =>
      'O código não correspondeu. Foi mal escrito ou alguém tentou intercetar a transferência. Nada foi enviado. Comece de novo com um código novo.';

  @override
  String get errorPeerLeft =>
      'O outro lado saiu antes de a transferência terminar.';

  @override
  String get errorDeclined => 'O destinatário recusou os ficheiros.';

  @override
  String get errorCancelled => 'O outro lado parou a transferência.';

  @override
  String get errorStopped => 'Parada neste dispositivo.';

  @override
  String get errorInvalidCode => 'Esse código não é válido.';

  @override
  String get errorServer =>
      'Não foi possível contactar o servidor do TotalDrop. Verifique a sua ligação à Internet.';

  @override
  String get errorConnect =>
      'Não foi possível ligar ao outro dispositivo. Verifique se ambos os dispositivos estão online e tente novamente.';

  @override
  String get errorInvalidOffer =>
      'A oferta violou uma regra de segurança e por isso foi recusada.';

  @override
  String get errorStorage => 'Não foi possível ler ou guardar um ficheiro.';

  @override
  String get errorOther => 'Ocorreu um problema.';

  @override
  String get settingsServer => 'Servidor de encontro';

  @override
  String get settingsServerHelp =>
      'Ajuda dois dispositivos com o mesmo código a encontrarem-se. Nunca vê os seus ficheiros nem as suas chaves.';

  @override
  String get settingsServerReset => 'Usar o predefinido';

  @override
  String get settingsServerInvalid => 'Tem de começar por wss://';

  @override
  String get settingsSave => 'Guardar';

  @override
  String get settingsAbout => 'Acerca de';

  @override
  String get settingsAboutBody =>
      'O TotalDrop é gratuito e de código aberto, financiado por donativos. Licença GPL-3.0-or-later.';

  @override
  String get settingsPrivacy => 'Privacidade';

  @override
  String get settingsPrivacyBody =>
      'O servidor de encontro, o servidor de retransmissão e o servidor STUN da Cloudflare veem o seu endereço IP, tal como o outro dispositivo quando há uma ligação direta (a não ser que o modo de privacidade esteja ativado). Nenhum deles vê nomes de ficheiros, conteúdos ou chaves, e os nossos servidores não guardam registos. Os ficheiros vão diretamente entre os dispositivos sempre que possível; caso contrário, passam pelo servidor de retransmissão. Em ambos os casos, têm encriptação ponto a ponto.';

  @override
  String get privacyMode => 'Ocultar o meu endereço do outro dispositivo';

  @override
  String get privacyModeHint =>
      'Modo de privacidade: tudo passa pelo servidor de retransmissão, pelo que o outro dispositivo nunca fica a saber o seu endereço IP. É mais lento, e o outro dispositivo tem de usar o mesmo servidor de retransmissão (o predefinido usa).';

  @override
  String get privacyModeConfirmTitle => 'Ativar o modo de privacidade?';

  @override
  String get privacyModeConfirmBody =>
      'As transferências serão mais lentas, porque nada passa diretamente entre os dispositivos. O servidor de encontro e o servidor de retransmissão continuam a ver o seu endereço IP; nunca veem os seus ficheiros nem as suas chaves.';

  @override
  String get privacyModeTurnOn => 'Ativar';

  @override
  String settingsVersion(String version) {
    return 'Versão $version';
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
      'Esta tradução está em revisão. Ajude a melhorá-la no GitHub.';

  @override
  String get sendDropHere => 'Largue os ficheiros aqui';

  @override
  String get sendDropHint => 'Também pode arrastar ficheiros para esta janela.';

  @override
  String get sendQrHint =>
      'Ou leia este código com a câmara do outro dispositivo.';

  @override
  String get sendWaitingAccept =>
      'Confira as palavras e aguarde que o destinatário aceite.';

  @override
  String get receiveFromImage => 'Ler QR de uma imagem';

  @override
  String get receiveQrNotFound =>
      'Não há nenhum código do TotalDrop nessa imagem.';

  @override
  String get pickTitle => 'Escolha os ficheiros a enviar';

  @override
  String get pickImageTitle => 'Escolha uma imagem com um código QR';

  @override
  String get openFolder => 'Abrir pasta';

  @override
  String get back => 'Voltar';

  @override
  String get codeSuggestions => 'Sugestões';

  @override
  String progressPercent(int percent) {
    final intl.NumberFormat percentNumberFormat =
        intl.NumberFormat.decimalPattern(localeName);
    final String percentString = percentNumberFormat.format(percent);

    return '$percentString%';
  }
}
