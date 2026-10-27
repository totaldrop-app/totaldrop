import 'package:material_ui/material_ui.dart';
import 'package:flutter/services.dart';
import 'package:share_plus/share_plus.dart';

import '../../l10n/app_localizations.dart';
import '../core/common.dart';
import '../core/orbit_logo.dart';
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
          TextButton(onPressed: () => Navigator.pop(context, false), child: Text(l.keepGoing)),
          FilledButton(onPressed: () => Navigator.pop(context, true), child: Text(l.stop)),
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
              if (vm.stage != SendStage.picking) vm.reset();
              return;
            }
            if (await _confirmStop(context)) {
              vm.reset();
              if (context.mounted) Navigator.pop(context);
            }
          },
          child: Scaffold(
            appBar: AppBar(title: Text(l.sendTitle)),
            floatingActionButton: vm.stage == SendStage.picking
                ? FloatingActionButton.extended(
                    onPressed: vm.addFiles,
                    icon: const Icon(Icons.add),
                    label: Text(l.sendAddFiles),
                  )
                : null,
            body: SafeArea(
              child: AnimatedSwitcher(
                duration: const Duration(milliseconds: 250),
                child: KeyedSubtree(
                  key: ValueKey(vm.stage),
                  child: switch (vm.stage) {
                    SendStage.picking => _Picking(vm: vm),
                    SendStage.preparing => _Centered(
                      logo: const OrbitLogo(size: 120, orbiting: true),
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
  const _Centered({required this.logo, required this.text});
  final Widget logo;
  final String text;

  @override
  Widget build(BuildContext context) => Center(
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        logo,
        Gap.l,
        Text(text, style: Theme.of(context).textTheme.titleMedium),
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
    final locale = Localizations.localeOf(context).languageCode;
    if (vm.files.isEmpty) {
      return Padding(
        padding: const EdgeInsets.all(32),
        child: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.upload_file_outlined, size: 64, color: theme.colorScheme.primary),
              Gap.m,
              Text(l.sendEmpty, style: theme.textTheme.headlineSmall, textAlign: TextAlign.center),
              Gap.s,
              Text(l.sendEmptyHint, textAlign: TextAlign.center, style: theme.textTheme.bodyLarge),
            ],
          ),
        ),
      );
    }
    return Column(
      children: [
        Expanded(
          child: ListView.separated(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 96),
            itemCount: vm.files.length,
            separatorBuilder: (_, _) => const SizedBox(height: 4),
            itemBuilder: (context, i) {
              final f = vm.files[i];
              return Card(
                color: theme.colorScheme.surfaceContainerLow,
                child: ListTile(
                  leading: const Icon(Icons.insert_drive_file_outlined),
                  title: Text(f.name, maxLines: 1, overflow: TextOverflow.ellipsis),
                  subtitle: Text(formatBytes(f.size, locale)),
                  trailing: IconButton(
                    tooltip: l.sendRemove,
                    icon: const Icon(Icons.close),
                    onPressed: () => vm.remove(f),
                  ),
                ),
              );
            },
          ),
        ),
        Material(
          color: theme.colorScheme.surfaceContainer,
          child: Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
            child: Column(
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
                  label: Text(
                    '${l.sendCreateCode} · ${l.itemsAndSize(vm.files.length, formatBytes(vm.totalSize, locale))}',
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
        const Center(child: OrbitLogo(size: 120, orbiting: true)),
        Gap.l,
        Card(
          color: theme.colorScheme.primaryContainer,
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Column(
              children: [
                Text(
                  l.sendYourCode,
                  style: theme.textTheme.labelLarge?.copyWith(
                    color: theme.colorScheme.onPrimaryContainer,
                  ),
                ),
                Gap.s,
                SelectableText(
                  code,
                  textAlign: TextAlign.center,
                  style: theme.textTheme.headlineLarge?.copyWith(
                    color: theme.colorScheme.onPrimaryContainer,
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
                            ScaffoldMessenger.of(context)
                                .showSnackBar(SnackBar(content: Text(l.sendCopied)));
                          }
                        },
                        icon: const Icon(Icons.copy),
                        label: Text(l.sendCopy),
                      ),
                    ),
                    Gap.s,
                    Expanded(
                      child: FilledButton.icon(
                        onPressed: () =>
                            SharePlus.instance.share(ShareParams(text: l.sendShareText(code))),
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
        Text(l.sendCodeHint, textAlign: TextAlign.center, style: theme.textTheme.bodyLarge),
        Gap.l,
        const LinearProgressIndicator(),
        Gap.s,
        Text(l.sendWaiting, textAlign: TextAlign.center),
        Gap.l,
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
        const Center(child: OrbitLogo(size: 140, orbiting: true, fast: true)),
        Gap.l,
        Text(
          l.sendSending,
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.headlineSmall,
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
    final dark = theme.brightness == Brightness.dark;
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Center(child: DoneBadge(dark: dark)),
          Gap.l,
          Text(l.sendDone, textAlign: TextAlign.center, style: theme.textTheme.headlineSmall),
          Gap.s,
          if (s != null) ...[
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
