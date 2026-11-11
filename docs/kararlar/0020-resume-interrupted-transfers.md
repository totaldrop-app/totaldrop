# 0020: An interrupted transfer continues when the same files are sent again

- **Status:** Proposed, 11 November 2026 (#31)
- **Context:** When a connection is lost for good (QUIC's idle timeout,
  an app killed in the background, a laptop closed), everything starts
  again from the first byte. The protocol already has room for resuming,
  but nothing uses it:

  - `accept` carries, for every wanted entry, a bitmap of the chunks the
    receiver already holds (`crates/totaldrop-proto/src/wire.rs:784`). The
    receiver always sends it empty and the sender ignores it
    (`crates/totaldrop-core/src/transfer.rs:583`).
  - A data stream header names its first chunk
    (`crates/totaldrop-proto/src/wire.rs:1000`). The sender always sends 0
    (`crates/totaldrop-core/src/transfer.rs:425`) and the receiver refuses
    anything else (`crates/totaldrop-core/src/transfer.rs:972`).
  - A receiver removes every partial file when a transfer ends without it
    (`crates/totaldrop-core/src/transfer.rs:154`), whatever the reason.

  Short breaks are already survived inside one connection: a direct path
  that goes quiet moves to the relay and back (protocol notes, item 22,
  #185). This decision is about the connection that does not come back.

- **Options:**
  1. **Send the same files again with a new code.** The receiver kept
     what it had verified; it recognises the repeated offer and asks only
     for what is missing. No new message, key or handshake: a new
     transfer has a new content key, as today.
  2. **Reconnect on their own.** After a break, both devices meet again
     without a new code, for a few minutes, using a secret from the first
     session. Better for the person, but it is a new protocol step (a
     mailbox and key derived from the session, how long they live, who
     may use them) that needs a cryptographic review and changes the
     scope of the audit before 1.0 (#40). Android also has to keep the
     app alive while it waits (decision 0010).
  3. **Both, one after the other.** Option 1 now; option 2 later, on top
     of it, in its own issue. Option 2 needs everything option 1 builds
     (kept partial files, bitmaps, starting a stream mid-file).

- **Decision:** Option 3, with option 1 in this change:

  1. **What is kept.** A receiver keeps a partial file only when the
     connection is lost. When either side stops or cancels, when the
     receiver declines, and when a file's hash does not match, it removes
     partial files as today. Only whole chunks that were decrypted and
     authenticated count; a torn last chunk is cut off.
  2. **Where the receiver remembers it.** In a resume record in the
     application's own state folder, never in the destination: the
     destination, every top-level name created, and for every file entry
     either the placed path, or the partial file and how many chunks it
     holds. A record holds names and sizes but no key, code or address.
     The record is named after a keyed hash of the manifest's entries
     (path, kind, size, hash, mtime), not after the transfer ID, which is
     new every time.
  3. **When it continues.** Only when a new offer's entries are exactly
     those of a record (same paths, kinds, sizes, hashes and times) and
     go to the same destination. Then the receiver may write into the
     top-level folders it created itself in the interrupted attempt;
     everywhere else the no-merge, no-overwrite rule of protocol notes
     item 18 holds. Files it already placed are not wanted again. Partial
     files are wanted with a bitmap of the chunks they hold. A record is
     used once and removed when the transfer ends in any way.
  4. **Bitmaps.** A receiver only ever sets a prefix: chunks are written in
     order, so it holds chunks 0 to n−1. A sender starts the data stream
     at the first chunk whose bit is clear and sends every chunk from
     there to the end, even ones whose bit is set later. A bitmap with
     every bit set, longer than the file needs, or with a bit set beyond
     the last chunk is a protocol error.
  5. **Checking.** The receiver reads the chunks it holds back from disk
     into the file's BLAKE3 hash before it takes new ones, so the whole
     file is checked against the hash in the signed manifest as today. A
     mismatch removes the partial file; the next attempt starts that file
     from the beginning.
  6. **Age.** A record and its partial files are removed 24 hours after
     the interruption, at the next start of a receive. Until then the
     partial files stay hidden in the destination under their
     `.totaldrop-….part` names.
  7. **Older devices.** An older receiver never sets a bit, so nothing
     changes for it. An older sender ignores bitmaps and starts at chunk
     0; the receiver then empties the partial file and takes the whole
     file again.

  Option 2 gets its own issue once this is merged.

- **Why:** Option 1 changes no cryptography: every transfer still has a
  fresh content key (`crates/totaldrop-crypto/src/lib.rs:109`), so a
  chunk index is never used twice under one key, whatever the sender's
  file looks like the second time. The whole-file hash in the signed
  manifest still decides whether a file is kept, so held chunks cannot
  smuggle in other content. It works with every sender, the command line
  and both apps, and it is a step option 2 needs anyway.

- **Consequences:**
  - The threat model records two new facts: partial files of an
    interrupted transfer stay on disk for up to 24 hours, and a repeated
    offer tells the sender how much of the same files this receiver holds
    from before, which links the two attempts. Someone who can make an
    offer with exactly the same entries must hold the same files (their
    hashes are in it).
  - The core library needs a state folder from each platform: the command
    line uses `$XDG_STATE_HOME/totaldrop` (or the platform's equivalent),
    the apps their private storage.
  - The resume record is read from disk, so its decoder is bounded and
    gets a fuzz target.
  - The apps say that a transfer continues where it stopped, in every
    language, in a later change; this one covers the core library and the
    command line.
