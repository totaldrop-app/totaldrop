// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:material_ui/material_ui.dart';
import 'package:flutter/services.dart';
import 'package:share_plus/share_plus.dart';

import '../../l10n/app_localizations.dart';
import '../core/adaptive.dart';
import '../core/common.dart';
import '../core/l10n.dart';
import '../core/mark.dart';
import '../core/theme.dart';
import 'receive_view_model.dart';

const _openUrlChannel = MethodChannel('app.totaldrop/open_url');

/// Maximum UTF-16 code units offered to Android's system share sheet via
/// EXTRA_TEXT so Binder parcels stay well below the 1 MiB transaction limit.
const maxSharedTextChars = 128 * 1024;

class ReceiveView extends StatefulWidget {
  const ReceiveView({super.key, required this.viewModel, this.onShareText});

  final ReceiveViewModel viewModel;
  final Future<void> Function(String text)? onShareText;

  @override
  State<ReceiveView> createState() => _ReceiveViewState();
}

class _ReceiveViewState extends State<ReceiveView> {
  late final _controller = TextEditingController(text: widget.viewModel.input);

  @override
  void initState() {
    super.initState();
    widget.viewModel.addListener(_showInput);
  }

  @override
  void dispose() {
    widget.viewModel.removeListener(_showInput);
    _controller.dispose();
    super.dispose();
  }

  /// A link can fill in the code from outside the field (#445).
  void _showInput() {
    final input = widget.viewModel.input;
    if (_controller.text == input) return;
    _controller.value = TextEditingValue(
      text: input,
      selection: TextSelection.collapsed(offset: input.length),
    );
  }

