// A failed transfer says why in the person's language; the technical text
// under Details stays English for problem reports, and says so (#212).
import 'package:material_ui/material_ui.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/l10n/app_localizations.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/src/rust/frb_generated.dart';
import 'package:totaldrop/ui/core/common.dart';

Future<void> _show(WidgetTester tester, Locale locale, TransferError e) async {
  await tester.pumpWidget(
    MaterialApp(
      locale: locale,
      localizationsDelegates: const [
        AppLocalizations.delegate,
        ...GlobalMaterialLocalizations.delegates,
      ],
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(
        body: SingleChildScrollView(
          child: ErrorPanel(error: e, onRetry: () {}),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

/// Stands in for the Rust library: the panel only asks it to make hidden
/// characters visible, which plain test text has none of.
class _Rust implements RustLibApi {
  @override
  String crateApiL10NRevealControls({required String name}) => name;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  setUpAll(() => RustLib.initMock(api: _Rust()));

  const timedOut = TransferError(
    kind: ErrorKind.connect,
    detail: 'could not connect to the other device: timed out',
    cause: 'TimedOut',
  );

  testWidgets('the cause is shown in the language of the device', (
    tester,
  ) async {
    await _show(tester, const Locale('tr'), timedOut);
    final l = await AppLocalizations.delegate.load(const Locale('tr'));
    await tester.tap(find.text(l.moreInfo));
    await tester.pumpAndSettle();
    expect(find.text(l.causeTimedOut), findsOneWidget);
    expect(find.text(l.technicalDetail), findsOneWidget);
    expect(l.causeTimedOut, isNot(contains('timed out')));
  });

  testWidgets('an error without a cause shows only the technical text', (
    tester,
  ) async {
    await _show(
      tester,
      const Locale('de'),
      const TransferError(kind: ErrorKind.other, detail: 'x', cause: ''),
    );
    final l = await AppLocalizations.delegate.load(const Locale('de'));
    await tester.tap(find.text(l.moreInfo));
    await tester.pumpAndSettle();
    expect(find.text(l.technicalDetail), findsOneWidget);
    expect(find.text(l.causeTimedOut), findsNothing);
  });

  test('every cause the core names has a text in every language', () async {
    // The names of Cause::ALL in the core; the core's own tests check that
    // each has an English text.
    const causes = [
      'TimedOut',
      'ConnectionLost',
      'NoRoute',
      'Relay',
      'Protocol',
      'DiskFull',
      'NoPermission',
      'NotFound',
      'ReadOnly',
      'FileChanged',
    ];
    final en = await AppLocalizations.delegate.load(const Locale('en'));
    for (final locale in AppLocalizations.supportedLocales) {
      final l = await AppLocalizations.delegate.load(locale);
      for (final c in causes) {
        final text = causeText(l, c);
        expect(text, isNotEmpty, reason: '$locale $c');
        if (locale.languageCode != 'en') {
          expect(
            text,
            isNot(causeText(en, c)),
            reason: '$locale $c untranslated',
          );
        }
      }
    }
  });
}
