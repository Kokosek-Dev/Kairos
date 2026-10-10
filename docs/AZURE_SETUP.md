# Rejestracja aplikacji w Azure (logowanie Microsoft)

Zenith loguje przez konto Microsoft. Do tego potrzebuje własnej aplikacji zarejestrowanej w Azure (Microsoft Entra ID). Nie potrzebujesz sekretu klienta ani karty płatniczej do samej rejestracji, a launcher nigdy nie używa identyfikatora oficjalnego launchera Mojang.

Flow w launcherze: **Microsoft (device code)** → Xbox Live → XSTS → Minecraft Services → sprawdzenie profilu (konto bez gry nie ma profilu).

## 1. Zarejestruj aplikację

1. Wejdź na <https://entra.microsoft.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade> (albo portal.azure.com → *Microsoft Entra ID* → *App registrations*) i zaloguj się swoim kontem Microsoft. Jeśli portal poprosi o utworzenie bezpłatnego katalogu (tenanta), zgódź się.
2. Kliknij **New registration** i ustaw:
   - **Name:** np. `Zenith` (ta nazwa pokaże się użytkownikom przy logowaniu; nie używaj w niej słów „Minecraft" ani „Mojang").
   - **Supported account types:** **Personal Microsoft accounts only**. Launcher używa endpointu `consumers`, a konta służbowe/szkolne nie mogą się tak zalogować.
   - **Redirect URI:** zostaw puste. Device code flow nie potrzebuje przekierowania.
3. Kliknij **Register**. Na stronie *Overview* skopiuj **Application (client) ID**. To jest ten identyfikator, którego potrzebuje Zenith.

## 2. Włącz flow dla klientów publicznych

*Authentication* → *Advanced settings* → **Allow public client flows: Yes** → **Save**. Bez tego logowanie kodem urządzenia zwróci błąd.

## 3. Uprawnienia (API permissions)

Nic nie dodajesz. Zakres `XboxLive.signin` jest żądany podczas logowania i zatwierdzany przez użytkownika na ekranie zgody. Domyślne `User.Read` może zostać.

## 4. Zgoda na użycie API Minecrafta (wymagana)

Nowe aplikacje Azure nie mają domyślnie dostępu do `api.minecraftservices.com`. Bez zgody logowanie przejdzie przez Microsoft, Xbox i XSTS, a zatrzyma się na kroku „Minecraft" z błędem HTTP 403 (`Invalid app registration`).

- Wyślij formularz: <https://aka.ms/mce-reviewappid> (wpisz w nim Application (client) ID z kroku 1).
- Starsze aplikacje (utworzone przed wprowadzeniem tego wymogu) działają bez zgody.
- Nie potrafię wskazać oficjalnego czasu rozpatrzenia ani kryteriów: formularz renderuje się w JavaScript i nie dało się go odczytać automatycznie. Źródła społecznościowe wspominają też o rejestracji w programie deweloperskim Xbox, ale to niepotwierdzone. Zapytaj w formularzu lub na stronie pomocy Microsoft.

## 5. Podaj identyfikator w Zenith

Do wyboru (w tej kolejności pierwszeństwa):

1. Zmienna środowiskowa `ZENITH_MS_CLIENT_ID` (w czasie działania).
2. **Ustawienia → Konta → Identyfikator aplikacji Microsoft** (zapis w `settings.json`).
3. Zmienna `ZENITH_MS_CLIENT_ID` ustawiona podczas budowania (wbudowana w plik wykonywalny).

Wydając własny launcher, wbuduj swój zatwierdzony identyfikator (opcja 3). Każdy wydawca potrzebuje własnej, zatwierdzonej aplikacji.

## Przechowywanie danych

- Zapisywany jest wyłącznie **refresh token**, w systemowym magazynie poświadczeń (Windows Credential Manager, macOS Keychain, Secret Service na Linuksie). Metadane kont (nazwa, UUID) są w `accounts.json` bez żadnych tokenów.
- Token dostępu do Minecrafta żyje tylko w pamięci i jest odnawiany przy starcie gry.

## Tryb deweloperski

W buildach debug można uruchomić grę bez logowania, ustawiając `ZENITH_DEV_SESSION=1`. Gra dostaje tymczasową tożsamość, która nie połączy się z serwerami online. Ten kod nie istnieje w buildach release.

## Najczęstsze błędy

| Objaw | Przyczyna |
| --- | --- |
| Błąd po kroku „Minecraft" (403) | Aplikacja nie ma jeszcze zgody z kroku 4 |
| Błąd zaraz po uruchomieniu logowania | Brak „Allow public client flows" lub zły typ kont |
| „Konto nie ma profilu Xbox" (XErr 2148916233) | Utwórz profil na xbox.com |
| Błąd kraju (2148916235) | Xbox Live niedostępny w kraju konta |
| Konto dziecięce (2148916238) | Dorosły musi dodać je do rodziny Microsoft |
| Konto ma grę, ale brak profilu | Zaloguj się raz oficjalnym launcherem i ustaw nazwę gracza (dotyczy m.in. Game Pass) |
