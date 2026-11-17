# Google Play Console Automated Upload (`play-release`)

Signed Android App Bundles (`TotalDrop-<version>.aab`) are built exclusively in
[`.github/workflows/android-release.yml`](../.github/workflows/android-release.yml)
inside the protected `play-release` GitHub Actions environment from an
annotated `android-v<semver>` tag on `main` ([`surec.md`](surec.md)).

When `GOOGLE_PLAY_SERVICE_ACCOUNT_JSON` is configured in `play-release`,
[`tools/play-upload.mjs`](../tools/play-upload.mjs) automatically uploads the
verified `.aab` to Google Play Console through the Google Play Android
Developer API v3 (`androidpublisher`) with zero third-party npm packages or
actions.

## Security & Release Invariants

1. **Key isolation:** `Upload to Google Play` runs only **after**
   `$RUNNER_TEMP/upload.jks` has been removed, Sigstore build provenance has
   been attested, and the `play-bundle` workflow artifact has been uploaded.
   `ANDROID_UPLOAD_KEYSTORE` and `ANDROID_UPLOAD_PASSWORD` are not in the
   upload step's environment.
2. **App Links preflight (`#445`):** Before uploading to Google Play, the
   workflow runs `node server/tools/probe.mjs site` to confirm that
   `https://totaldrop.app/.well-known/assetlinks.json` is live and declares
   `app.totaldrop` without redirects.
3. **Manifest, digest, and `versionCode` agreement:** `tools/play-upload.mjs`
   cross-checks the `android-v<semver>` tag against `versions.json` and
   `app/pubspec.yaml`, verifies the bundle against `dist/SHA256SUMS`, and
   confirms that the `versionCode` and `sha256` returned by Google Play's
   `edits.bundles.upload` endpoint match the local file before committing the
   `AppEdit`. If any step fails before commit, the tool deletes the uncommitted
   `AppEdit` on a best-effort basis (logging a warning if cleanup fails).
4. **Pinned endpoints and secret hygiene:** `tools/play-upload.mjs` reads
   credentials only from `GOOGLE_PLAY_SERVICE_ACCOUNT_JSON` (never CLI
   arguments), talks only to `https://oauth2.googleapis.com/token` and
   `https://androidpublisher.googleapis.com`, escapes control characters in
   remote error messages, and never logs private keys, JWT assertions, or
   access tokens.

## One-Time Setup (Repository Owner Only)

Credentials live only in GitHub Actions environment secrets (`docs/surec.md`,
section *Gizli bilgiler*); no agent session reads or handles the key file.

1. **Enable the API and create a service account (Google Cloud Console):**
   - Select or create a dedicated Google Cloud project for TotalDrop Play
     publishing.
   - Enable the **Google Play Android Developer API**
     (`androidpublisher.googleapis.com`).
   - Under **IAM & Admin -> Service Accounts**, create a service account (for
     example, `totaldrop-play-release`) **without** granting project-wide IAM
     roles, then create one JSON key and keep it temporarily in memory or an
     access-restricted file until step 3.
2. **Grant app release access (Google Play Console):**
   - Open **Google Play Console -> Users and permissions -> Invite new users**.
   - Enter the service account's `client_email`, select **`app.totaldrop`**
     under **App permissions**, and grant **Releases** permissions for the
     tracks CI should manage (*Release to testing tracks*, and *Release to
     production, exclude devices, and use Play App Signing* only when ready for
     production uploads).
   - Send/apply the invitation (service accounts activate immediately without
     an email confirmation click).
3. **Store secrets and optional variables (`GitHub -> Settings -> Environments -> play-release`):**
   - Add environment secret **`GOOGLE_PLAY_SERVICE_ACCOUNT_JSON`** containing
     the full JSON key file contents, then delete any local copy of the JSON
     file.
   - Optionally configure environment variables in `play-release` (defaults
     apply when unset):

| Environment variable (`play-release`) | Default | Allowed values / behavior |
| --- | --- | --- |
| `GOOGLE_PLAY_TRACK` | `internal` | `internal`, `alpha` (Closed testing), `beta` (Open testing), `production`, or a custom closed-testing track name |
| `GOOGLE_PLAY_RELEASE_STATUS` | `draft` | `draft`, `completed`, `halted`, or `inProgress` (note: Google Play requires `draft` until an app has completed its first Play Console review) |
| `GOOGLE_PLAY_USER_FRACTION` | *(unset)* | Decimal strictly between `0` and `1` (required only when `GOOGLE_PLAY_RELEASE_STATUS` is `inProgress`) |
| `GOOGLE_PLAY_CHANGES_NOT_SENT_FOR_REVIEW` | *(unset)* | `true` / `1` to pass `changesNotSentForReview=true` on `edits.commit` when Play Console requires manual review submission |