  Future<bool> _confirmStop(BuildContext context) async {
    final l = AppLocalizations.of(context);
    return await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: Text(l.cancelTransferTitle),
            content: Text(l.cancelTransferBody),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: Text(l.keepGoing),
              ),
              FilledButton(
                onPressed: () => Navigator.pop(context, true),
                child: Text(l.stop),
              ),
            ],
          ),
        ) ??
        false;
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final vm = widget.viewModel;
    return ListenableBuilder(
      listenable: vm,
      builder: (context, _) {
        return PopScope(
          canPop: !vm.busy && vm.stage != ReceiveStage.offered,
          onPopInvokedWithResult: (didPop, _) async {
            if (didPop) {
              if (vm.stage != ReceiveStage.entering) vm.reset();
              _controller.clear();
              return;
            }
            if (vm.stage == ReceiveStage.offered) {
              await vm.decline();
              vm.reset();
              if (context.mounted) Navigator.pop(context);
              return;
            }
            if (await _confirmStop(context)) {
              vm.reset();
              if (context.mounted) Navigator.pop(context);
            }
          },
          child: Scaffold(
            appBar: AppBar(title: Text(l.receiveTitle)),
            body: SafeArea(
              child: AnimatedSwitcher(
                duration: const Duration(milliseconds: 250),
                child: KeyedSubtree(
                  key: ValueKey(vm.stage),
                  child: switch (vm.stage) {
                    ReceiveStage.entering => _Entering(
                      vm: vm,
                      controller: _controller,
                    ),
                    ReceiveStage.connecting => CenteredScroll(
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          const TotalDropMark(size: 120, sending: true),
                          Gap.l,
                          Announced(
                            Text(
                              l.receiveConnecting,
                              textAlign: TextAlign.center,
                              style: Theme.of(context).textTheme.titleMedium,
                            ),
                          ),
                          if (vm.connectingCode case final code?) ...[
                            Gap.s,
                            // The code is English words, left to right in
                            // every language.
                            Text(
                              code,
                              textAlign: TextAlign.center,
                              textDirection: TextDirection.ltr,
                              style: Theme.of(context).textTheme.titleLarge,
                            ),
                          ],
                        ],
                      ),
                    ),
                    ReceiveStage.offered => _Offered(vm: vm),
                    ReceiveStage.transferring => _Transferring(vm: vm),
                    ReceiveStage.done => _Done(
                      vm: vm,
                      onShareText: widget.onShareText,
                    ),
                    ReceiveStage.declined => _Message(
                      icon: Icons.block,
                      text: l.receiveDeclined,
                      onDone: () {
                        vm.reset();
                        Navigator.pop(context);
                      },
                    ),
                    ReceiveStage.failed => AdaptiveErrorStage(
                      error: vm.error!,
                      onRetry: () {
                        vm.reset();
                        _controller.clear();
                      },
                    ),
                  },
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

class _Entering extends StatefulWidget {
  const _Entering({required this.vm, required this.controller});
  final ReceiveViewModel vm;
  final TextEditingController controller;

  @override
  State<_Entering> createState() => _EnteringState();
}

class _EnteringState extends State<_Entering> {
  // Owned inside AnimatedSwitcher's child state so rapid stage transitions
  // never collide on GlobalKey while AdaptivePaneLayout posture/keyboard
  // switches preserve focus and prevent autofocus bounce loops (#632).
  final _fieldKey = GlobalKey();
  final _focusNode = FocusNode();

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final vm = widget.vm;
    final controller = widget.controller;
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);

    final field = TextField(
      key: _fieldKey,
      focusNode: _focusNode,
      controller: controller,
      autofocus: true,
      // The code is English words, left to right in every language.
      textDirection: TextDirection.ltr,
      autocorrect: false,
      enableSuggestions: false,
      keyboardType: TextInputType.visiblePassword,
      textInputAction: TextInputAction.go,
      style: theme.textTheme.headlineSmall,
      decoration: InputDecoration(
        labelText: l.receiveCodeLabel,
        hintText: l.receiveCodeHint,
        // The example code must never read as typed text.
        hintStyle: theme.textTheme.headlineSmall?.copyWith(
          color: theme.colorScheme.onSurfaceVariant.withValues(alpha: 0.45),
          fontWeight: FontWeight.w500,
        ),
        helperText: l.receiveCodeHelp,
        errorText: vm.showInvalid ? l.receiveCodeInvalid : null,
        // All of it, in every language and text size: at 200 % the
        // Japanese message ran past three lines and lost its end
        // (#480). errorText cannot go without a limit, so the limit is
        // one no message reaches; errorText, not the error widget,
        // because only it is read out with the field.
        errorMaxLines: 99,
        prefixIcon: const Icon(Icons.key_outlined),
        suffixIcon: KeepIconButtonVariant(
          child: IconButton(
            tooltip: l.receivePaste,
            icon: const Icon(Icons.content_paste),
            onPressed: () async {
              final data = await Clipboard.getData(Clipboard.kTextPlain);
              final text = data?.text;
              if (text == null) return;
              final value = codeFromPaste(text);
              if (value.isEmpty) return;
              controller.text = value;
              vm.setInput(value);
            },
          ),
        ),
      ),
      onChanged: vm.setInput,
      onSubmitted: (_) => vm.connect(),
    );

    final connectButton = FilledButton.icon(
      onPressed: vm.input.trim().isEmpty ? null : vm.connect,
      icon: const Icon(Icons.link),
      label: Text(l.receiveConnect),
    );

    return AdaptivePaneLayout(
      primary: Builder(
        builder: (context) {
          final isTabletop = AdaptivePaneLayout.isTabletopOf(context);
          return CenteredScroll(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Center(child: TotalDropMark(size: isTabletop ? 80 : 96)),
                if (!isTabletop) ...[Gap.xl, const SameNetworkNote()],
              ],
            ),
          );
        },
      ),
      secondary: Builder(
        builder: (context) {
          final isTabletop = AdaptivePaneLayout.isTabletopOf(context);
          return CenteredScroll(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                field,
                Gap.l,
                connectButton,
                if (isTabletop) ...[Gap.l, const SameNetworkNote()],
              ],
            ),
          );
        },
      ),
      singlePane: ColumnListView(
        children: [
          const Center(child: TotalDropMark(size: 96)),
          Gap.l,
          field,
          Gap.l,
          connectButton,
          Gap.l,
          const SameNetworkNote(),
        ],
      ),
    );
  }
}

class _Offered extends StatelessWidget {
  const _Offered({required this.vm});
  final ReceiveViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final o = vm.offer!;
    final names = o.names;
    if (vm.isText) return _OfferedText(vm: vm);

