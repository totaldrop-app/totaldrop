// node --test site/tools/test-favicons.mjs
import test from "node:test";
import assert from "node:assert/strict";
import { buildIco, icoSizes, pngSize } from "./favicons.mjs";

// The smallest PNG header that pngSize reads: signature and IHDR.
function fakePng(side) {
  const b = Buffer.alloc(33);
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]).copy(b);
  b.writeUInt32BE(13, 8);
  b.write("IHDR", 12, "latin1");
  b.writeUInt32BE(side, 16);
  b.writeUInt32BE(side, 20);
  return b;
}

test("an ICO built from PNGs reads back their sizes", () => {
  assert.deepEqual(icoSizes(buildIco([fakePng(16), fakePng(32), fakePng(48)])), [16, 32, 48]);
});

test("256 px is stored as 0 and read back as 256", () => {
  const ico = buildIco([fakePng(256)]);
  assert.equal(ico[6], 0);
  assert.deepEqual(icoSizes(ico), [256]);
});

test("a non-PNG, a non-square image or a broken ICO is refused", () => {
  assert.equal(pngSize(Buffer.from("GIF89a and more bytes here....")), null);
  const wide = fakePng(32);
  wide.writeUInt32BE(16, 20);
  assert.throws(() => buildIco([wide]), /square/);
  assert.throws(() => buildIco([fakePng(0)]), /square/);
  assert.throws(() => icoSizes(Buffer.from([0, 0, 2, 0, 1, 0])), /not an ICO/);
  const ico = buildIco([fakePng(16)]);
  assert.throws(() => icoSizes(ico.subarray(0, ico.length - 1)), /outside/);
  const lying = Buffer.from(ico);
  lying[6] = 32;
  assert.throws(() => icoSizes(lying), /directory says/);
});
