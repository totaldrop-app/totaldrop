import 'package:material_ui/material_ui.dart';
import 'package:flutter/services.dart';

import '../../l10n/app_localizations.dart';
import '../core/common.dart';
import '../core/l10n.dart';
import '../core/mark.dart';
import '../core/theme.dart';
import 'receive_view_model.dart';

class ReceiveView extends StatefulWidget {
  const ReceiveView({super.key, required this.viewModel});

  final ReceiveViewModel viewModel;

  @override
  State<ReceiveView> createState() => _ReceiveViewState();
}

class _ReceiveViewState extends State<ReceiveView> {
  late final _controller = TextEditingController(text: widget.viewModel.input);

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<bool> _confirmStop(BuildContext context) async {
    final l = AppLocalizations.of(context);
    return await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            title: Text(l.cancelTransferTitle),
            content: Text(l.cancelTransferBody),
            actions: [
              TextButton(onPressed: () => Navigator.pop(context, false), child: Text(l.keepGoing)),
              FilledButton(onPressed: () => Navigator.pop(context, true), child: Text(l.stop)),
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
                    ReceiveStage.entering => _Entering(vm: vm, controller: _controller),
                    ReceiveStage.connecting => Center(
                      child: Column(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          const TotalDropMark(size: 120, sending: true),
                          Gap.l,
                          Text(l.receiveConnecting, style: Theme.of(context).textTheme.titleMedium),
                        ],
                      ),
                    ),
                    ReceiveStage.offered => _Offered(vm: vm),
                    ReceiveStage.transferring => _Transferring(vm: vm),
                    ReceiveStage.done => _Done(vm: vm),
                    ReceiveStage.declined => _Message(
                      icon: Icons.block,
                      text: l.receiveDeclined,
                      onDone: () {
                        vm.reset();
                        Navigator.pop(context);
                      },
                    ),
                    ReceiveStage.failed => Padding(
                      padding: const EdgeInsets.all(24),
                      child: Center(
                        child: ErrorPanel(
                          error: vm.error!,
                          onRetry: () {
                            vm.reset();
                            _controller.clear();
                          },
                        ),
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

class _Entering extends StatelessWidget {
  const _Entering({required this.vm, required this.controller});
  final ReceiveViewModel vm;
  final TextEditingController controller;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        const Center(child: TotalDropMark(size: 96)),
        Gap.l,
        TextField(
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
            errorMaxLines: 3,
            prefixIcon: const Icon(Icons.key_outlined),
            suffixIcon: IconButton(
              tooltip: l.receivePaste,
              icon: const Icon(Icons.content_paste),
              onPressed: () async {
                final data = await Clipboard.getData(Clipboard.kTextPlain);
                final text = data?.text?.trim();
                if (text == null || text.isEmpty) return;
                // A shared message holds more than the code: take its last word group.
                final match = RegExp(r'\d+(?:[-\s][a-zA-Z]+){2,3}').firstMatch(text);
                final value = match?.group(0) ?? text;
                controller.text = value;
                vm.setInput(value);
              },
            ),
          ),
          onChanged: vm.setInput,
          onSubmitted: (_) => vm.connect(),
        ),
        Gap.l,
        FilledButton.icon(
          onPressed: vm.input.trim().isEmpty ? null : vm.connect,
          icon: const Icon(Icons.link),
          label: Text(l.receiveConnect),
        ),
        Gap.l,
        const SameNetworkNote(),
      ],
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
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        Text(l.receiveOffer, style: theme.textTheme.headlineSmall),
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
                  child: Text(revealControls(n), maxLines: 1, overflow: TextOverflow.ellipsis),
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
        Gap.l,
        VerificationWords(words: o.words),
        Gap.l,
        Row(
          children: [
            Expanded(
              child: OutlinedButton(
                onPressed: vm.decline,
                child: Text(safeL10n(context, 'receiveDecline').receiveDecline),
              ),
            ),
            Gap.m,
            Expanded(
              child: FilledButton(
                onPressed: vm.accept,
                child: Text(safeL10n(context, 'receiveAccept').receiveAccept),
              ),
            ),
          ],
        ),
      ],
    );
  }
}

class _Transferring extends StatelessWidget {
  const _Transferring({required this.vm});
  final ReceiveViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        const Center(child: TotalDropMark(size: 140, sending: true)),
        Gap.l,
        Text(
          l.receiveReceiving,
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        Gap.m,
        TransferProgress(done: vm.done, total: vm.total),
        Gap.l,
        if (vm.offer != null) VerificationWords(words: vm.offer!.words),
      ],
    );
  }
}

class _Done extends StatelessWidget {
  const _Done({required this.vm});
  final ReceiveViewModel vm;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final theme = Theme.of(context);
    final s = vm.summary;
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        Center(child: DoneBadge(dark: theme.brightness == Brightness.dark)),
        Gap.l,
        Text(l.receiveDone, textAlign: TextAlign.center, style: theme.textTheme.headlineSmall),
        Gap.s,
        if (vm.folder != null)
          Text(
            l.receiveSaveTo(vm.folder!.replaceFirst('/storage/emulated/0/', '')),
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
        Gap.m,
        if (s != null)
          for (final p in s.saved)
            ListTile(
              leading: Icon(Icons.check, color: theme.colorScheme.onSurfaceVariant),
              title: Text(
                revealControls(p.split('/').last),
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
        Gap.l,
        FilledButton(
          onPressed: () {
            vm.reset();
            Navigator.pop(context);
          },
          child: Text(l.done),
        ),
      ],
    );
  }
}

class _Message extends StatelessWidget {
  const _Message({required this.icon, required this.text, required this.onDone});
  final IconData icon;
  final String text;
  final VoidCallback onDone;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Icon(icon, size: 56, color: Theme.of(context).colorScheme.onSurfaceVariant),
          Gap.m,
          Text(text, textAlign: TextAlign.center, style: Theme.of(context).textTheme.titleMedium),
          Gap.l,
          FilledButton(onPressed: onDone, child: Text(AppLocalizations.of(context).done)),
        ],
      ),
    );
  }
}