    final offerItems = <Widget>[
      Announced(Text(l.receiveOffer, style: theme.textTheme.headlineSmall)),
      Gap.s,
      Text(
        l.receiveOfferSummary(o.files, formatBytes(l, o.totalBytes)),
        style: theme.textTheme.titleMedium?.copyWith(
          fontFeatures: const [FontFeature.tabularFigures()],
        ),
      ),
      Gap.m,
      const Divider(),
      for (final n in names.take(6)) ...[
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 12),
          child: Row(
            children: [
              Icon(
                Icons.insert_drive_file_outlined,
                size: 20,
                color: theme.colorScheme.onSurfaceVariant,
              ),
              Gap.s,
              // Names come from the other side: hidden direction
              // characters are shown, not obeyed.
              Expanded(
                child: Text(
                  revealControls(n),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
        ),
        const Divider(),
      ],
      if (names.length > 6)
        Padding(
          padding: const EdgeInsets.only(top: 12),
          child: Text(
            l.receiveAndMore(names.length - 6),
            style: TextStyle(color: theme.colorScheme.onSurfaceVariant),
          ),
        ),
    ];

    final decision = <Widget>[
      VerificationWords(words: o.words),
      Gap.l,
      ButtonPair(
        outlined: PairedButton(
          safeL10n(context, 'receiveDecline').receiveDecline,
          vm.decline,
        ),
        filled: PairedButton(
          safeL10n(context, 'receiveAccept').receiveAccept,
          vm.accept,
        ),
      ),
    ];

    return AdaptivePaneLayout(
      primary: ColumnListView(children: offerItems),
      secondary: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: decision,
        ),
      ),
      singlePane: ColumnListView(children: [...offerItems, Gap.l, ...decision]),
    );
  }
}

/// A text offered (#326): shown once accepted, never saved.
class _OfferedText extends StatelessWidget {
  const _OfferedText({required this.vm});
  final ReceiveViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final o = vm.offer!;

    final summary = <Widget>[
      Announced(Text(l.receiveOfferText, style: theme.textTheme.headlineSmall)),
      Gap.s,
      Text(
        formatBytes(l, o.totalBytes),
        style: theme.textTheme.titleMedium?.copyWith(
          fontFeatures: const [FontFeature.tabularFigures()],
        ),
      ),
      Gap.s,
      Text(
        l.receiveOfferTextNote,
        style: theme.textTheme.bodyMedium?.copyWith(
          color: theme.colorScheme.onSurfaceVariant,
        ),
      ),
    ];

    final decision = <Widget>[
      VerificationWords(words: o.words),
      Gap.l,
      ButtonPair(
        outlined: PairedButton(
          safeL10n(context, 'receiveDecline').receiveDecline,
          vm.decline,
        ),
        filled: PairedButton(
          safeL10n(context, 'receiveAccept').receiveAccept,
          vm.accept,
        ),
      ),
    ];

    return AdaptivePaneLayout(
      primary: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: summary,
        ),
      ),
      secondary: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: decision,
        ),
      ),
      singlePane: ColumnListView(children: [...summary, Gap.l, ...decision]),
    );
  }
}

class _Transferring extends StatelessWidget {
  const _Transferring({required this.vm});
  final ReceiveViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    Widget buildStatus({required bool isTabletop}) => Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Center(
          child: TotalDropMark(size: isTabletop ? 96 : 140, sending: true),
        ),
        Gap.l,
        // Keep the live-region headline separate from the sibling
        // TransferProgress bar inside two-pane layouts.
        Announced(
          Text(
            l.receiveReceiving,
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          container: true,
        ),
        Gap.m,
        TransferProgress(done: vm.done, total: vm.total),
      ],
    );
    if (vm.offer == null) {
      return AdaptiveSinglePane(
        child: Builder(
          builder: (context) => ColumnListView(
            children: [
              buildStatus(isTabletop: AdaptivePaneLayout.isTabletopOf(context)),
            ],
          ),
        ),
      );
    }
    final words = VerificationWords(words: vm.offer!.words);
    return AdaptivePaneLayout(
      primary: Builder(
        builder: (context) => CenteredScroll(
          child: buildStatus(
            isTabletop: AdaptivePaneLayout.isTabletopOf(context),
          ),
        ),
      ),
      secondary: CenteredScroll(child: words),
      singlePane: ColumnListView(
        children: [buildStatus(isTabletop: false), Gap.l, words],
      ),
    );
  }
}

