# Rejestracja aplikacji w Azure (logowanie Microsoft)

Kairos loguje przez konto Microsoft. Do tego potrzebuje własnej aplikacji zarejestrowanej w Azure (Microsoft Entra ID). Nie potrzebujesz sekretu klienta. Dokumentacja Microsoftu wymienia jako warunek wstępny konto Azure (można je założyć za darmo: <https://azure.microsoft.com/pricing/purchase-options/azure-account>) i dostęp do katalogu (np. *Default Directory*). Launcher nigdy nie używa identyfikatora oficjalnego launchera Mojang.

Flow w launcherze: **logowanie w przeglądarce (kod autoryzacji + PKCE, automatyczny powrót na lokalny adres)** → Xbox Live → XSTS → Minecraft Services → sprawdzenie profilu (konto bez gry nie ma profilu).

Dla użytkownika wygląda to tak: klik „Dodaj konto", otwiera się przeglądarka ze stroną logowania Microsoft, po zalogowaniu strona pokazuje „możesz zamknąć kartę", a launcher sam kończy logowanie.

## 1. Zarejestruj aplikację

1. Wejdź na <https://entra.microsoft.com> i zaloguj się swoim kontem Microsoft. Jeśli masz dostęp do kilku katalogów (tenantów), przełącz się ikoną ustawień u góry na właściwy.
2. Przejdź do **Entra ID** → **App registrations** i kliknij **New registration**.
3. Ustaw:
   - **Name:** np. `Kairos` (ta nazwa pokaże się użytkownikom przy logowaniu; nie używaj w niej słów „Minecraft" ani „Mojang").
   - **Supported account types:** **Personal accounts only**. Launcher używa endpointu `consumers`, a konta służbowe/szkolne nie mogą się tak zalogować.
   - **Redirect URI:** możesz zostawić puste i dodać w kroku 2.
4. Kliknij **Register**. Na stronie *Overview* skopiuj **Application (client) ID**. To jest ten identyfikator, którego potrzebuje Kairos.

## 2. Dodaj adres przekierowania i włącz klienta publicznego

Po zalogowaniu Microsoft odsyła przeglądarkę na lokalny adres launchera (`http://localhost:<losowy port>`). Microsoft ignoruje numer portu przy dopasowaniu adresu `localhost`, więc rejestrujesz go raz, bez portu:

1. *Authentication* → **Add a platform** (w nowszym interfejsie: **Add Redirect URI**) → **Mobile and desktop applications**.
2. W polu własnego adresu wpisz dokładnie `http://localhost` i zatwierdź (**Configure** / **Save**).
3. Przełącznik **Allow public client flows** ustaw na **Yes** i kliknij **Save** (w nowszym interfejsie bywa w zakładce *Settings* na stronie Authentication, w starszym w sekcji *Advanced settings*).

Bez poprawnego adresu przekierowania Microsoft zwróci błąd `AADSTS50011` (reply URL does not match).

## 3. Uprawnienia (API permissions)

Nic nie dodajesz. Zakres `XboxLive.signin` jest żądany podczas logowania i zatwierdzany przez użytkownika na ekranie zgody. Domyślne `User.Read` może zostać.

## 4. Zgoda na użycie API Minecrafta (wymagana)

Nowe aplikacje Azure nie mają domyślnie dostępu do `api.minecraftservices.com`. Bez zgody logowanie przejdzie przez Microsoft, Xbox i XSTS, a zatrzyma się na kroku „Minecraft" z błędem HTTP 403 (`Invalid app registration`).

- Wyślij formularz: <https://aka.ms/mce-reviewappid> (wpisz w nim Application (client) ID z kroku 1).
- Starsze aplikacje (utworzone przed wprowadzeniem tego wymogu) działają bez zgody.
- Nie potrafię wskazać oficjalnego czasu rozpatrzenia ani kryteriów: formularz renderuje się w JavaScript i nie dało się go odczytać automatycznie. Źródła społecznościowe wspominają też o rejestracji w programie deweloperskim Xbox, ale to niepotwierdzone. Zapytaj w formularzu lub na stronie pomocy Microsoft.

## 5. Wbuduj identyfikator w launcher

Identyfikator wpisujesz **raz, jako twórca launchera**. Gracze go nie widzą i nie muszą nic konfigurować: klikają „Zaloguj się", otwiera się strona Microsoft, a po zalogowaniu wracają do launchera.

Wklej Application (client) ID do pliku `src-tauri/.cargo/config.toml`:

```toml
[env]
KAIROS_MS_CLIENT_ID = { value = "00000000-0000-0000-0000-000000000000", force = false }
```

i zbuduj launcher (`npm run tauri dev` albo `npm run tauri build`). Identyfikator klienta nie jest sekretem, więc plik można commitować. Do szybkich testów można też ustawić zmienną środowiskową `KAIROS_MS_CLIENT_ID` przy uruchamianiu, ma ona pierwszeństwo.

Każdy, kto wydaje własny launcher, potrzebuje własnej, zatwierdzonej aplikacji.
## Przechowywanie danych

- Zapisywany jest wyłącznie **refresh token**, w systemowym magazynie poświadczeń (Windows Credential Manager, macOS Keychain, Secret Service na Linuksie). Metadane kont (nazwa, UUID) są w `accounts.json` bez żadnych tokenów.
- Token dostępu do Minecrafta żyje tylko w pamięci i jest odnawiany przy starcie gry.
- Lokalny serwer odbierający przekierowanie działa tylko na `127.0.0.1`, tylko podczas logowania, i akceptuje wyłącznie odpowiedź z poprawnym parametrem `state` (ochrona przed podstawionym żądaniem). Kod autoryzacji jest wymieniany z kodem PKCE, bez sekretu.

## Tryb deweloperski

W buildach debug można uruchomić grę bez logowania, ustawiając `KAIROS_DEV_SESSION=1`. Gra dostaje tymczasową tożsamość, która nie połączy się z serwerami online. Ten kod nie istnieje w buildach release.

## Najczęstsze błędy

| Objaw | Przyczyna |
| --- | --- |
| Błąd po kroku „Minecraft" (403) | Aplikacja nie ma jeszcze zgody z kroku 4 |
| Błąd `AADSTS50011` w przeglądarce | Brak adresu `http://localhost` w platformie *Mobile and desktop applications* |
| Inny błąd zaraz po otwarciu strony logowania | Brak „Allow public client flows" lub zły typ kont |
| „Konto nie ma profilu Xbox" (XErr 2148916233) | Utwórz profil na xbox.com |
| Błąd kraju (2148916235) | Xbox Live niedostępny w kraju konta |
| Konto dziecięce (2148916238) | Dorosły musi dodać je do rodziny Microsoft |
| Konto ma grę, ale brak profilu | Zaloguj się raz oficjalnym launcherem i ustaw nazwę gracza (dotyczy m.in. Game Pass) |
