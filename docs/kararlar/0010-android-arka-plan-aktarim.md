# 0010: On Android, a transfer continues in the background as a user-initiated job

- **Status:** Accepted, 3 November 2026
- **Context:** Android cuts the network connection of an app that is not on
  screen. When the sender left TotalDrop to share the code in another app,
  the connection dropped ("Software caused connection abort"). The
  rendezvous server then closed the mailbox and the code became invalid
  (#194). Yet that is exactly the normal way to share the code.
- **Options:**
  1. A foreground service of type `dataSync`. Works everywhere, but needs a
     foreground service declaration and a demo video in the Play Console.
     On Android 15 and later it is also limited to 6 hours a day.
  2. A user-initiated data transfer job
     (`JobInfo.Builder.setUserInitiated`, Android 14 and later). This is
     the route Android recommends for transfers that a person starts; it
     needs no Play declaration, and a notification is mandatory.
  3. Reconnecting when the connection drops. Useless while there is no
     network in the background.
- **Decision:** Option 2. When sending or receiving starts, the app
  schedules a job. While the job runs, a "Sending/Receiving with TotalDrop"
  notification is shown, with a **Stop** button. However the transfer ends,
  the job ends too. The transfer itself still runs in the Rust core; the
  job only keeps the process and its network connection alive. Since the
  minimum version is already Android 15, no other route is needed.
- **Why:** This is the right tool for a transfer that a person starts. It
  needs no extra Play declaration, and the code stays valid for as long as
  it is being shared. Security does not change: the code is still single
  use, is deleted from the server after 15 minutes if unused, and the
  verification words are compared.
- **Consequences:**
  - New permissions: `RUN_USER_INITIATED_JOBS`, `POST_NOTIFICATIONS` (asked
    on the first transfer; if refused, the transfer still continues, only
    without a notification) and `ACCESS_NETWORK_STATE`.
  - The notification texts are in `l10n/app/`, in 35 languages.
  - If the system stops the job, or the person stops it from the task
    manager, the transfer stops too.
  - The iOS counterpart will be dealt with separately.