class _Done extends StatelessWidget {
  const _Done({required this.vm, this.onShareText});
  final ReceiveViewModel vm;
  final Future<void> Function(String text)? onShareText;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final s = vm.summary;
    if (vm.receivedText case final text?) {
      return _TextDone(vm: vm, text: text, onShareText: onShareText);
    }

    final header = <Widget>[
      const Center(child: DoneBadge()),
      Gap.l,
      Announced(
        Text(
          l.receiveDone,
          textAlign: TextAlign.center,
          style: theme.textTheme.headlineSmall,
        ),
      ),
      Gap.s,
      if (vm.folder != null)
        Text(
          l.receiveSaveTo(isolate(displayReceiveFolder(vm.folder!))),
          textAlign: TextAlign.center,
        ),
      if (s != null && s.filesFailed > 0) ...[
        Gap.s,
        Text(
          l.sendSomeFailed(s.filesFailed),
          textAlign: TextAlign.center,
          style: TextStyle(color: theme.colorScheme.error),
        ),
      ],
    ];

    final savedTiles = <Widget>[
      if (s != null)
        for (final p in s.saved)
          ListTile(
            leading: Icon(
              Icons.check,
              color: theme.colorScheme.onSurfaceVariant,
            ),
            title: Text(
              revealControls(p.split('/').last),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ),
    ];

    final doneButton = FilledButton(
      onPressed: () {
        vm.reset();
        Navigator.pop(context);
      },
      child: Text(l.done),
    );

    if (savedTiles.isEmpty) {
      return AdaptivePaneLayout(
        allowFlatTwoPane: false,
        primary: CenteredScroll(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: header,
          ),
        ),
        secondary: CenteredScroll(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [doneButton],
          ),
        ),
        singlePane: CenteredScroll(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [...header, Gap.l, doneButton],
          ),
        ),
      );
    }

    return AdaptivePaneLayout(
      primary: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: header,
        ),
      ),
      secondary: ColumnListView(children: [...savedTiles, Gap.l, doneButton]),
      singlePane: ColumnListView(
        children: [...header, Gap.m, ...savedTiles, Gap.l, doneButton],
      ),
    );
  }
}

/// A text received (#326, #53): shown as plain text with hidden characters
/// named, never saved. Copy and Share pass it on as it came; a single web
/// link can be opened after showing its full normalized address.
class _TextDone extends StatelessWidget {
  const _TextDone({required this.vm, required this.text, this.onShareText});
  final ReceiveViewModel vm;
  final String text;
  final Future<void> Function(String text)? onShareText;

