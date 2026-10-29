<!-- Translated from README.md e572385d1192. -->
<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../../docs/assets/lockup-dark.svg">
  <img src="../../docs/assets/lockup-light.svg" alt="TotalDrop" width="300">
</picture>

### Οποιοδήποτε αρχείο, σε οποιονδήποτε, οπουδήποτε.

Δωρεάν και ανοιχτού κώδικα. Κρυπτογράφηση από άκρο σε άκρο. Χωρίς email, χωρίς αριθμό τηλεφώνου, χωρίς λογαριασμό.

[![Linux](https://img.shields.io/github/v/release/totaldrop-app/totaldrop?filter=linux-v*&include_prereleases&label=Linux&color=6D4AE0)](https://github.com/totaldrop-app/totaldrop/releases?q=linux&expanded=true)
[![Android](https://img.shields.io/badge/Android-closed%20beta-6D4AE0)](https://totaldrop.app/download/)
[![CI](https://img.shields.io/github/actions/workflow/status/totaldrop-app/totaldrop/ci.yml?branch=main&event=push&label=CI)](https://github.com/totaldrop-app/totaldrop/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL--3.0%20%C2%B7%20MIT%2FApache--2.0-22184F)](../../LICENSING.md)
[![Sponsor](https://img.shields.io/github/sponsors/ozdil?label=Sponsor&logo=githubsponsors&color=EA4AAA)](https://github.com/sponsors/ozdil)

**[Ιστότοπος](https://totaldrop.app)** · **[Λήψη](https://totaldrop.app/download/)** · [Οδικός χάρτης](https://totaldrop.app/roadmap/) · [Δωρεά](https://totaldrop.app/donate/) · [Εκδόσεις](https://github.com/totaldrop-app/totaldrop/releases)

<!-- languages -->
<sub>[العربية](README.ar.md) · [বাংলা](README.bn.md) · [Čeština](README.cs.md) · [Dansk](README.da.md) · [Deutsch](README.de.md) · **Ελληνικά** · [English](../../README.md) · [Español](README.es.md) · [فارسی](README.fa.md) · [Suomi](README.fi.md) · [Français](README.fr.md) · [עברית](README.he.md) · [हिन्दी](README.hi.md) · [Magyar](README.hu.md) · [Bahasa Indonesia](README.id.md) · [Italiano](README.it.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Bahasa Melayu](README.ms.md) · [Norsk bokmål](README.nb.md) · [Nederlands](README.nl.md) · [Polski](README.pl.md) · [Português (Brasil)](README.pt-BR.md) · [Português (Portugal)](README.pt-PT.md) · [Română](README.ro.md) · [Русский](README.ru.md) · [Svenska](README.sv.md) · [Kiswahili](README.sw.md) · [ไทย](README.th.md) · [Türkçe](README.tr.md) · [Українська](README.uk.md) · [اردو](README.ur.md) · [Tiếng Việt](README.vi.md) · [简体中文](README.zh-Hans.md) · [繁體中文](README.zh-Hant.md)</sub>
<!-- /languages -->

<br>

<img src="../../docs/assets/android.png" alt="Το TotalDrop στο Android: η αρχική οθόνη σε φωτεινό και σκοτεινό θέμα, και μια μεταφορά που έφτασε και επαληθεύτηκε" width="820">

</div>

> [!NOTE]
> Αυτή η μετάφραση είναι προσχέδιο και βρίσκεται υπό έλεγχο. Το κείμενο αναφοράς είναι το [αγγλικό](../../README.md).

## Γιατί TotalDrop

- **Αρκεί ένας σύντομος κωδικός.** Επιλέξτε αρχεία και πάρτε έναν κωδικό όπως `7-harbor-rocket`. Διαβάστε τον φωναχτά ή δείξτε τον κωδικό QR· ο άλλος τον πληκτρολογεί και τα αρχεία φτάνουν.
- **Κρυπτογράφηση από άκρο σε άκρο, που μπορείτε να ελέγξετε.** Και οι δύο οθόνες δείχνουν τις ίδιες τέσσερις λέξεις πριν μεταφερθεί οτιδήποτε. Αν διαφέρουν, σταματήστε.
- **Τίποτα για εγγραφή.** Χωρίς λογαριασμό, email ή αριθμό τηλεφώνου. Οι προαιρετικοί λογαριασμοί θα είναι μόνο αριθμοί, όπως στο Mullvad ([σχεδιασμός](../../docs/hesap.md)).
- **Απευθείας από συσκευή σε συσκευή, οπουδήποτε.** Ο διακομιστής συνάντησης απλώς συστήνει τις δύο συσκευές και δεν κρατά αρχεία καταγραφής· όπως κάθε διακομιστής, βλέπει τις διευθύνσεις IP τους. Οι συσκευές συνδέονται απευθείας μέσω διαδικτύου όπου μπορούν. Όταν παρεμβάλλεται δρομολογητής ή τείχος προστασίας, ένας διακομιστής αναμετάδοσης μεταφέρει την κίνηση χωρίς να μπορεί να τη διαβάσει.
- **Σαν στο σπίτι του στο σύστημά σας.** Οι εφαρμογές ακολουθούν το ανοιχτό ή σκούρο θέμα του συστήματος, το χρώμα έμφασης, τη γλώσσα και τη ρύθμιση μειωμένης κίνησης. Η εφαρμογή Linux είναι εγγενής Qt 6 σε Wayland και παίρνει επίσης ζωντανά γραμματοσειρές, γωνίες και εικονίδια από τα θέματα των KDE Plasma, Hyprland και Omarchy.
- **Δωρεάν, για πάντα.** Ανοιχτός κώδικας με χρηματοδότηση από δωρεές. Χωρίς διαφημίσεις, χωρίς παρακολούθηση, χωρίς επί πληρωμή επίπεδο για τα βασικά.

<p align="center">
  <img src="../../docs/assets/linux.png" alt="Η εφαρμογή Linux στο Omarchy, με τα χρώματα, τις γραμματοσειρές και τα εικονίδια της επιφάνειας εργασίας" width="560">
  <br><sub>Η εφαρμογή Linux στο Omarchy: τα χρώματα, οι γωνίες, οι γραμματοσειρές και τα εικονίδια προέρχονται από την επιφάνεια εργασίας.</sub>
</p>

## Λήψη

| Πλατφόρμα | Λήψη | Σημειώσεις |
| --- | --- | --- |
| **Linux** (x86-64) | [Arch / Omarchy](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0_beta.1-1-x86_64.pkg.tar.zst) · [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_amd64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.x86_64.rpm) | 0.2.0-beta.1, Qt 6.8 ή νεότερο |
| **Linux** (ARM64) | [Debian / Ubuntu](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop_0.2.0.beta.1-1_arm64.deb) · [Fedora](https://github.com/totaldrop-app/totaldrop/releases/download/linux-v0.2.0-beta.1/totaldrop-0.2.0.beta.1-1.fc43.aarch64.rpm) | 0.2.0-beta.1, Qt 6.8 ή νεότερο |
| **Android** | [Συμμετοχή στη beta](mailto:hello@totaldrop.app?subject=TotalDrop%20Android%20beta) | 0.2.0-beta.2, Android 15 ή νεότερο, Google Play με πρόσκληση |
| **Γραμμή εντολών** | `cargo install --git https://github.com/totaldrop-app/totaldrop totaldrop-cli` | Linux, macOS, Windows |
| **Windows, macOS, iOS** | Έρχεται | Η υπογραφή κώδικα για αυτά είναι το πρώτο που πληρώνουν οι δωρεές |

Κάθε πακέτο κατασκευάζεται από το CI από ένα commit με ετικέτα, ποτέ στον υπολογιστή κάποιου προγραμματιστή. Για να ελέγξετε ένα πακέτο πριν το εγκαταστήσετε, συγκρίνετέ το με το `SHA256SUMS` της έκδοσης και επαληθεύστε πού κατασκευάστηκε:

```bash
gh attestation verify totaldrop_0.2.0.beta.1-1_amd64.deb --repo totaldrop-app/totaldrop
```

> [!NOTE]
> **Beta.** Ο κώδικας δεν έχει περάσει ακόμη από ανεξάρτητο έλεγχο ασφαλείας, γι’ αυτό μην βασίζεστε σε αυτόν για ευαίσθητα αρχεία.

## Πώς λειτουργεί

1. Ο αποστολέας ζητά από τον διακομιστή συνάντησης έναν σύντομο κωδικό. Ο κωδικός είναι το μόνο μυστικό που μοιράζονται τα δύο άτομα.
2. Και οι δύο συσκευές μετατρέπουν τον κωδικό σε ισχυρό κλειδί με το **CPace**. Ένας επιτιθέμενος έχει μία προσπάθεια ανά μεταφορά και δεν μπορεί να δοκιμάσει κωδικούς εκτός σύνδεσης.
3. Συνδέονται απευθείας μέσω **QUIC** (TLS 1.3 με καρφιτσωμένα ακατέργαστα δημόσια κλειδιά) και εκτελούν **Noise XXpsk3 με ML-KEM-768**, που έχει σχεδιαστεί για να αντέχει απέναντι σε μελλοντικούς κβαντικούς υπολογιστές.
4. Τα αρχεία κρυπτογραφούνται σε τμήματα (STREAM), ώστε κάθε κομμάτι να επαληθεύεται καθώς φτάνει.

Το πρωτόκολλο περιγράφεται στο *TotalDrop Protocol v0*. Οι αποφάσεις που πάρθηκαν κατά την υλοποίησή του βρίσκονται στο [docs/protocol-v0.1-notes.md](../../docs/protocol-v0.1-notes.md), και το τι μπορεί και τι δεν μπορεί να δει κάθε διακομιστής περιγράφεται στο [μοντέλο απειλών](../../docs/tehdit-modeli.md) (στα τουρκικά) και στην [πολιτική απορρήτου](https://totaldrop.app/privacy).

## Στη γλώσσα σας

Το TotalDrop μιλά 35 γλώσσες και ακολουθεί τις μορφές αριθμών της περιοχής σας. Τα αγγλικά και τα τουρκικά έχουν ελεγχθεί· οι άλλες 33 είναι προσχέδια που ελέγχουν φυσικοί ομιλητές, γι’ αυτό οι εφαρμογές κρατούν τα κείμενα ασφαλείας (επαλήθευση, λάθος κωδικός, αποδοχή και απόρριψη) στα αγγλικά μέχρι να ελεγχθεί μια γλώσσα. Ο ιστότοπος τις έχει όλες ήδη σήμερα· οι εφαρμογές θα τις πάρουν με τις επόμενες εκδόσεις τους.

Βρήκατε κάποια αδέξια πρόταση; [Βοηθήστε στη μετάφραση](../../docs/ceviri.md) (ο οδηγός είναι στα τουρκικά).

## Υποστηρίξτε το έργο

Το TotalDrop μένει δωρεάν επειδή οι άνθρωποι συνεισφέρουν. Οι δωρεές πληρώνουν τον διακομιστή αναμετάδοσης, τα πιστοποιητικά υπογραφής κώδικα που απαιτούν τα Windows και το macOS, και έναν ανεξάρτητο έλεγχο ασφαλείας πριν από την 1.0. Κάθε κόστος, γραμμή προς γραμμή, και όσα έχουν συγκεντρωθεί μέχρι τώρα: [totaldrop.app/donate](https://totaldrop.app/donate/).

<a href="https://github.com/sponsors/ozdil"><img src="https://img.shields.io/badge/Sponsor%20on%20GitHub-EA4AAA?logo=githubsponsors&logoColor=white&style=for-the-badge" alt="Χορηγία στο GitHub"></a>

Μπορείτε επίσης να βοηθήσετε [αναφέροντας σφάλματα](https://github.com/totaldrop-app/totaldrop/issues/new/choose), μεταφράζοντας ή δοκιμάζοντας την εφαρμογή στον υπολογιστή σας.

## Για προγραμματιστές

### Δομή του αποθετηρίου

| Διαδρομή | Τι είναι | Άδεια |
| --- | --- | --- |
| `crates/totaldrop-crypto` | Σουίτα TD0: CPace, παραγωγή κλειδιών, κρυπτογράφηση γραμματοκιβωτίου και συνεδρίας, κλειδιά συσκευής, κρυπτογράφηση τμημάτων STREAM | MIT OR Apache-2.0 |
| `crates/totaldrop-proto` | Κωδικοί μεταφοράς, όρια, ασφάλεια διαδρομών, κάθε μήνυμα του πρωτοκόλλου (CBOR) | MIT OR Apache-2.0 |
| `crates/totaldrop-core` | Μηχανή πελάτη: επικοινωνία με τον διακομιστή συνάντησης, QUIC με ακατέργαστα δημόσια κλειδιά, Noise XXpsk3 + ML-KEM-768, μεταφορά | MIT OR Apache-2.0 |
| `crates/totaldrop-l10n` | Κείμενα διεπαφής για τα προγράμματα Rust: διαπραγμάτευση γλώσσας, μηνύματα ICU, πληθυντικοί, τοπικές μορφές αριθμών | MIT OR Apache-2.0 |
| `crates/totaldrop-cli` | Ο πελάτης γραμμής εντολών `totaldrop` | GPL-3.0-or-later |
| `l10n/` | Όλα τα κείμενα της διεπαφής, κοινά για Android, Linux και CLI ([μετάφραση](../../docs/ceviri.md)) | GPL-3.0-or-later |
| `linux/` | Εφαρμογή επιφάνειας εργασίας για Linux: Qt 6 και QML πάνω από το `totaldrop-core`, εγγενές Wayland | GPL-3.0-or-later |
| `packaging/` | Πακέτα Linux, κάθε μορφή χωριστά: Arch (`PKGBUILD`), Debian (`debian/`), RPM (`.spec`) | GPL-3.0-or-later |
| `app/` | Εφαρμογή Android (Flutter)· iOS αργότερα | GPL-3.0-or-later |
| `site/` | totaldrop.app, download.totaldrop.app και η σελίδα που ανοίγουν οι σύνδεσμοι QR | GPL-3.0-or-later; fonts SIL OFL 1.1 |
| `fuzz/` | Στόχοι fuzzing για κάθε αναλυτή | MIT OR Apache-2.0 |
| `server/totaldrop-rendezvous` | Διακομιστής συνάντησης σε Rust, για αυτοφιλοξενία και δοκιμές | AGPL-3.0-or-later |
| `server/rendezvous-worker` | Ο ίδιος διακομιστής στο Cloudflare Workers, για το `rendezvous.totaldrop.app` | AGPL-3.0-or-later |

Οι βιβλιοθήκες έχουν ελαστικές άδειες ώστε ο καθένας να μπορεί να χτίσει πάνω στο πρωτόκολλο· οι εφαρμογές είναι GPL και οι διακομιστές AGPL. Δείτε το [LICENSING.md](../../LICENSING.md).

Κάθε πλατφόρμα έχει τη δική της έκδοση, ετικέτα (`linux-v…`, `android-v…`) και ροή εργασιών CI, ώστε μία να μπορεί να κυκλοφορήσει χωρίς να επηρεάσει τις άλλες. Πώς λειτουργούν οι κλάδοι και οι εκδόσεις: [docs/gelistirme.md](../../docs/gelistirme.md) (στα τουρκικά) και [docs/versioning.md](../../docs/versioning.md). Κανόνες σχεδιασμού: [docs/tasarim.md](../../docs/tasarim.md) και αρχιτεκτονικές αποφάσεις: [docs/kararlar](../../docs/kararlar/README.md), και τα δύο στα τουρκικά.

### Κατασκευή

```bash
cargo test
cargo build --release
```

Δοκιμάστε μια μεταφορά σε έναν υπολογιστή, σε τρία τερματικά:

```bash
# 1. a local rendezvous server
./target/release/totaldrop-rendezvous 127.0.0.1:4000

# 2. the sender prints a code such as 3-harbor-rocket
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop send ./photos

# 3. the receiver
TOTALDROP_SERVER=ws://127.0.0.1:4000 ./target/release/totaldrop receive 3-harbor-rocket
```

Και οι δύο πλευρές δείχνουν τις ίδιες τέσσερις λέξεις επαλήθευσης. Το μη κρυπτογραφημένο `ws://` γίνεται δεκτό μόνο για διακομιστή στον ίδιο υπολογιστή· οτιδήποτε άλλο πρέπει να είναι `wss://`.

Το έργο στοχεύει πάντα στη νεότερη σταθερή έκδοση της Rust (καρφιτσωμένη στο `rust-toolchain.toml`)· το rustup την εγκαθιστά στην πρώτη κατασκευή.

**Εφαρμογή Linux** (χρειάζεται Qt 6.8 ή νεότερο: `qt6-base`, `qt6-declarative`, `qt6-wayland`, `qt6-svg`):

```bash
cargo run --release -p totaldrop-linux
packaging/build.sh arch   # or deb, rpm: a package in dist/packages/, built in a container
```

### Συνεισφορά και ασφάλεια

Διαβάστε το [CONTRIBUTING.md](../../CONTRIBUTING.md) πριν ανοίξετε ένα pull request, καθώς και τον [κώδικα δεοντολογίας](../../CODE_OF_CONDUCT.md) μας. Για ερωτήσεις, δείτε το [SUPPORT.md](../../SUPPORT.md). Αναφέρετε τα κενά ασφαλείας ιδιωτικά, όπως περιγράφεται στο [SECURITY.md](../../SECURITY.md), ποτέ σε δημόσιο issue.
