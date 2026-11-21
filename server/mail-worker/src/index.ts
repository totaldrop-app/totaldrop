// SPDX-FileCopyrightText: 2026 The TotalDrop Contributors
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// TotalDrop mail Worker (#415): Email Routing sends hello@totaldrop.app
// here, and the Worker forwards each message to every maintainer. It has
// no HTTP route. Its log holds only the count of failed destinations
// (wrangler.jsonc); the logic is in forward.ts, where it is tested.

import { handle } from "./forward";

interface Env {
  /** The destinations, comma separated. A Worker secret set from the
   * `cloudflare` environment on deploy; never in the repository. Each
   * address must be a verified destination in Email Routing. */
  FORWARD_TO?: string;
}

export default {
  async email(message, env) {
    await handle(message, env.FORWARD_TO, (line) => console.error(line));
  },
} satisfies ExportedHandler<Env>;
