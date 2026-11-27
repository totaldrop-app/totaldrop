#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
# SPDX-License-Identifier: GPL-3.0-or-later

"""Verify 16 KB ELF PT_LOAD alignment for Android 64-bit native libraries (#632).

Checks every 64-bit shared library (`arm64-v8a` and `x86_64`) inside an `.apk`
or `.aab` archive:
  - Every ELF64 `PT_LOAD` segment must have `p_align >= 16384`, be a power of
    two, and satisfy `p_offset % 16384 == p_vaddr % 16384`, as required for
    Android 15+ 16 KB page-size devices and Google Play.
  - With `--require-uncompressed` (used for `.apk` outputs), every native
    `.so` entry must also be stored uncompressed (`ZIP_STORED`), and every
    64-bit `.so` payload must start at a 16 KB-aligned file offset inside the
    APK archive.
"""

from __future__ import annotations

import argparse
import re
import struct
import sys
import zipfile

PAGE_SIZE_16K = 16384
ELF_MAGIC = b"\x7fELF"
ELFCLASS64 = 2
ELFDATA2LSB = 1
ELFDATA2MSB = 2
PT_LOAD = 1
ZIP_LFH_MAGIC = b"PK\x03\x04"
ABI_64_RE = re.compile(r"^(?:base/)?lib/(arm64-v8a|x86_64)/[^/]+\.so$")
ANY_SO_RE = re.compile(r"^(?:base/)?lib/[^/]+/[^/]+\.so$")


def _is_power_of_two(value: int) -> bool:
    return value > 0 and (value & (value - 1)) == 0


def check_elf64_load_alignment(data: bytes, name: str) -> list[str]:
    """Return a list of alignment errors for a 64-bit ELF binary."""
    if len(data) < 64 or data[:4] != ELF_MAGIC:
        return [f"{name}: not a valid ELF binary"]
    ei_class = data[4]
    if ei_class != ELFCLASS64:
        return [f"{name}: expected ELF64 (class 2), got class {ei_class}"]
    ei_data = data[5]
    if ei_data == ELFDATA2LSB:
        endian = "<"
    elif ei_data == ELFDATA2MSB:
        endian = ">"
    else:
        return [f"{name}: unknown ELF endianness {ei_data}"]

    # ELF64 header fields: e_phoff (offset 32, 8B), e_phentsize (54, 2B), e_phnum (56, 2B)
    (e_phoff,) = struct.unpack_from(f"{endian}Q", data, 32)
    e_phentsize, e_phnum = struct.unpack_from(f"{endian}HH", data, 54)
    if e_phentsize < 56 or e_phnum == 0:
        return [f"{name}: invalid program header table (phentsize={e_phentsize}, phnum={e_phnum})"]
    if e_phoff + e_phentsize * e_phnum > len(data):
        return [f"{name}: program header table exceeds file length"]

    errors: list[str] = []
    load_count = 0
    for i in range(e_phnum):
        off = e_phoff + i * e_phentsize
        # Elf64_Phdr: p_type (0, 4B), p_flags (4, 4B), p_offset (8, 8B),
        # p_vaddr (16, 8B), ..., p_align (48, 8B).
        (p_type,) = struct.unpack_from(f"{endian}I", data, off)
        if p_type != PT_LOAD:
            continue
        load_count += 1
        p_offset, p_vaddr = struct.unpack_from(f"{endian}QQ", data, off + 8)
        (p_align,) = struct.unpack_from(f"{endian}Q", data, off + 48)
        if p_align < PAGE_SIZE_16K or not _is_power_of_two(p_align):
            errors.append(
                f"{name}: PT_LOAD[{i}] p_align={p_align} (0x{p_align:x}) "
                f"must be a power of two >= {PAGE_SIZE_16K}"
            )
        if p_offset % PAGE_SIZE_16K != p_vaddr % PAGE_SIZE_16K:
            errors.append(
                f"{name}: PT_LOAD[{i}] p_offset=0x{p_offset:x} and "
                f"p_vaddr=0x{p_vaddr:x} are not congruent mod 16 KB"
            )
    if load_count == 0:
        errors.append(f"{name}: no PT_LOAD segments found")
    return errors


def _zip_data_offset(raw_file, info: zipfile.ZipInfo) -> int | None:
    """Compute exact payload byte offset from the ZIP Local File Header."""
    raw_file.seek(info.header_offset)
    lfh = raw_file.read(30)
    if len(lfh) != 30 or lfh[:4] != ZIP_LFH_MAGIC:
        return None
    fname_len, extra_len = struct.unpack_from("<HH", lfh, 26)
    return info.header_offset + 30 + fname_len + extra_len


def check_archive(path: str, *, require_uncompressed: bool = False) -> tuple[int, list[str]]:
    """Inspect an APK or AAB zip archive; returns (checked_64bit_libs, errors)."""
    errors: list[str] = []
    checked = 0
    with open(path, "rb") as raw_file, zipfile.ZipFile(raw_file, "r") as zf:
        for info in zf.infolist():
            name = info.filename
            is_64 = bool(ABI_64_RE.match(name))
            if require_uncompressed and ANY_SO_RE.match(name):
                if info.compress_type != zipfile.ZIP_STORED:
                    errors.append(
                        f"{name}: compressed (method={info.compress_type}), expected ZIP_STORED"
                    )
                elif is_64:
                    data_offset = _zip_data_offset(raw_file, info)
                    if data_offset is None:
                        errors.append(
                            f"{name}: invalid ZIP local file header at offset {info.header_offset}"
                        )
                    elif data_offset % PAGE_SIZE_16K != 0:
                        errors.append(
                            f"{name}: uncompressed payload offset {data_offset} "
                            f"(0x{data_offset:x}) is not 16 KB aligned"
                        )
            if is_64:
                checked += 1
                data = zf.read(info)
                errors.extend(check_elf64_load_alignment(data, name))
    if checked == 0:
        errors.append(f"{path}: no 64-bit native libraries (arm64-v8a / x86_64) found")
    return checked, errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", help="Path to the .apk or .aab file")
    parser.add_argument(
        "--require-uncompressed",
        action="store_true",
        help="Require all native .so entries to be stored uncompressed and 16 KB offset-aligned (APK check)",
    )
    args = parser.parse_args(argv)
    checked, errors = check_archive(
        args.archive,
        require_uncompressed=args.require_uncompressed,
    )
    if errors:
        for err in errors:
            print(f"FAIL: {err}", file=sys.stderr)
        return 1
    print(f"16 KB ELF alignment ok: checked {checked} 64-bit shared libraries in {args.archive}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
