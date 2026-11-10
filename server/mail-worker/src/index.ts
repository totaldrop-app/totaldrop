// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// TotalDrop mail Worker (#415): Email Routing sends hello@totaldrop.app
// here, and the Worker forwards each message to every maintainer. It has
// no HTTP route and logs nothing about the mail.

import { forwardAll, recipients } from "./forward";

interface Env {
  /** The destinations, comma separated. A Worker secret set from the
   * `cloudflare` environment on deploy; never in the repository. Each
   * address must be a verified destination in Email Routing. */
  FORWARD_TO?: string;
}

export default {
  async email(message, env) {
    // Read on every message, so a changed secret applies at once, and a
    // missing one fails this message rather than the deploy.
    const failed = await forwardAll(message, recipients(env.FORWARD_TO));
    // A count only: no address, sender or subject.
    if (failed > 0) console.error(`forwarding failed for ${failed} destination(s)`);
  },
} satisfies ExportedHandler<Env>;
