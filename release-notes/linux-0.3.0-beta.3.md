One window, transfers that hold through the relay, and Stop that stops. **Beta:** the code has not been reviewed independently yet.

For transfers with a phone, also update the Android app to 0.3.0-beta.3. The protocol is unchanged (v0), so this release works with every earlier release.

## Install

| Distribution | File | Command |
| --- | --- | --- |
| Arch, Omarchy, Manjaro, EndeavourOS | `totaldrop-0.3.0_beta.3-1-x86_64.pkg.tar.zst` | `sudo pacman -U ./totaldrop-0.3.0_beta.3-1-x86_64.pkg.tar.zst` |
| Debian 13+, Ubuntu 25.04+ (x86-64) | `totaldrop_0.3.0.beta.3-1_amd64.deb` | `sudo apt install ./totaldrop_0.3.0.beta.3-1_amd64.deb` |
| Debian 13+, Ubuntu 25.04+ (ARM64) | `totaldrop_0.3.0.beta.3-1_arm64.deb` | `sudo apt install ./totaldrop_0.3.0.beta.3-1_arm64.deb` |
| Fedora 43+ (x86-64) | `totaldrop-0.3.0.beta.3-1.fc43.x86_64.rpm` | `sudo dnf install ./totaldrop-0.3.0.beta.3-1.fc43.x86_64.rpm` |
| Fedora 43+ (ARM64) | `totaldrop-0.3.0.beta.3-1.fc43.aarch64.rpm` | `sudo dnf install ./totaldrop-0.3.0.beta.3-1.fc43.aarch64.rpm` |

Check the download first: `sha256sum -c SHA256SUMS --ignore-missing`. Each package was built in CI from this tag and carries signed build provenance, which you can check with `gh attestation verify <file> --repo totaldrop-app/totaldrop`. Each package installs the desktop app (`totaldrop-desktop`) and the `totaldrop` command, which is now 0.2.0-beta.2.

## Changes

- **One window.** Opening files with TotalDrop while it is already open, for example with "Open with" in a file manager, adds them to the open window instead of starting a second one, and the window comes to the front. During a transfer the request waits until the transfer is over. A relative path on the command line now opens too.
- **Transfers through the relay.** A transfer through the relay connects even when the relay is slow to get going, instead of failing after 20 seconds, and the connection is made again when the relay closes it. Over a long round trip it is also faster.
- **No stop after the code.** A transfer no longer sometimes stops right after the code is entered, with "the other side left" on the sending device while the receiving device waits.
- **Stop stops.** Stop on the sending device now cuts the transfer at once. Before, files already on the way could still arrive in full and be kept on the receiving device.
- **Android 0.3.0-beta.3** keeps a transfer going when you switch to another app, for example to share the code, and shows it in a notification with a Stop button.

