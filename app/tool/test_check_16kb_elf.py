#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

"""Unit tests for check_16kb_elf.py (#632)."""

from __future__ import annotations

import io
import os
import struct
import tempfile
import unittest
import zipfile
from contextlib import redirect_stderr, redirect_stdout

from check_16kb_elf import (
    ELF_MAGIC,
    ELFCLASS64,
    ELFDATA2LSB,
    PAGE_SIZE_16K,
    PT_LOAD,
    check_archive,
    check_elf64_load_alignment,
    main,
)

PT_GNU_STACK = 0x6474E551


def make_elf64(
    segments: list[tuple[int, int, int, int]],
    *,
    ei_class: int = ELFCLASS64,
) -> bytes:
    """Build a minimal little-endian ELF binary from (p_type, p_offset, p_vaddr, p_align) tuples."""
    e_phoff = 64
    e_phentsize = 56
    e_phnum = len(segments)
    ehdr = bytearray(64)
    ehdr[0:4] = ELF_MAGIC
    ehdr[4] = ei_class
    ehdr[5] = ELFDATA2LSB
    ehdr[6] = 1  # EV_CURRENT
    struct.pack_into("<Q", ehdr, 32, e_phoff)
    struct.pack_into("<HH", ehdr, 54, e_phentsize, e_phnum)

    phdrs = bytearray()
    for p_type, p_offset, p_vaddr, p_align in segments:
        phdr = bytearray(56)
        struct.pack_into("<I", phdr, 0, p_type)
        struct.pack_into("<Q", phdr, 8, p_offset)
        struct.pack_into("<Q", phdr, 16, p_vaddr)
        struct.pack_into("<Q", phdr, 48, p_align)
        phdrs.extend(phdr)
    return bytes(ehdr + phdrs)


def write_aligned_stored_entry(
    zf: zipfile.ZipFile,
    arcname: str,
    payload: bytes,
    *,
    align: int = PAGE_SIZE_16K,
) -> None:
    """Write a ZIP_STORED entry padded via local extra bytes so its payload starts at an `align` boundary."""
    current_pos = zf.fp.tell()
    base_data_offset = current_pos + 30 + len(arcname.encode("utf-8"))
    pad = (-base_data_offset) % align
    info = zipfile.ZipInfo(arcname)
    info.compress_type = zipfile.ZIP_STORED
    info.extra = b"\x00" * pad
    zf.writestr(info, payload)


class Check16KbElfTest(unittest.TestCase):
    def test_accepts_16kb_and_64kb_aligned_segments_and_skips_non_load(self) -> None:
        elf = make_elf64(
            [
                (PT_LOAD, 0x0, 0x0, PAGE_SIZE_16K),
                (PT_LOAD, 0x1234, 0x11234, 65536),
                (PT_GNU_STACK, 0, 0, 16),
            ]
        )
        self.assertEqual(check_elf64_load_alignment(elf, "libtest.so"), [])

    def test_rejects_4kb_aligned_segment(self) -> None:
        elf = make_elf64(
            [
                (PT_LOAD, 0, 0, PAGE_SIZE_16K),
                (PT_LOAD, 0, 0, 4096),
            ]
        )
        errors = check_elf64_load_alignment(elf, "libbad.so")
        self.assertEqual(len(errors), 1)
        self.assertIn("p_align=4096", errors[0])

    def test_rejects_non_power_of_two_p_align(self) -> None:
        elf = make_elf64([(PT_LOAD, 0, 0, 3 * PAGE_SIZE_16K)])
        errors = check_elf64_load_alignment(elf, "libnonpow2.so")
        self.assertEqual(len(errors), 1)
        self.assertIn("power of two", errors[0])

    def test_rejects_offset_vaddr_mod_16kb_mismatch(self) -> None:
        elf = make_elf64([(PT_LOAD, 0x1000, 0x2000, PAGE_SIZE_16K)])
        errors = check_elf64_load_alignment(elf, "libcongruence.so")
        self.assertEqual(len(errors), 1)
        self.assertIn("not congruent mod 16 KB", errors[0])

    def test_rejects_non_elf_elf32_empty_phdr_and_missing_pt_load(self) -> None:
        self.assertIn("not a valid ELF", check_elf64_load_alignment(b"not-an-elf", "x.so")[0])

        elf32 = make_elf64([(PT_LOAD, 0, 0, PAGE_SIZE_16K)], ei_class=1)
        self.assertIn("expected ELF64", check_elf64_load_alignment(elf32, "32.so")[0])

        zero_phnum = make_elf64([])
        self.assertIn("invalid program header table", check_elf64_load_alignment(zero_phnum, "z.so")[0])

        no_load = make_elf64([(PT_GNU_STACK, 0, 0, 16)])
        self.assertIn("no PT_LOAD", check_elf64_load_alignment(no_load, "noload.so")[0])

    def test_archive_checks_aab_and_apk_compression_and_offset(self) -> None:
        good_elf = make_elf64([(PT_LOAD, 0, 0, PAGE_SIZE_16K)])
        with tempfile.TemporaryDirectory() as tmp:
            aab = os.path.join(tmp, "app.aab")
            with zipfile.ZipFile(aab, "w", compression=zipfile.ZIP_DEFLATED) as zf:
                zf.writestr("base/lib/arm64-v8a/librust.so", good_elf)
                zf.writestr("base/lib/x86_64/librust.so", good_elf)
            checked, errors = check_archive(aab, require_uncompressed=False)
            self.assertEqual(checked, 2)
            self.assertEqual(errors, [])

            apk_compressed = os.path.join(tmp, "compressed.apk")
            with zipfile.ZipFile(apk_compressed, "w", compression=zipfile.ZIP_DEFLATED) as zf:
                zf.writestr("lib/arm64-v8a/librust.so", good_elf)
            _, errors = check_archive(apk_compressed, require_uncompressed=True)
            self.assertTrue(any("compressed" in e for e in errors))

            apk_unaligned = os.path.join(tmp, "unaligned.apk")
            with zipfile.ZipFile(apk_unaligned, "w", compression=zipfile.ZIP_STORED) as zf:
                zf.writestr("lib/arm64-v8a/librust.so", good_elf)
            _, errors = check_archive(apk_unaligned, require_uncompressed=True)
            self.assertTrue(any("uncompressed payload offset" in e for e in errors))

            apk_aligned = os.path.join(tmp, "aligned.apk")
            with zipfile.ZipFile(apk_aligned, "w", compression=zipfile.ZIP_STORED) as zf:
                write_aligned_stored_entry(zf, "lib/arm64-v8a/librust.so", good_elf)
            checked, errors = check_archive(apk_aligned, require_uncompressed=True)
            self.assertEqual(checked, 1)
            self.assertEqual(errors, [])

            empty_aab = os.path.join(tmp, "empty.aab")
            with zipfile.ZipFile(empty_aab, "w") as zf:
                zf.writestr("base/manifest/AndroidManifest.xml", b"x")
            checked, errors = check_archive(empty_aab, require_uncompressed=False)
            self.assertEqual(checked, 0)
            self.assertTrue(any("no 64-bit native libraries" in e for e in errors))

            out = io.StringIO()
            err = io.StringIO()
            with redirect_stdout(out), redirect_stderr(err):
                rc_ok = main(["--require-uncompressed", apk_aligned])
                rc_fail = main(["--require-uncompressed", apk_unaligned])
            self.assertEqual(rc_ok, 0)
            self.assertEqual(rc_fail, 1)
            self.assertIn("16 KB ELF alignment ok", out.getvalue())
            self.assertIn("FAIL:", err.getvalue())


if __name__ == "__main__":
    unittest.main()
