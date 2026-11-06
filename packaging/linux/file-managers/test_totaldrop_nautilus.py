# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Tests for the Nautilus extension, without Nautilus: `gi` is replaced by
# a stand-in that records what the extension asks of it.
#
#   python3 -I packaging/linux/file-managers/test_totaldrop_nautilus.py

import importlib.util
import pathlib
import sys
import types
import unittest

HERE = pathlib.Path(__file__).resolve().parent
# Leave no __pycache__ next to the installed sources.
sys.dont_write_bytecode = True


class GLibError(Exception):
    pass


class FakeAppInfo:
    launched = []
    fail = False

    @classmethod
    def new(cls, desktop_id):
        cls.desktop_id = desktop_id
        return None if desktop_id == "missing.desktop" else cls()

    def launch_uris(self, uris, context):
        if FakeAppInfo.fail:
            raise GLibError("no")
        FakeAppInfo.launched.append((list(uris), context))
        return True


class FakeMenuItem:
    def __init__(self, name, label):
        self.name = name
        self.label = label
        self.handlers = []

    def connect(self, signal, handler):
        assert signal == "activate"
        self.handlers.append(handler)

    def activate(self):
        for h in self.handlers:
            h(self)


def load(language_names=("C",)):
    gi = types.ModuleType("gi")
    versions = []

    def require_version(name, version):
        if name == "Nautilus" and version == "4.1":
            raise ValueError("Namespace Nautilus not available for version 4.1")
        if name == "Gdk":
            raise ValueError("no display in tests")
        versions.append((name, version))

    gi.require_version = require_version
    repo = types.ModuleType("gi.repository")
    repo.Gio = types.SimpleNamespace(DesktopAppInfo=FakeAppInfo)
    repo.GLib = types.SimpleNamespace(Error=GLibError, get_language_names=lambda: list(language_names))
    repo.GObject = types.SimpleNamespace(GObject=type("GObject", (), {}))
    repo.Nautilus = types.SimpleNamespace(MenuProvider=type("MenuProvider", (), {}), MenuItem=FakeMenuItem)
    gi.repository = repo
    sys.modules["gi"] = gi
    sys.modules["gi.repository"] = repo
    spec = importlib.util.spec_from_file_location("totaldrop_nautilus", HERE / "totaldrop_nautilus.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module._versions = versions
    return module


class FakeFile:
    """A NautilusFileInfo: `uri` is where it is shown, `activation` the
    file it stands for (a file:// URI for items in Recent)."""

    def __init__(self, uri, activation=None):
        self.uri = uri
        self.activation = activation or uri

    def get_uri_scheme(self):
        return self.uri.split(":", 1)[0]

    def get_uri(self):
        return self.uri

    def get_activation_uri(self):
        return self.activation


class Extension(unittest.TestCase):
    def setUp(self):
        FakeAppInfo.launched = []
        FakeAppInfo.fail = False

    def test_falls_back_to_api_4_0(self):
        self.assertIn(("Nautilus", "4.0"), load()._versions)

    def test_label_follows_the_language(self):
        m = load()
        self.assertEqual(m.label(["tr_TR.UTF-8", "tr_TR", "tr.UTF-8", "tr", "C"]), "TotalDrop ile gönder")
        self.assertEqual(m.label(["pt_BR.UTF-8", "pt_BR", "pt", "C"]), "Enviar com o TotalDrop")
        self.assertEqual(m.label(["zh_TW.UTF-8", "zh_TW", "zh", "C"]), "用 TotalDrop 傳送")
        self.assertEqual(m.label(["zh_CN", "zh", "C"]), "用 TotalDrop 发送")
        self.assertEqual(m.label(["xx_YY", "xx", "C"]), "Send with TotalDrop")
        self.assertEqual(m.label([]), "Send with TotalDrop")

    def test_every_language_has_a_label(self):
        m = load()
        self.assertGreaterEqual(len(m.LABELS), 35)
        for loc, text in m.LABELS.items():
            self.assertIn("TotalDrop", text, loc)

    def test_local_files_and_folders_are_offered_as_uris(self):
        m = load(["tr", "C"])
        # A leading "-", spaces, quotes and non-ASCII stay inside the URI.
        uris = [
            "file:///home/a/-rf%20%22x%22.txt",
            "file:///home/a/Belgeler/%C3%A7i%C3%A7ek",
        ]
        items = m.TotalDropMenu().get_file_items([FakeFile(u) for u in uris])
        self.assertEqual(len(items), 1)
        self.assertEqual(items[0].label, "TotalDrop ile gönder")
        items[0].activate()
        self.assertEqual(FakeAppInfo.desktop_id, "app.totaldrop.TotalDrop.desktop")
        self.assertEqual(FakeAppInfo.launched, [(uris, None)])

    def test_recent_items_send_the_file_they_stand_for(self):
        m = load()
        items = m.TotalDropMenu().get_file_items([FakeFile("recent:///abc", "file:///home/a/x.pdf")])
        self.assertEqual(len(items), 1)
        items[0].activate()
        self.assertEqual(FakeAppInfo.launched, [(["file:///home/a/x.pdf"], None)])

    def test_old_api_passes_the_window_first(self):
        m = load()
        items = m.TotalDropMenu().get_file_items(object(), [FakeFile("file:///a")])
        self.assertEqual(len(items), 1)

    def test_nothing_is_offered_for_remote_or_empty_selections(self):
        m = load()
        menu = m.TotalDropMenu()
        self.assertEqual(menu.get_file_items([]), [])
        self.assertEqual(menu.get_file_items([FakeFile("file:///a"), FakeFile("sftp://host/b")]), [])
        self.assertEqual(menu.get_file_items([FakeFile("trash:///c")]), [])

    def test_launch_failures_are_reported_not_raised(self):
        m = load()
        FakeAppInfo.fail = True
        self.assertFalse(m.launch(["file:///a"]))
        FakeAppInfo.fail = False
        m.APP_ID = "missing"
        self.assertFalse(m.launch(["file:///a"]))


if __name__ == "__main__":
    unittest.main()
