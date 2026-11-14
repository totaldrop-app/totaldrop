# 0004: An account is only a random number

- **Status:** Superseded by [0013](0013-hayalet-halka-hesap.md) (4 November
  2026). First accepted: 27 October 2026.
- **Context:** "My devices" and saved contacts need an account, but no
  personal information may be collected.
- **Decision:** Mullvad style: an account is a random 16-digit number; there
  is no email, phone number, user name or password. The account is
  optional. A device is added to the account once; a device added with the
  number is announced to the other devices and, for 24 hours, does not
  receive what is sent to the owner's own devices.
- **Why:** Data that is not kept cannot leak. If the number is stolen, the
  announcement and the waiting period guard against a device being added
  silently.
- **Consequences:** If the number and every device are lost, the account
  cannot be recovered; this is a deliberate choice. The details of this
  decision were in the version of `docs/hesap.md` before 4 November 2026;
  the new design is in [hesap.md](../hesap.md).
