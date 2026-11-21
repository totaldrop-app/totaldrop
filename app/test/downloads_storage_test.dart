// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Public Downloads resolution across Android users / Work Profiles and
// MediaStore indexing when a file receive finishes (#592).
import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:totaldrop/data/downloads_storage.dart';
import 'package:totaldrop/data/settings_repository.dart';
import 'package:totaldrop/data/transfer_repository.dart';
import 'package:totaldrop/src/rust/api/transfer.dart';
import 'package:totaldrop/ui/receive/receive_view.dart';
import 'package:totaldrop/ui/receive/receive_view_model.dart';

import 'fakes.dart';

class _Settings extends ChangeNotifier implements SettingsRepository {
  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _FakeStorage implements DownloadsStorage {
  _FakeStorage(this.dir);

  final String? dir;
  final scanned = <String>[];

  @override
  Future<String?> downloadsDirectory() async => dir;

  @override
  Future<void> scanFiles(List<String> paths) async => scanned.addAll(paths);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'displayReceiveFolder strips /storage/emulated/<userId>/ for any profile',
    () {
      expect(
        displayReceiveFolder('/storage/emulated/0/Download/TotalDrop'),
        'Download/TotalDrop',
      );
      expect(
        displayReceiveFolder('/storage/emulated/10/Download/TotalDrop'),
        'Download/TotalDrop',
      );
      expect(
        displayReceiveFolder('/storage/emulated/999/Download/TotalDrop'),
        'Download/TotalDrop',
      );
      expect(
        displayReceiveFolder(
          '/storage/1234-5678/Android/data/app.totaldrop/files/TotalDrop',
        ),
        '/storage/1234-5678/Android/data/app.totaldrop/files/TotalDrop',
      );
    },
  );

  test(
    'TransferRepository.receiveFolder uses the active profile public Downloads',
    () async {
      final temp = await Directory.systemTemp.createTemp('td-work-profile-');
      addTearDown(() => temp.delete(recursive: true));

      final workDownloads = '${temp.path}/storage/emulated/10/Download';
      final repo = TransferRepository(
        _Settings(),
        storage: _FakeStorage(workDownloads),
        isAndroid: true,
      );

      final folder = await repo.receiveFolder();
      expect(folder, '$workDownloads/TotalDrop');
      expect(Directory(folder).existsSync(), isTrue);
      expect(File('$folder/.totaldrop-write-test').existsSync(), isFalse);
    },
  );

  test('TransferRepository.receiveFolder falls back to app external folder when unwritable', () async {
    final temp = await Directory.systemTemp.createTemp('td-app-fallback-');
    addTearDown(() => temp.delete(recursive: true));

    // Create a regular file where the directory would go so mkdir fails.
    final blocked = File('${temp.path}/blocked-downloads')
      ..writeAsStringSync('x');
    final appOwn = Directory('${temp.path}/app-external')..createSync();

    final repo = TransferRepository(
      _Settings(),
      storage: _FakeStorage(blocked.path),
      isAndroid: true,
      externalStorageDirectory: () async => appOwn,
    );

    expect(await repo.receiveFolder(), '${appOwn.path}/TotalDrop');
  });

  test(
    'AndroidDownloadsStorage talks over app.totaldrop/storage channel',
    () async {
      const channel = MethodChannel('app.totaldrop/storage');
      final calls = <MethodCall>[];
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
          .setMockMethodCallHandler(channel, (call) async {
            calls.add(call);
            if (call.method == 'downloadsDir') {
              return '/storage/emulated/10/Download';
            }
            return null;
          });
      addTearDown(
        () => TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
            .setMockMethodCallHandler(channel, null),
      );

      const storage = AndroidDownloadsStorage();
      expect(
        await storage.downloadsDirectory(),
        '/storage/emulated/10/Download',
      );
      await storage.scanFiles(const []);
      await storage.scanFiles(const [
        '/storage/emulated/10/Download/TotalDrop/a.jpg',
      ]);

      expect(calls.map((c) => c.method), ['downloadsDir', 'scanFiles']);
      expect(calls.last.arguments, [
        '/storage/emulated/10/Download/TotalDrop/a.jpg',
      ]);
    },
  );

  test(
    'ReceiveViewModel indexes saved files on completion, never received text',
    () async {
      final fileRepo = FakeRepository();
      final fileVm = ReceiveViewModel(repository: fileRepo)
        ..setInput('7-harbor-rocket');
      unawaited(fileVm.connect());
      fileRepo.connected.complete(FakeOffer(isText: false));
      await pumpEventQueue();
      await fileVm.accept();
      fileRepo.events.add(
        const TransferEvent(
          kind: EventKind.finished,
          words: [],
          done: 10,
          total: 10,
          summary: TransferSummary(
            filesOk: 2,
            filesFailed: 0,
            bytes: 10,
            saved: [
              '/storage/emulated/10/Download/TotalDrop/one.jpg',
              '/storage/emulated/10/Download/TotalDrop/two.pdf',
            ],
            text: null,
          ),
          error: null,
        ),
      );
      await pumpEventQueue();
      expect(fileVm.stage, ReceiveStage.done);
      expect(fileRepo.scannedFiles, [
        '/storage/emulated/10/Download/TotalDrop/one.jpg',
        '/storage/emulated/10/Download/TotalDrop/two.pdf',
      ]);

      final textRepo = FakeRepository();
      final textVm = ReceiveViewModel(repository: textRepo)
        ..setInput('7-harbor-rocket');
      unawaited(textVm.connect());
      textRepo.connected.complete(FakeOffer(isText: true));
      await pumpEventQueue();
      await textVm.accept();
      textRepo.events.add(
        const TransferEvent(
          kind: EventKind.finished,
          words: [],
          done: 0,
          total: 0,
          summary: TransferSummary(
            filesOk: 1,
            filesFailed: 0,
            bytes: 5,
            saved: [],
            text: 'hello',
          ),
          error: null,
        ),
      );
      await pumpEventQueue();
      expect(textVm.stage, ReceiveStage.done);
      expect(textRepo.scannedFiles, isEmpty);
    },
  );
}