Full history: [CHANGELOG.md](https://github.com/totaldrop-app/totaldrop/blob/main/CHANGELOG.md).

---

**Türkçe:** Tek pencere, relay üzerinden kopmayan aktarımlar ve gerçekten duran Durdur.

- **Tek pencere.** TotalDrop açıkken dosyaları onunla açtığınızda (örneğin dosya yöneticisinde "Birlikte aç" ile) ikinci bir pencere açılmaz; dosyalar açık pencereye eklenir ve pencere öne gelir. Aktarım sürerken istek, aktarım bitene kadar bekler. Komut satırında göreli bir yol da artık açılıyor.
- **Relay üzerinden aktarım.** Relay geç açılsa da aktarım 20 saniyede düşmek yerine bağlanır; relay bağlantıyı kapatırsa yeniden kurulur. Gidiş-dönüş süresi uzun olduğunda aktarım ayrıca daha hızlı.
- **Koddan sonra durma yok.** Aktarım artık kod girildikten hemen sonra, gönderen cihazda "karşı taraf ayrıldı" yazıp alan cihaz beklerken durmuyor.
- **Durdur gerçekten durdurur.** Gönderen cihazdaki Durdur aktarımı hemen keser. Önceden yolda olan dosyalar yine de tam olarak ulaşıp alan cihazda kalabiliyordu.
- **Android 0.3.0-beta.3**, örneğin kodu paylaşmak için başka bir uygulamaya geçtiğinizde aktarımı sürdürür ve onu Durdur düğmeli bir bildirimde gösterir.

Telefonla aktarım için Android uygulamasını da 0.3.0-beta.3'e güncelleyin. Kurulum için yukarıdaki tablodan dağıtımınıza uyan dosyayı indirin.

**Other languages.** These translations are drafts that native speakers have not checked yet; English and Turkish are the reference.

<details lang="de">
<summary>Deutsch</summary>

Ein Fenster, Übertragungen, die über das Relay halten, und ein Stopp, der wirklich stoppt.

- **Ein Fenster:** Öffnen Sie Dateien mit TotalDrop, während es schon läuft, kommen sie ins offene Fenster, statt ein zweites zu starten, und das Fenster kommt nach vorne.
- **Über das Relay:** Eine Übertragung über das Relay verbindet sich auch, wenn das Relay langsam startet, statt nach 20 Sekunden abzubrechen, und baut die Verbindung neu auf, wenn das Relay sie schließt.
- **Kein Abbruch nach dem Code:** Eine Übertragung bricht nicht mehr manchmal direkt nach der Codeeingabe mit „Die Gegenseite hat die Verbindung beendet“ ab.
- **Stopp stoppt:** Stopp auf dem sendenden Gerät beendet die Übertragung sofort; schon unterwegs befindliche Dateien werden nicht mehr vollständig auf dem empfangenden Gerät behalten.
- **Android:** Die Übertragung läuft weiter, wenn Sie zu einer anderen App wechseln, etwa um den Code zu teilen, und eine Benachrichtigung mit Stopp-Taste zeigt sie an.

Für Übertragungen mit einem Telefon aktualisieren Sie auch die Android-App auf 0.3.0-beta.3.

</details>

<details lang="fr">
<summary>Français</summary>

Une seule fenêtre, des transferts qui tiennent via le relais, et un bouton Arrêter qui arrête vraiment.

- **Une seule fenêtre :** ouvrir des fichiers avec TotalDrop alors qu’il est déjà ouvert les ajoute à la fenêtre ouverte au lieu d’en lancer une seconde, et la fenêtre passe au premier plan.
- **Via le relais :** un transfert via le relais se connecte même si le relais tarde à démarrer, au lieu d’échouer après 20 secondes, et la connexion est rétablie si le relais la ferme.
- **Plus d’arrêt après le code :** un transfert ne s’arrête plus parfois juste après la saisie du code, avec « l’autre appareil est parti ».
- **Arrêter arrête :** Arrêter sur l’appareil qui envoie coupe le transfert immédiatement ; les fichiers déjà en route ne sont plus conservés en entier sur l’appareil qui reçoit.
- **Android :** le transfert continue quand vous passez à une autre application, par exemple pour partager le code, et une notification avec un bouton Arrêter l’affiche.

Pour les transferts avec un téléphone, mettez aussi l’application Android à jour vers 0.3.0-beta.3.

</details>

<details lang="es">
<summary>Español</summary>

Una sola ventana, transferencias que aguantan a través del relé y un Detener que de verdad detiene.

- **Una sola ventana:** al abrir archivos con TotalDrop cuando ya está abierto, se añaden a la ventana abierta en lugar de abrir una segunda, y la ventana pasa al frente.
- **A través del relé:** una transferencia por el relé se conecta aunque el relé tarde en arrancar, en vez de fallar a los 20 segundos, y la conexión se rehace si el relé la cierra.
- **Sin cortes tras el código:** una transferencia ya no se detiene a veces justo después de introducir el código, con «el otro dispositivo se fue».
- **Detener detiene:** Detener en el dispositivo que envía corta la transferencia al instante; los archivos que ya estaban en camino ya no se guardan completos en el dispositivo que recibe.
- **Android:** la transferencia sigue cuando cambias a otra app, por ejemplo para compartir el código, y una notificación con un botón Detener la muestra.

Para transferir con un teléfono, actualiza también la app de Android a 0.3.0-beta.3.

</details>

<details lang="it">
<summary>Italiano</summary>

Una sola finestra, trasferimenti che reggono tramite il relay e un Interrompi che interrompe davvero.

- **Una sola finestra:** aprendo file con TotalDrop quando è già aperto, vengono aggiunti alla finestra aperta invece di avviarne una seconda, e la finestra passa in primo piano.
- **Tramite il relay:** un trasferimento tramite il relay si collega anche se il relay è lento ad avviarsi, invece di fallire dopo 20 secondi, e la connessione viene ristabilita se il relay la chiude.
- **Nessuno stop dopo il codice:** un trasferimento non si ferma più a volte subito dopo l’inserimento del codice, con «l’altro dispositivo se n’è andato».
- **Interrompi interrompe:** Interrompi sul dispositivo che invia taglia subito il trasferimento; i file già in viaggio non vengono più conservati interi sul dispositivo che riceve.
- **Android:** il trasferimento continua quando passi a un’altra app, ad esempio per condividere il codice, e una notifica con il pulsante Interrompi lo mostra.

Per i trasferimenti con un telefono, aggiorna anche l’app Android alla 0.3.0-beta.3.

</details>

<details lang="pt-BR">
<summary>Português (Brasil)</summary>

Uma só janela, transferências que se mantêm pelo relay e um Parar que realmente para.

- **Uma só janela:** abrir arquivos com o TotalDrop quando ele já está aberto os adiciona à janela aberta em vez de abrir uma segunda, e a janela vem para a frente.
- **Pelo relay:** uma transferência pelo relay se conecta mesmo quando o relay demora para começar, em vez de falhar após 20 segundos, e a conexão é refeita se o relay a fechar.
- **Sem parada depois do código:** uma transferência não para mais, às vezes, logo depois de digitar o código, com “o outro dispositivo saiu”.
- **Parar para:** Parar no dispositivo que envia interrompe a transferência na hora; arquivos que já estavam a caminho não ficam mais inteiros no dispositivo que recebe.
- **Android:** a transferência continua quando você muda para outro app, por exemplo para compartilhar o código, e uma notificação com botão Parar a mostra.

Para transferir com um celular, atualize também o app Android para 0.3.0-beta.3.

</details>

<details lang="pt-PT">
<summary>Português (Portugal)</summary>

Uma só janela, transferências que se aguentam através do relay e um Parar que para mesmo.

- **Uma só janela:** abrir ficheiros com o TotalDrop quando já está aberto adiciona-os à janela aberta em vez de abrir uma segunda, e a janela passa para a frente.
- **Através do relay:** uma transferência através do relay liga-se mesmo quando o relay demora a arrancar, em vez de falhar ao fim de 20 segundos, e a ligação é refeita se o relay a fechar.
- **Sem paragem depois do código:** uma transferência já não para, por vezes, logo após introduzir o código, com «o outro dispositivo saiu».
- **Parar para:** Parar no dispositivo que envia corta a transferência de imediato; os ficheiros que já iam a caminho deixam de ficar inteiros no dispositivo que recebe.
- **Android:** a transferência continua quando muda para outra aplicação, por exemplo para partilhar o código, e uma notificação com um botão Parar mostra-a.

Para transferências com um telemóvel, atualize também a aplicação Android para 0.3.0-beta.3.

</details>

<details lang="nl">
<summary>Nederlands</summary>

Eén venster, overdrachten die via de relay blijven staan, en een Stoppen dat echt stopt.

- **Eén venster:** bestanden openen met TotalDrop terwijl het al open is, voegt ze toe aan het open venster in plaats van een tweede te starten, en het venster komt naar voren.
- **Via de relay:** een overdracht via de relay maakt ook verbinding als de relay traag op gang komt, in plaats van na 20 seconden te mislukken, en de verbinding wordt opnieuw gemaakt als de relay haar sluit.
- **Geen stop na de code:** een overdracht stopt niet meer soms direct na het invoeren van de code met ‘het andere apparaat is weg’.
- **Stoppen stopt:** Stoppen op het verzendende apparaat breekt de overdracht meteen af; bestanden die al onderweg waren, blijven niet meer volledig op het ontvangende apparaat staan.
- **Android:** de overdracht gaat door als je naar een andere app gaat, bijvoorbeeld om de code te delen, en een melding met een Stoppen-knop toont haar.

Werk voor overdrachten met een telefoon ook de Android-app bij naar 0.3.0-beta.3.

</details>

<details lang="pl">
<summary>Polski</summary>

Jedno okno, przesyłanie, które utrzymuje się przez przekaźnik, i Zatrzymaj, które naprawdę zatrzymuje.

- **Jedno okno:** otwarcie plików w TotalDrop, gdy jest już otwarty, dodaje je do otwartego okna zamiast uruchamiać drugie, a okno wychodzi na wierzch.
- **Przez przekaźnik:** przesyłanie przez przekaźnik łączy się, nawet gdy przekaźnik wolno startuje, zamiast kończyć się błędem po 20 sekundach, a połączenie jest nawiązywane ponownie, gdy przekaźnik je zamknie.
- **Bez zatrzymania po kodzie:** przesyłanie nie zatrzymuje się już czasem zaraz po wpisaniu kodu z komunikatem „drugie urządzenie odeszło”.
- **Zatrzymaj zatrzymuje:** Zatrzymaj na urządzeniu wysyłającym od razu przerywa przesyłanie; pliki, które były już w drodze, nie zostają już w całości na urządzeniu odbierającym.
- **Android:** przesyłanie trwa dalej, gdy przełączysz się do innej aplikacji, na przykład by udostępnić kod, a pokazuje je powiadomienie z przyciskiem Zatrzymaj.

Do przesyłania z telefonem zaktualizuj też aplikację na Androida do 0.3.0-beta.3.

</details>

<details lang="cs">
<summary>Čeština</summary>

Jedno okno, přenosy, které vydrží přes relay, a Zastavit, které opravdu zastaví.

- **Jedno okno:** otevření souborů v TotalDropu, když už běží, je přidá do otevřeného okna místo spuštění druhého a okno se přesune do popředí.
- **Přes relay:** přenos přes relay se připojí, i když se relay rozbíhá pomalu, místo aby po 20 sekundách selhal, a spojení se obnoví, když ho relay zavře.
- **Žádné zastavení po kódu:** přenos se už občas nezastaví hned po zadání kódu se zprávou „druhé zařízení odešlo“.
- **Zastavit zastaví:** Zastavit na odesílajícím zařízení přenos okamžitě přeruší; soubory, které už byly na cestě, už na přijímajícím zařízení nezůstanou celé.
- **Android:** přenos pokračuje, když přepnete do jiné aplikace, například kvůli sdílení kódu, a ukazuje ho oznámení s tlačítkem Zastavit.

Pro přenosy s telefonem aktualizujte také aplikaci pro Android na 0.3.0-beta.3.

</details>

<details lang="da">
<summary>Dansk</summary>

Ét vindue, overførsler der holder gennem relayet, og en Stop-knap der stopper.

- **Ét vindue:** åbner du filer med TotalDrop, mens det allerede er åbent, føjes de til det åbne vindue i stedet for at starte et nyt, og vinduet kommer frem.
- **Gennem relayet:** en overførsel gennem relayet forbinder, selv når relayet er langsomt til at starte, i stedet for at fejle efter 20 sekunder, og forbindelsen genoprettes, når relayet lukker den.
- **Intet stop efter koden:** en overførsel stopper ikke længere nogle gange lige efter, at koden er indtastet, med "den anden enhed forlod".
- **Stop stopper:** Stop på den afsendende enhed afbryder overførslen med det samme; filer, der allerede var på vej, bliver ikke længere gemt hele på den modtagende enhed.
- **Android:** overførslen fortsætter, når du skifter til en anden app, for eksempel for at dele koden, og en notifikation med en Stop-knap viser den.

Til overførsler med en telefon skal du også opdatere Android-appen til 0.3.0-beta.3.

</details>

<details lang="sv">
<summary>Svenska</summary>

Ett fönster, överföringar som håller via reläet och ett Stoppa som stoppar.

- **Ett fönster:** öppnar du filer med TotalDrop när det redan är öppet läggs de till i det öppna fönstret i stället för att starta ett andra, och fönstret kommer fram.
- **Via reläet:** en överföring via reläet ansluter även när reläet är långsamt att komma i gång, i stället för att misslyckas efter 20 sekunder, och anslutningen görs om när reläet stänger den.
- **Inget stopp efter koden:** en överföring stannar inte längre ibland direkt efter att koden matats in, med ”den andra enheten lämnade”.
- **Stoppa stoppar:** Stoppa på den sändande enheten avbryter överföringen direkt; filer som redan var på väg sparas inte längre hela på den mottagande enheten.
- **Android:** överföringen fortsätter när du byter till en annan app, till exempel för att dela koden, och en avisering med en Stoppa-knapp visar den.

För överföringar med en telefon, uppdatera även Android-appen till 0.3.0-beta.3.

</details>

<details lang="nb">
<summary>Norsk bokmål</summary>

Ett vindu, overføringer som holder gjennom reléet, og en Stopp som stopper.

- **Ett vindu:** åpner du filer med TotalDrop mens det allerede er åpent, legges de til i det åpne vinduet i stedet for å starte et nytt, og vinduet kommer frem.
- **Gjennom reléet:** en overføring gjennom reléet kobler til selv når reléet er tregt i gang, i stedet for å feile etter 20 sekunder, og forbindelsen opprettes på nytt når reléet lukker den.
- **Ingen stopp etter koden:** en overføring stopper ikke lenger noen ganger rett etter at koden er skrevet inn, med «den andre enheten forlot».
- **Stopp stopper:** Stopp på enheten som sender, avbryter overføringen med en gang; filer som allerede var på vei, blir ikke lenger lagret hele på enheten som mottar.
- **Android:** overføringen fortsetter når du bytter til en annen app, for eksempel for å dele koden, og et varsel med en Stopp-knapp viser den.

For overføringer med en telefon må du også oppdatere Android-appen til 0.3.0-beta.3.

</details>

<details lang="fi">
<summary>Suomi</summary>

Yksi ikkuna, välityspalvelimen kautta kestävät siirrot ja Pysäytä, joka todella pysäyttää.

- **Yksi ikkuna:** kun avaat tiedostoja TotalDropilla sen ollessa jo auki, ne lisätään avoimeen ikkunaan toisen sijaan, ja ikkuna nousee etualalle.
- **Välityspalvelimen kautta:** siirto välityspalvelimen kautta yhdistyy, vaikka palvelin käynnistyisi hitaasti, sen sijaan että se epäonnistuisi 20 sekunnin jälkeen, ja yhteys muodostetaan uudelleen, jos palvelin sulkee sen.
- **Ei pysähtymistä koodin jälkeen:** siirto ei enää joskus pysähdy heti koodin syöttämisen jälkeen ilmoitukseen ”toinen laite poistui”.
- **Pysäytä pysäyttää:** Pysäytä lähettävällä laitteella katkaisee siirron heti; jo matkalla olleita tiedostoja ei enää säilytetä kokonaisina vastaanottavalla laitteella.
- **Android:** siirto jatkuu, kun vaihdat toiseen sovellukseen esimerkiksi jakaaksesi koodin, ja ilmoitus Pysäytä-painikkeineen näyttää sen.

Puhelimen kanssa siirtämistä varten päivitä myös Android-sovellus versioon 0.3.0-beta.3.

</details>

<details lang="hu">
<summary>Magyar</summary>

Egy ablak, a közvetítőn át is kitartó átvitelek, és egy Leállítás, ami tényleg leállít.

- **Egy ablak:** ha a már futó TotalDroppal nyit meg fájlokat, azok a nyitott ablakba kerülnek, nem indul második, és az ablak előtérbe kerül.
- **A közvetítőn át:** a közvetítőn át zajló átvitel akkor is kapcsolódik, ha a közvetítő lassan indul, nem hiúsul meg 20 másodperc után, és a kapcsolat újraépül, ha a közvetítő bezárja.
- **Nincs leállás a kód után:** az átvitel már nem áll le néha közvetlenül a kód beírása után „a másik eszköz kilépett” üzenettel.
- **A Leállítás leállít:** a küldő eszközön a Leállítás azonnal megszakítja az átvitelt; a már úton lévő fájlok nem maradnak meg teljesen a fogadó eszközön.
- **Android:** az átvitel folytatódik, ha másik alkalmazásra vált, például a kód megosztásához, és egy Leállítás gombos értesítés mutatja.

Telefonos átvitelhez frissítse az Android-alkalmazást is 0.3.0-beta.3-ra.

</details>

<details lang="ro">
<summary>Română</summary>

O singură fereastră, transferuri care rezistă prin releu și un Oprește care chiar oprește.

- **O singură fereastră:** deschiderea fișierelor cu TotalDrop când este deja deschis le adaugă în fereastra deschisă în loc să pornească a doua, iar fereastra vine în față.
- **Prin releu:** un transfer prin releu se conectează chiar și când releul pornește greu, în loc să eșueze după 20 de secunde, iar conexiunea se refăce când releul o închide.
- **Fără oprire după cod:** un transfer nu se mai oprește uneori imediat după introducerea codului, cu „celălalt dispozitiv a plecat”.
- **Oprește oprește:** Oprește pe dispozitivul care trimite taie transferul imediat; fișierele aflate deja pe drum nu mai rămân întregi pe dispozitivul care primește.
- **Android:** transferul continuă când treci la altă aplicație, de exemplu pentru a partaja codul, iar o notificare cu buton Oprește îl arată.

Pentru transferuri cu un telefon, actualizează și aplicația Android la 0.3.0-beta.3.

</details>

<details lang="el">
<summary>Ελληνικά</summary>

Ένα παράθυρο, μεταφορές που αντέχουν μέσω του αναμεταδότη και Διακοπή που πράγματι διακόπτει.

- **Ένα παράθυρο:** αν ανοίξετε αρχεία με το TotalDrop ενώ είναι ήδη ανοιχτό, προστίθενται στο ανοιχτό παράθυρο αντί να ανοίξει δεύτερο, και το παράθυρο έρχεται μπροστά.
- **Μέσω του αναμεταδότη:** μια μεταφορά μέσω του αναμεταδότη συνδέεται ακόμη κι όταν ο αναμεταδότης αργεί να ξεκινήσει, αντί να αποτύχει μετά από 20 δευτερόλεπτα, και η σύνδεση αποκαθίσταται όταν ο αναμεταδότης την κλείσει.
- **Χωρίς διακοπή μετά τον κωδικό:** μια μεταφορά δεν σταματά πια μερικές φορές αμέσως μετά την εισαγωγή του κωδικού με «η άλλη συσκευή αποχώρησε».
- **Η Διακοπή διακόπτει:** η Διακοπή στη συσκευή αποστολής κόβει αμέσως τη μεταφορά· αρχεία που ήταν ήδη καθ’ οδόν δεν μένουν πια ολόκληρα στη συσκευή λήψης.
- **Android:** η μεταφορά συνεχίζεται όταν αλλάζετε σε άλλη εφαρμογή, για παράδειγμα για να μοιραστείτε τον κωδικό, και μια ειδοποίηση με κουμπί Διακοπή τη δείχνει.

Για μεταφορές με τηλέφωνο, ενημερώστε και την εφαρμογή Android στην 0.3.0-beta.3.

</details>

<details lang="ru">
<summary>Русский</summary>

Одно окно, передачи, которые держатся через ретранслятор, и «Остановить», которое действительно останавливает.

- **Одно окно:** если открыть файлы в TotalDrop, когда он уже запущен, они добавляются в открытое окно, а не открывают второе, и окно выходит на передний план.
- **Через ретранслятор:** передача через ретранслятор подключается, даже если он медленно запускается, а не обрывается через 20 секунд, и соединение восстанавливается, если ретранслятор его закрыл.
- **Без остановки после кода:** передача больше не останавливается иногда сразу после ввода кода с сообщением «другое устройство ушло».
- **«Остановить» останавливает:** «Остановить» на отправляющем устройстве сразу прерывает передачу; файлы, которые уже были в пути, больше не сохраняются целиком на принимающем устройстве.
- **Android:** передача продолжается, когда вы переходите в другое приложение, например чтобы поделиться кодом, и её показывает уведомление с кнопкой «Остановить».

Для передачи с телефоном обновите и приложение для Android до 0.3.0-beta.3.

</details>

<details lang="uk">
<summary>Українська</summary>

Одне вікно, передавання, що тримаються через ретранслятор, і «Зупинити», яке справді зупиняє.

- **Одне вікно:** якщо відкрити файли в TotalDrop, коли він уже запущений, вони додаються до відкритого вікна замість другого, і вікно виходить на передній план.
- **Через ретранслятор:** передавання через ретранслятор підключається, навіть коли той повільно запускається, а не обривається через 20 секунд, і з’єднання відновлюється, якщо ретранслятор його закрив.
- **Без зупинки після коду:** передавання більше не зупиняється іноді одразу після введення коду з повідомленням «інший пристрій пішов».
- **«Зупинити» зупиняє:** «Зупинити» на пристрої, що надсилає, одразу перериває передавання; файли, які вже були в дорозі, більше не зберігаються повністю на пристрої, що отримує.
- **Android:** передавання триває, коли ви перемикаєтеся в інший застосунок, наприклад щоб поділитися кодом, і його показує сповіщення з кнопкою «Зупинити».

Для передавання з телефоном оновіть і застосунок для Android до 0.3.0-beta.3.

</details>

<details lang="hi">
<summary>हिन्दी</summary>

एक ही विंडो, रिले के ज़रिए टिके रहने वाले ट्रांसफ़र, और ऐसा "रोकें" जो सच में रोकता है।

- **एक ही विंडो:** TotalDrop पहले से खुला हो और आप उससे फ़ाइलें खोलें, तो दूसरी विंडो खुलने के बजाय वे खुली विंडो में जुड़ जाती हैं, और विंडो सामने आ जाती है।
- **रिले के ज़रिए:** रिले के ज़रिए ट्रांसफ़र तब भी जुड़ता है जब रिले शुरू होने में धीमा हो, 20 सेकंड बाद विफल नहीं होता, और रिले कनेक्शन बंद करे तो वह फिर से जुड़ता है।
- **कोड के बाद रुकना नहीं:** कोड डालने के ठीक बाद ट्रांसफ़र अब कभी-कभी "दूसरा डिवाइस चला गया" के साथ नहीं रुकता।
- **रोकें सच में रोकता है:** भेजने वाले डिवाइस पर "रोकें" ट्रांसफ़र को तुरंत काट देता है; रास्ते में मौजूद फ़ाइलें अब पाने वाले डिवाइस पर पूरी नहीं रखी जातीं।
- **Android:** किसी दूसरे ऐप पर जाने पर, जैसे कोड साझा करने के लिए, ट्रांसफ़र चलता रहता है, और "रोकें" बटन वाली एक सूचना उसे दिखाती है।

फ़ोन के साथ ट्रांसफ़र के लिए Android ऐप को भी 0.3.0-beta.3 पर अपडेट करें।

</details>

<details lang="bn">
<summary>বাংলা</summary>

একটিই উইন্ডো, রিলের মাধ্যমে টিকে থাকা স্থানান্তর, আর এমন "থামান" যা সত্যিই থামায়।

- **একটিই উইন্ডো:** TotalDrop আগে থেকে খোলা থাকলে তা দিয়ে ফাইল খুললে দ্বিতীয় উইন্ডো না খুলে সেগুলো খোলা উইন্ডোতে যোগ হয়, আর উইন্ডোটি সামনে আসে।
- **রিলের মাধ্যমে:** রিলে চালু হতে দেরি করলেও রিলের মাধ্যমে স্থানান্তর ২০ সেকেন্ড পরে ব্যর্থ না হয়ে সংযুক্ত হয়, আর রিলে সংযোগ বন্ধ করলে তা আবার তৈরি হয়।
- **কোডের পরে থেমে যাওয়া নয়:** কোড দেওয়ার ঠিক পরে স্থানান্তর আর মাঝে মাঝে "অন্য ডিভাইস চলে গেছে" বলে থেমে যায় না।
- **থামান সত্যিই থামায়:** পাঠানো ডিভাইসে "থামান" স্থানান্তর সঙ্গে সঙ্গে কেটে দেয়; পথে থাকা ফাইল আর গ্রহণকারী ডিভাইসে পুরো রাখা হয় না।
- **Android:** অন্য অ্যাপে গেলে, যেমন কোড শেয়ার করতে, স্থানান্তর চলতে থাকে, আর "থামান" বোতামসহ একটি বিজ্ঞপ্তি তা দেখায়।

ফোনের সঙ্গে স্থানান্তরের জন্য Android অ্যাপটিও 0.3.0-beta.3-এ আপডেট করুন।

</details>

<details lang="th">
<summary>ไทย</summary>

หน้าต่างเดียว การโอนที่ผ่านรีเลย์ได้อย่างมั่นคง และปุ่มหยุดที่หยุดได้จริง

- **หน้าต่างเดียว:** เมื่อเปิดไฟล์ด้วย TotalDrop ขณะที่เปิดอยู่แล้ว ไฟล์จะถูกเพิ่มลงในหน้าต่างที่เปิดอยู่แทนการเปิดหน้าต่างที่สอง และหน้าต่างจะขึ้นมาด้านหน้า
- **ผ่านรีเลย์:** การโอนผ่านรีเลย์จะเชื่อมต่อได้แม้รีเลย์เริ่มทำงานช้า แทนที่จะล้มเหลวหลัง 20 วินาที และจะเชื่อมต่อใหม่เมื่อรีเลย์ปิดการเชื่อมต่อ
- **ไม่หยุดหลังใส่รหัส:** การโอนจะไม่หยุดอีกต่อไปในบางครั้งทันทีหลังใส่รหัสพร้อมข้อความ "อีกฝ่ายออกไปแล้ว"
- **หยุดได้จริง:** ปุ่มหยุดบนอุปกรณ์ที่ส่งจะตัดการโอนทันที ไฟล์ที่กำลังส่งอยู่จะไม่ถูกเก็บไว้ครบบนอุปกรณ์ที่รับอีกต่อไป
- **Android:** การโอนจะดำเนินต่อเมื่อคุณสลับไปแอปอื่น เช่น เพื่อแชร์รหัส และการแจ้งเตือนที่มีปุ่มหยุดจะแสดงการโอนนั้น

สำหรับการโอนกับโทรศัพท์ โปรดอัปเดตแอป Android เป็น 0.3.0-beta.3 ด้วย

</details>

<details lang="vi">
<summary>Tiếng Việt</summary>

Một cửa sổ, truyền tệp bền vững qua máy chuyển tiếp và nút Dừng thực sự dừng.

- **Một cửa sổ:** mở tệp bằng TotalDrop khi ứng dụng đang mở sẽ thêm chúng vào cửa sổ đang mở thay vì mở cửa sổ thứ hai, và cửa sổ được đưa lên trước.
- **Qua máy chuyển tiếp:** việc truyền qua máy chuyển tiếp vẫn kết nối dù máy chuyển tiếp khởi động chậm, thay vì thất bại sau 20 giây, và kết nối được tạo lại khi máy chuyển tiếp đóng nó.
- **Không dừng sau mã:** việc truyền không còn đôi khi dừng ngay sau khi nhập mã với thông báo "thiết bị kia đã rời đi".
- **Dừng là dừng:** nút Dừng trên thiết bị gửi cắt việc truyền ngay lập tức; các tệp đang trên đường không còn được giữ nguyên vẹn trên thiết bị nhận.
- **Android:** việc truyền tiếp tục khi bạn chuyển sang ứng dụng khác, ví dụ để chia sẻ mã, và một thông báo có nút Dừng hiển thị nó.

Để truyền với điện thoại, hãy cập nhật cả ứng dụng Android lên 0.3.0-beta.3.

</details>

<details lang="id">
<summary>Bahasa Indonesia</summary>

Satu jendela, transfer yang bertahan lewat relay, dan Hentikan yang benar-benar menghentikan.

- **Satu jendela:** membuka file dengan TotalDrop saat aplikasi sudah terbuka akan menambahkannya ke jendela yang terbuka alih-alih membuka jendela kedua, dan jendela muncul ke depan.
- **Lewat relay:** transfer lewat relay tetap tersambung meski relay lambat memulai, alih-alih gagal setelah 20 detik, dan sambungan dibuat ulang saat relay menutupnya.
- **Tidak berhenti setelah kode:** transfer tidak lagi terkadang berhenti tepat setelah kode dimasukkan dengan pesan "perangkat lain telah pergi".
- **Hentikan benar-benar menghentikan:** Hentikan di perangkat pengirim langsung memutus transfer; file yang sudah dalam perjalanan tidak lagi tersimpan utuh di perangkat penerima.
- **Android:** transfer tetap berjalan saat Anda beralih ke aplikasi lain, misalnya untuk membagikan kode, dan notifikasi dengan tombol Hentikan menampilkannya.

Untuk transfer dengan ponsel, perbarui juga aplikasi Android ke 0.3.0-beta.3.

</details>

<details lang="ms">
<summary>Bahasa Melayu</summary>

Satu tetingkap, pemindahan yang bertahan melalui geganti, dan Henti yang benar-benar menghentikan.

- **Satu tetingkap:** membuka fail dengan TotalDrop semasa ia sudah dibuka akan menambahkannya ke tetingkap yang terbuka dan bukan membuka tetingkap kedua, dan tetingkap itu dibawa ke hadapan.
- **Melalui geganti:** pemindahan melalui geganti tetap bersambung walaupun geganti lambat bermula, dan bukan gagal selepas 20 saat, dan sambungan dibuat semula apabila geganti menutupnya.
- **Tiada henti selepas kod:** pemindahan tidak lagi kadangkala berhenti sejurus selepas kod dimasukkan dengan mesej "peranti lain telah pergi".
- **Henti benar-benar menghentikan:** Henti pada peranti penghantar memutuskan pemindahan serta-merta; fail yang sudah dalam perjalanan tidak lagi disimpan penuh pada peranti penerima.
- **Android:** pemindahan diteruskan apabila anda bertukar ke aplikasi lain, contohnya untuk berkongsi kod, dan pemberitahuan dengan butang Henti memaparkannya.

Untuk pemindahan dengan telefon, kemas kini juga aplikasi Android ke 0.3.0-beta.3.

</details>

<details lang="sw">
<summary>Kiswahili</summary>

Dirisha moja, uhamishaji unaodumu kupitia relay, na Simamisha inayosimamisha kweli.

- **Dirisha moja:** ukifungua faili kwa TotalDrop ikiwa tayari imefunguliwa, zinaongezwa kwenye dirisha lililo wazi badala ya kufungua dirisha la pili, na dirisha linakuja mbele.
- **Kupitia relay:** uhamishaji kupitia relay unaunganishwa hata relay ikichelewa kuanza, badala ya kushindwa baada ya sekunde 20, na muunganisho unafanywa upya relay ikiufunga.
- **Hakuna kusimama baada ya msimbo:** uhamishaji hausimami tena wakati mwingine mara tu baada ya kuweka msimbo kwa ujumbe "kifaa kingine kimeondoka".
- **Simamisha inasimamisha:** Simamisha kwenye kifaa kinachotuma inakata uhamishaji mara moja; faili zilizokuwa njiani hazihifadhiwi tena zikiwa kamili kwenye kifaa kinachopokea.
- **Android:** uhamishaji unaendelea unapohamia programu nyingine, kwa mfano kushiriki msimbo, na arifa yenye kitufe cha Simamisha inauonyesha.

Kwa uhamishaji na simu, sasisha pia programu ya Android hadi 0.3.0-beta.3.

</details>

<details lang="ja">
<summary>日本語</summary>

ウィンドウは一つに、リレー経由でも途切れない転送、そして本当に止まる「停止」。

- **ウィンドウは一つ:** TotalDrop が開いているときにファイルを TotalDrop で開くと、2 つ目のウィンドウを開かずに開いているウィンドウへ追加され、ウィンドウが前面に出ます。
- **リレー経由:** リレーの起動が遅くても、20 秒で失敗せずに接続し、リレーが接続を閉じた場合は接続し直します。
- **コード入力後に止まらない:** コード入力の直後に「相手が退出しました」と表示されて転送が止まることがなくなりました。
- **「停止」で止まる:** 送信側の「停止」で転送はすぐに切れます。送信途中のファイルが受信側に完全な形で残ることはもうありません。
- **Android:** コードを共有するためなどに別のアプリへ切り替えても転送は続き、「停止」ボタン付きの通知に表示されます。

スマートフォンとの転送には、Android アプリも 0.3.0-beta.3 に更新してください。

</details>

<details lang="ko">
<summary>한국어</summary>

창은 하나로, 릴레이를 거쳐도 끊기지 않는 전송, 그리고 정말로 멈추는 중지.

- **창 하나:** TotalDrop이 이미 열려 있을 때 TotalDrop으로 파일을 열면 두 번째 창을 띄우지 않고 열린 창에 추가되며, 창이 앞으로 나옵니다.
- **릴레이를 거쳐:** 릴레이가 늦게 시작해도 20초 뒤에 실패하지 않고 연결되며, 릴레이가 연결을 닫으면 다시 연결합니다.
- **코드 입력 후 멈춤 없음:** 코드를 입력한 직후 "상대 기기가 나갔습니다"와 함께 전송이 멈추는 일이 더 이상 없습니다.
- **중지는 중지:** 보내는 기기에서 중지를 누르면 전송이 바로 끊깁니다. 이미 가는 중이던 파일이 받는 기기에 온전히 남지 않습니다.
- **Android:** 코드를 공유하려고 다른 앱으로 전환해도 전송이 계속되며, 중지 버튼이 있는 알림에 표시됩니다.

휴대폰과 전송하려면 Android 앱도 0.3.0-beta.3으로 업데이트하세요.

</details>

<details lang="zh-Hans">
<summary>简体中文</summary>

只有一个窗口，经由中继也不中断的传输，以及真正能停下的“停止”。

- **一个窗口**：TotalDrop 已打开时再用它打开文件，文件会加入已打开的窗口，而不是再开一个窗口，并且窗口会切到前台。
- **经由中继**：即使中继启动较慢，经由中继的传输也会连上，而不是 20 秒后失败；中继关闭连接时会重新连接。
- **输入代码后不再中断**：传输不再偶尔在输入代码后立即停止并显示“对方已离开”。
- **“停止”真正停止**：在发送设备上点“停止”会立即中断传输；已在途中的文件不再完整保留在接收设备上。
- **Android**：切换到其他应用（例如为了分享代码）时传输会继续，并通过带“停止”按钮的通知显示。

如需与手机互传，请同时将 Android 应用更新到 0.3.0-beta.3。

</details>

<details lang="zh-Hant">
<summary>繁體中文</summary>

只有一個視窗，經由中繼也不中斷的傳輸，以及真正能停下的「停止」。

- **一個視窗**：TotalDrop 已開啟時再用它開啟檔案，檔案會加入已開啟的視窗，而不是再開一個視窗，並且視窗會移到最前面。
- **經由中繼**：即使中繼啟動較慢，經由中繼的傳輸也會連上，而不是 20 秒後失敗；中繼關閉連線時會重新連線。
- **輸入代碼後不再中斷**：傳輸不再偶爾在輸入代碼後立刻停止並顯示「對方已離開」。
- **「停止」真正停止**：在傳送裝置上按「停止」會立即中斷傳輸；已在途中的檔案不再完整保留在接收裝置上。
- **Android**：切換到其他應用程式（例如為了分享代碼）時傳輸會繼續，並透過帶「停止」按鈕的通知顯示。

如需與手機互傳，請同時將 Android 應用程式更新至 0.3.0-beta.3。

</details>

<details lang="ar" dir="rtl">
<summary>العربية</summary>

نافذة واحدة، ونقل يصمد عبر خادم الترحيل، وزر إيقاف يوقف فعلًا.

- **نافذة واحدة:** إذا فتحت ملفات باستخدام TotalDrop وهو مفتوح بالفعل، تُضاف إلى النافذة المفتوحة بدل فتح نافذة ثانية، وتنتقل النافذة إلى الأمام.
- **عبر خادم الترحيل:** يتصل النقل عبر خادم الترحيل حتى لو تأخر الخادم في البدء، بدل أن يفشل بعد 20 ثانية، ويُعاد الاتصال إذا أغلقه الخادم.
- **لا توقف بعد الرمز:** لم يعد النقل يتوقف أحيانًا مباشرة بعد إدخال الرمز مع رسالة «غادر الجهاز الآخر».
- **الإيقاف يوقف:** زر الإيقاف على الجهاز المرسِل يقطع النقل فورًا؛ ولم تعد الملفات التي كانت في الطريق تُحفظ كاملة على الجهاز المستقبِل.
- **Android:** يستمر النقل عند الانتقال إلى تطبيق آخر، مثلًا لمشاركة الرمز، ويظهر في إشعار فيه زر إيقاف.

للنقل مع هاتف، حدّث تطبيق Android أيضًا إلى 0.3.0-beta.3.

</details>

<details lang="fa" dir="rtl">
<summary>فارسی</summary>

یک پنجره، انتقال‌هایی که از راه سرور رله پایدار می‌مانند، و دکمهٔ توقفی که واقعاً متوقف می‌کند.

- **یک پنجره:** اگر وقتی TotalDrop باز است فایل‌هایی را با آن باز کنید، به جای باز شدن پنجرهٔ دوم به پنجرهٔ باز اضافه می‌شوند و پنجره جلو می‌آید.
- **از راه رله:** انتقال از راه رله حتی وقتی رله دیر راه می‌افتد وصل می‌شود، به جای اینکه پس از ۲۰ ثانیه شکست بخورد، و اگر رله اتصال را ببندد دوباره برقرار می‌شود.
- **بدون توقف پس از کد:** انتقال دیگر گاهی درست پس از وارد کردن کد با پیام «دستگاه دیگر رفت» متوقف نمی‌شود.
- **توقف واقعاً متوقف می‌کند:** توقف روی دستگاه فرستنده انتقال را فوراً قطع می‌کند؛ فایل‌هایی که در راه بودند دیگر کامل روی دستگاه گیرنده نمی‌مانند.
- **Android:** با رفتن به برنامه‌ای دیگر، مثلاً برای اشتراک‌گذاری کد، انتقال ادامه می‌یابد و اعلانی با دکمهٔ توقف آن را نشان می‌دهد.

برای انتقال با تلفن، برنامهٔ Android را هم به 0.3.0-beta.3 به‌روز کنید.

</details>

<details lang="he" dir="rtl">
<summary>עברית</summary>

חלון אחד, העברות שמחזיקות דרך שרת הממסר, ועצירה שעוצרת באמת.

- **חלון אחד:** פתיחת קבצים עם TotalDrop כשהוא כבר פתוח מוסיפה אותם לחלון הפתוח במקום לפתוח חלון שני, והחלון עולה לחזית.
- **דרך הממסר:** העברה דרך הממסר מתחברת גם כשהממסר מתחיל לאט, במקום להיכשל אחרי 20 שניות, והחיבור נוצר מחדש כשהממסר סוגר אותו.
- **אין עצירה אחרי הקוד:** העברה כבר לא נעצרת לפעמים מיד אחרי הזנת הקוד עם ההודעה "המכשיר השני עזב".
- **עצירה עוצרת:** עצירה במכשיר השולח קוטעת את ההעברה מיד; קבצים שכבר היו בדרך לא נשמרים עוד במלואם במכשיר המקבל.
- **Android:** ההעברה ממשיכה כשעוברים לאפליקציה אחרת, למשל כדי לשתף את הקוד, והתראה עם כפתור עצירה מציגה אותה.

להעברות עם טלפון, עדכנו גם את אפליקציית Android לגרסה 0.3.0-beta.3.

</details>

<details lang="ur" dir="rtl">
<summary>اردو</summary>

ایک ہی ونڈو، ریلے کے ذریعے قائم رہنے والی منتقلی، اور ایسا "روکیں" جو واقعی روکتا ہے۔

- **ایک ونڈو:** TotalDrop پہلے سے کھلا ہو اور آپ اس سے فائلیں کھولیں تو دوسری ونڈو کھلنے کے بجائے وہ کھلی ونڈو میں شامل ہو جاتی ہیں، اور ونڈو سامنے آ جاتی ہے۔
- **ریلے کے ذریعے:** ریلے کے ذریعے منتقلی اس وقت بھی جڑ جاتی ہے جب ریلے دیر سے شروع ہو، 20 سیکنڈ بعد ناکام ہونے کے بجائے، اور ریلے کنکشن بند کرے تو دوبارہ جڑ جاتی ہے۔
- **کوڈ کے بعد رکاوٹ نہیں:** کوڈ درج کرنے کے فوراً بعد منتقلی اب کبھی کبھی "دوسرا آلہ چلا گیا" کے ساتھ نہیں رکتی۔
- **روکیں واقعی روکتا ہے:** بھیجنے والے آلے پر "روکیں" منتقلی کو فوراً کاٹ دیتا ہے؛ راستے میں موجود فائلیں اب وصول کرنے والے آلے پر پوری محفوظ نہیں رہتیں۔
- **Android:** کسی دوسری ایپ پر جانے پر، مثلاً کوڈ شیئر کرنے کے لیے، منتقلی جاری رہتی ہے، اور "روکیں" بٹن والی ایک اطلاع اسے دکھاتی ہے۔

فون کے ساتھ منتقلی کے لیے Android ایپ کو بھی 0.3.0-beta.3 پر اپ ڈیٹ کریں۔

</details>
