// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later

import 'package:material_ui/material_ui.dart';
import 'package:flutter/semantics.dart';
import 'package:flutter/services.dart';
import 'package:share_plus/share_plus.dart';

import '../../l10n/app_localizations.dart';
import '../core/common.dart';
import '../core/l10n.dart';
import '../core/mark.dart';
import '../core/theme.dart';
import 'send_view_model.dart';

class SendView extends StatelessWidget {
  const SendView({super.key, required this.viewModel});

  final SendViewModel viewModel;

  Future<bool> _confirmStop(BuildContext context) async {
    final l = AppLocalizations.of(context);
    final stop = await showDialog<bool>(
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
    );
    return stop ?? false;
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    return ListenableBuilder(
      listenable: viewModel,
      builder: (context, _) {
        final vm = viewModel;
        return PopScope(
          canPop: !vm.busy,
          onPopInvokedWithResult: (didPop, _) async {
            if (didPop) {
              if (vm.stage != SendStage.picking) {
                vm.reset();
              } else {
                // Leaving the wait ends it: the shares stop, and what
                // was chosen before stays. A text being written goes.
                vm.cancelShares();
                vm.leaveText();
              }
              return;
            }
            if (await _confirmStop(context)) {
              vm.reset();
              if (context.mounted) Navigator.pop(context);
            }
          },
          child: Scaffold(
            appBar: AppBar(title: Text(vm.textMode ? l.sendText : l.sendTitle)),
            body: SafeArea(
              child: AnimatedSwitcher(
                duration: const Duration(milliseconds: 250),
                child: KeyedSubtree(
                  key: ValueKey((vm.stage, vm.copying || vm.sharing)),
                  child: switch (vm.stage) {
                    // The picker copies the chosen files into the app
                    // cache before it hands them over (#269).
                    SendStage.picking when vm.copying || vm.sharing =>
                      _Centered(
                        logo: const TotalDropMark(size: 120, sending: true),
                        text: l.sendPreparing,
                        // A share's copy can be stopped; the picker's
                        // belongs to the picker.
                        action: vm.sharing && !vm.copying
                            ? TextButton(
                                onPressed: vm.cancelShares,
                                child: Text(l.cancel),
                              )
                            : null,
                      ),
                    SendStage.picking when vm.textMode => _TextPicking(vm: vm),
                    SendStage.picking => _Picking(vm: vm),
                    SendStage.preparing => _Centered(
                      logo: const TotalDropMark(size: 120, sending: true),
                      text: l.sendPreparing,
                    ),
                    SendStage.waiting => _Waiting(vm: vm),
                    SendStage.transferring => _Transferring(vm: vm),
                    SendStage.done => _Done(vm: vm),
                    SendStage.failed => Padding(
                      padding: const EdgeInsets.all(24),
                      child: Center(
                        child: ErrorPanel(error: vm.error!, onRetry: vm.start),
                      ),
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

class _Centered extends StatelessWidget {
  const _Centered({required this.logo, required this.text, this.action});
  final Widget logo;
  final String text;
  final Widget? action;

  @override
  Widget build(BuildContext context) => Center(
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        logo,
        Gap.l,
        Announced(Text(text, style: Theme.of(context).textTheme.titleMedium)),
        if (action case final a?) ...[Gap.l, a],
      ],
    ),
  );
}

class _Picking extends StatelessWidget {
  const _Picking({required this.vm});
  final SendViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    if (vm.files.isEmpty) {
      // One screen, one job: the only action is choosing files.
      return Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Spacer(),
            Icon(
              Icons.upload_file_outlined,
              size: 56,
              color: theme.colorScheme.onSurfaceVariant,
            ),
            Gap.m,
            Text(
              l.sendEmpty,
              style: theme.textTheme.headlineSmall,
              textAlign: TextAlign.center,
            ),
            Gap.s,
            Text(
              l.sendEmptyHint,
              textAlign: TextAlign.center,
              style: theme.textTheme.bodyLarge?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const Spacer(),
            if (vm.addFailed) ...[
              const _AddFailed(textAlign: TextAlign.center),
              Gap.m,
            ],
            FilledButton.icon(
              onPressed: vm.addFiles,
              icon: const Icon(Icons.attach_file),
              label: Text(l.sendAddFiles),
            ),
            Gap.s,
            OutlinedButton.icon(
              onPressed: () => vm.setTextMode(true),
              icon: const Icon(Icons.notes),
              label: Text(l.sendText),
            ),
          ],
        ),
      );
    }
    return Column(
      children: [
        Expanded(
          child: ListView.separated(
            padding: const EdgeInsets.fromLTRB(24, 8, 24, 8),
            itemCount: vm.files.length + 1,
            separatorBuilder: (_, _) => const Divider(),
            itemBuilder: (context, i) {
              if (i == vm.files.length) {
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    TextButton.icon(
                      onPressed: vm.addFiles,
                      icon: const Icon(Icons.add),
                      label: Text(l.sendAddFiles),
                    ),
                    if (vm.addFailed)
                      const Padding(
                        padding: EdgeInsetsDirectional.fromSTEB(12, 0, 12, 8),
                        child: _AddFailed(textAlign: TextAlign.start),
                      ),
                  ],
                );
              }
              final f = vm.files[i];
              return ListTile(
                leading: Icon(
                  Icons.insert_drive_file_outlined,
                  color: theme.colorScheme.onSurfaceVariant,
                ),
                title: Text(
                  f.name,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
                subtitle: Text(
                  formatBytes(l, f.size),
                  style: const TextStyle(
                    fontFeatures: [FontFeature.tabularFigures()],
                  ),
                ),
                trailing: KeepIconButtonVariant(
                  child: IconButton(
                    tooltip: l.sendRemove,
                    icon: const Icon(Icons.close),
                    onPressed: () => vm.remove(f),
                  ),
                ),
              );
            },
          ),
        ),
        const Divider(),
        Padding(
          padding: const EdgeInsets.fromLTRB(24, 8, 24, 24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(l.sendStrongCode),
                subtitle: Text(l.sendStrongCodeHint),
                value: vm.strong,
                onChanged: vm.setStrong,
              ),
              Gap.s,
              FilledButton.icon(
                onPressed: vm.start,
                icon: const Icon(Icons.key_outlined),
                // Read out when the list appears after the picker's copy
                // and when it changes, so a screen reader is not left
                // silent where "Add files" was (#271).
                label: Announced(
                  Text(
                    '${l.sendCreateCode} · ${l.itemsAndSize(vm.files.length, formatBytes(l, vm.totalSize))}',
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

/// A text to send instead of files (#326): typed, pasted, shared from
/// another app or selected there. What is sent is the text in the box,
/// as UTF-8; a text too long to edit is shown in part and sent whole.
class _TextPicking extends StatefulWidget {
  const _TextPicking({required this.vm});
  final SendViewModel vm;

  @override
  State<_TextPicking> createState() => _TextPickingState();
}

class _TextPickingState extends State<_TextPicking> {
  late final _controller = TextEditingController(text: _boxText);

  String get _boxText {
    final vm = widget.vm;
    return vm.bigText ? vm.text.substring(0, _cut(vm.text)) : vm.text;
  }

  /// Not inside a character made of two UTF-16 units.
  static int _cut(String t) {
    final end = shownTextChars;
    final last = t.codeUnitAt(end - 1);
    return last >= 0xD800 && last <= 0xDBFF ? end - 1 : end;
  }

  @override
  void didUpdateWidget(_TextPicking old) {
    super.didUpdateWidget(old);
    _sync();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  /// The box follows a text that came from elsewhere (clipboard, share).
  void _sync() {
    final want = _boxText;
    if (_controller.text != want) {
      _controller.value = TextEditingValue(
        text: want,
        selection: TextSelection.collapsed(offset: want.length),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final vm = widget.vm;
    final notice = switch (vm.textNotice) {
      TextNotice.clipboardEmpty => l.sendClipboardEmpty,
      TextNotice.clipboardKept => l.sendClipboardKept,
      TextNotice.none when vm.textTooLong => l.sendTextTooLong(
        formatBytes(l, vm.maxTextBytes),
      ),
      TextNotice.none when vm.bigText => l.sendTextShortened,
      TextNotice.none => null,
    };
    final warn = vm.textTooLong || vm.textNotice != TextNotice.none;
    // The box shows hidden characters as the system draws them, which is
    // often not at all: the screen names them (#326).
    final hidden = vm.text.isNotEmpty && shownText(vm.text).hidden;
    // The box takes the room left; with the keyboard up or large type,
    // the page scrolls instead of overflowing.
    return CustomScrollView(
      slivers: [
        SliverFillRemaining(
          hasScrollBody: false,
          child: Padding(
            padding: const EdgeInsets.fromLTRB(24, 8, 24, 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Expanded(
                  child: ConstrainedBox(
                    constraints: const BoxConstraints(minHeight: 160),
                    child: TextField(
                      controller: _controller,
                      // A long text cannot be edited, only taken away.
                      readOnly: vm.bigText,
                      autofocus: vm.text.isEmpty,
                      maxLines: null,
                      expands: true,
                      textAlignVertical: TextAlignVertical.top,
                      keyboardType: TextInputType.multiline,
                      decoration: InputDecoration(
                        hintText: l.sendTextHint,
                        border: const OutlineInputBorder(),
                      ),
                      onChanged: vm.setText,
                    ),
                  ),
                ),
                if (notice != null) ...[
                  Gap.s,
                  Announced(
                    Text(
                      notice,
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: warn
                            ? theme.colorScheme.error
                            : theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ),
                ],
                if (hidden) ...[
                  Gap.s,
                  Announced(
                    Text(
                      l.sendTextHidden,
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ),
                ],
                Gap.s,
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    if (vm.text.isEmpty)
                      OutlinedButton.icon(
                        onPressed: vm.pasteClipboard,
                        icon: const Icon(Icons.content_paste),
                        label: Text(l.receivePaste),
                      )
                    else
                      OutlinedButton.icon(
                        onPressed: () => vm.setText(''),
                        icon: const Icon(Icons.clear),
                        label: Text(l.sendRemove),
                      ),
                    if (vm.text.isEmpty)
                      OutlinedButton.icon(
                        onPressed: () {
                          vm.setTextMode(false);
                          if (vm.files.isEmpty) vm.addFiles();
                        },
                        icon: const Icon(Icons.attach_file),
                        label: Text(l.sendAddFiles),
                      ),
                  ],
                ),
                const Divider(height: 24),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(l.sendStrongCode),
                  subtitle: Text(l.sendStrongCodeHint),
                  value: vm.strong,
                  onChanged: vm.setStrong,
                ),
                Gap.s,
                FilledButton.icon(
                  onPressed: vm.canStart ? vm.start : null,
                  icon: const Icon(Icons.key_outlined),
                  label: Text(
                    vm.text.isEmpty
                        ? l.sendCreateCode
                        : '${l.sendCreateCode} · ${formatBytes(l, vm.textBytes)}',
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

/// Why the picker came back with nothing (#271).
class _AddFailed extends StatelessWidget {
  const _AddFailed({required this.textAlign});
  final TextAlign textAlign;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Announced(
      Text(
        AppLocalizations.of(context).sendAddFailed,
        textAlign: textAlign,
        style: theme.textTheme.bodyMedium?.copyWith(
          color: theme.colorScheme.error,
        ),
      ),
    );
  }
}

class _Waiting extends StatelessWidget {
  const _Waiting({required this.vm});
  final SendViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final code = vm.code ?? '';
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        const Center(child: TotalDropMark(size: 120, sending: true)),
        Gap.s,
        Announced(
          Text(
            l.sendWaiting,
            textAlign: TextAlign.center,
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
        ),
        Gap.l,
        // The code is the content; the frame stays quiet.
        Card(
          color: theme.colorScheme.surface,
          shape: RoundedRectangleBorder(
            borderRadius: const BorderRadius.all(Radius.circular(28)),
            side: BorderSide(color: theme.colorScheme.outlineVariant),
          ),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              children: [
                Text(
                  l.sendYourCode,
                  style: theme.textTheme.labelLarge?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
                Gap.s,
                // English words, left to right in every language.
                // Spoken with an English voice; the Copy button below copies it.
                Semantics(
                  attributedLabel: AttributedString(
                    code,
                    attributes: [
                      LocaleStringAttribute(
                        range: TextRange(start: 0, end: code.length),
                        locale: const Locale('en'),
                      ),
                    ],
                  ),
                  excludeSemantics: true,
                  child: Directionality(
                    textDirection: TextDirection.ltr,
                    child: SelectableText(
                      code,
                      textAlign: TextAlign.center,
                      style: theme.textTheme.headlineLarge,
                    ),
                  ),
                ),
                Gap.m,
                Row(
                  children: [
                    Expanded(
                      child: OutlinedButton.icon(
                        onPressed: () async {
                          await Clipboard.setData(ClipboardData(text: code));
                          if (context.mounted) {
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(content: Text(l.sendCopied)),
                            );
                          }
                        },
                        icon: const Icon(Icons.copy),
                        label: Text(l.sendCopy),
                      ),
                    ),
                    Gap.s,
                    Expanded(
                      child: FilledButton.icon(
                        // Isolated, the code keeps its order in a right-to-left
                        // sentence; pasted back, the receiver drops the marks.
                        onPressed: () => SharePlus.instance.share(
                          ShareParams(text: l.sendShareText(isolate(code))),
                        ),
                        icon: const Icon(Icons.share),
                        label: Text(l.sendShare),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
        Gap.m,
        Text(
          l.sendCodeHint,
          textAlign: TextAlign.center,
          style: theme.textTheme.bodyLarge,
        ),
        Gap.xl,
        const SameNetworkNote(),
      ],
    );
  }
}

class _Transferring extends StatelessWidget {
  const _Transferring({required this.vm});
  final SendViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        const Center(child: TotalDropMark(size: 140, sending: true)),
        Gap.l,
        Announced(
          Text(
            l.sendSending,
            textAlign: TextAlign.center,
            style: Theme.of(context).textTheme.headlineSmall,
          ),
        ),
        Gap.m,
        TransferProgress(done: vm.done, total: vm.total),
        Gap.l,
        if (vm.words.isNotEmpty) VerificationWords(words: vm.words),
      ],
    );
  }
}

class _Done extends StatelessWidget {
  const _Done({required this.vm});
  final SendViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final s = vm.summary;
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const Center(child: DoneBadge()),
          Gap.l,
          Announced(
            Text(
              l.sendDone,
              textAlign: TextAlign.center,
              style: theme.textTheme.headlineSmall,
            ),
          ),
          Gap.s,
          if (vm.textMode)
            Text(l.sendTextDone, textAlign: TextAlign.center)
          else if (s != null) ...[
            Text(l.sendDoneBody(s.filesOk), textAlign: TextAlign.center),
            if (s.filesFailed > 0) ...[
              Gap.s,
              Text(
                l.sendSomeFailed(s.filesFailed),
                textAlign: TextAlign.center,
                style: TextStyle(color: theme.colorScheme.error),
              ),
            ],
          ],
          Gap.xl,
          FilledButton(
            onPressed: () {
              vm.reset();
              Navigator.pop(context);
            },
            child: Text(l.done),
          ),
        ],
      ),
    );
  }
}