  Future<void> _confirmAndOpenLink(BuildContext context, String url) async {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        scrollable: true,
        title: Text(l.openLinkTitle),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(l.openLinkBody),
            Gap.m,
            DecoratedBox(
              decoration: BoxDecoration(
                border: Border.all(color: theme.colorScheme.outlineVariant),
                borderRadius: const BorderRadius.all(Radius.circular(12)),
              ),
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: SelectableText(
                  url,
                  style: theme.textTheme.bodyMedium,
                  textDirection: TextDirection.ltr,
                ),
              ),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(l.cancel),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(l.openLinkConfirm),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    bool? opened;
    try {
      opened = await _openUrlChannel.invokeMethod<bool>('open', url);
    } on PlatformException {
      opened = false;
    } on MissingPluginException {
      opened = false;
    }
    if (!(opened ?? false) && context.mounted) {
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text(l.openLinkFailed)));
    }
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final shown = shownText(text);
    final url = singleWebUrl(text);
    final note = theme.textTheme.bodyMedium?.copyWith(
      color: theme.colorScheme.onSurfaceVariant,
    );
    final screenHeight = MediaQuery.sizeOf(context).height;

    List<Widget> buildTextSection({required bool isTabletop}) => <Widget>[
      const Center(child: DoneBadge()),
      Gap.l,
      Announced(
        Text(
          l.receiveTextDone,
          textAlign: TextAlign.center,
          style: theme.textTheme.headlineSmall,
        ),
      ),
      Gap.m,
      // The text scrolls in its own box, at most half the screen high (or 30%
      // when actively split in tabletop), and the page scrolls when large type
      // leaves no room.
      ConstrainedBox(
        constraints: BoxConstraints(
          maxHeight: screenHeight * (isTabletop ? 0.3 : 0.5),
        ),
        child: DecoratedBox(
          decoration: BoxDecoration(
            border: Border.all(color: theme.colorScheme.outlineVariant),
            borderRadius: const BorderRadius.all(Radius.circular(12)),
          ),
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            // Plain text: nothing in it is a link or markup.
            child: SelectableText(shown.text),
          ),
        ),
      ),
      if (shown.short) ...[Gap.s, Text(l.receiveTextShortened, style: note)],
      if (shown.hidden) ...[Gap.s, Text(l.receiveTextHidden, style: note)],
    ];

    final actions = <Widget>[
      if (url != null) ...[
        OutlinedButton.icon(
          onPressed: () => _confirmAndOpenLink(context, url),
          icon: const Icon(Icons.open_in_new),
          label: Text(l.openLink),
        ),
        Gap.s,
      ],
      OutlinedButton.icon(
        onPressed: () async {
          await Clipboard.setData(ClipboardData(text: text));
          if (context.mounted) {
            ScaffoldMessenger.of(context)
                .showSnackBar(SnackBar(content: Text(l.textCopied)));
          }
        },
        icon: const Icon(Icons.copy),
        label: Text(l.copyText),
      ),
      Gap.s,
      // Any text above 128 Ki UTF-16 code units exceeds SHOWN_CHARS (16 Ki
      // scalar values), so receiveTextShortened above already points to Copy text.
      if (text.length <= maxSharedTextChars) ...[
        OutlinedButton.icon(
          onPressed: () async {
            try {
              final share = onShareText;
              if (share != null) {
                await share(text);
              } else {
                await SharePlus.instance.share(ShareParams(text: text));
              }
            } on PlatformException {
              if (context.mounted) {
                ScaffoldMessenger.of(context)
                    .showSnackBar(SnackBar(content: Text(l.errorOther)));
              }
            }
          },
          icon: const Icon(Icons.share_outlined),
          label: Text(l.shareText),
        ),
        Gap.s,
      ],
      FilledButton(
        onPressed: () {
          vm.reset();
          Navigator.pop(context);
        },
        child: Text(l.done),
      ),
    ];

    return AdaptivePaneLayout(
      primary: Builder(
        builder: (context) => ColumnListView(
          children: buildTextSection(
            isTabletop: AdaptivePaneLayout.isTabletopOf(context),
          ),
        ),
      ),
      secondary: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: actions,
        ),
      ),
      singlePane: ColumnListView(
        children: [...buildTextSection(isTabletop: false), Gap.m, ...actions],
      ),
    );
  }
}

class _Message extends StatelessWidget {
  const _Message({
    required this.icon,
    required this.text,
    required this.onDone,
  });
  final IconData icon;
  final String text;
  final VoidCallback onDone;

  @override
  Widget build(BuildContext context) {
    final header = <Widget>[
      Icon(
        icon,
        size: 56,
        color: Theme.of(context).colorScheme.onSurfaceVariant,
      ),
      Gap.m,
      Announced(
        Text(
          text,
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.titleMedium,
        ),
      ),
    ];
    final doneButton = FilledButton(
      onPressed: onDone,
      child: Text(AppLocalizations.of(context).done),
    );
    return AdaptivePaneLayout(
      allowFlatTwoPane: false,
      primary: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: header,
        ),
      ),
      secondary: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [doneButton],
        ),
      ),
      singlePane: CenteredScroll(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [...header, Gap.l, doneButton],
        ),
      ),
    );
  }
}

/// Shortens `/storage/emulated/<userId>/Download/TotalDrop` to
/// `Download/TotalDrop` for primary, secondary, and work-profile users (#592).
String displayReceiveFolder(String path) =>
    path.replaceFirst(RegExp(r'^/storage/emulated/\d+/'), '');
