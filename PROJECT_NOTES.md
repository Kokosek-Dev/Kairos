# PROJECT_NOTES.md: wszystko o projekcie Kairos w jednym miejscu

Ten plik jest dla Ciebie i dla każdej nowej rozmowy z Claude. Nową rozmowę zacznij od zdania:

> Przeczytaj PROJECT_NOTES.md w D:\Dev\Minecraft Launcher i kontynuuj projekt Kairos.

## 1. Czym jest Kairos

Niezależny launcher do Minecraft: Java Edition. Backend: Rust + Tauri 2. Interfejs: React, TypeScript, Vite,
Tailwind 4. Licencja MIT. Nie jest związany z Mojang ani Microsoft (patrz `NOTICE.md`). Katalog projektu:
`D:\Dev\Minecraft Launcher`.

## 2. Zasady (od właściciela projektu, obowiązują zawsze)

- Rozmowa po polsku, krótko, konkretnie.
- Wszystko na dysku D:. Dysk C: ma mało miejsca, nie instalować tam niczego bez pytania.
- Tylko legalne konta Microsoft. **Żadnego trybu „non-premium"** ani obchodzenia licencji. Dozwolone: tryb offline dla
  właściciela konta i przełącznik testowy `KAIROS_DEV_SESSION=1` (tylko w buildzie debug, nie trafia do wydania).
- Bez nazw „Minecraft"/„Mojang" sugerujących oficjalność. Bez nazwy dostawcy katalogu modów w interfejsie.
- Kod czysty i otypowany, bez martwego kodu, minimum komentarzy.
- Po każdym skończonym kawałku pracy commit: `git -c user.name=Kairos -c user.email=Kokosek-Dev@users.noreply.github.com commit`.
- Nigdy nie prosić o dane karty płatniczej ani nie wpisywać cudzych identyfikatorów aplikacji Azure.

## 3. Jak uruchomić i sprawdzić

```
npm install
npm run tauri dev          # aplikacja deweloperska
npx tsc --noEmit           # sprawdzenie typów
cargo test                 # w katalogu src-tauri, ok. 115 testów
```

- **Przed `cargo test` zamknij aplikację dev** (równoległe budowanie wyczerpuje pamięć, błąd 1455 / crash rustc).
- Zmienne do testów: `KAIROS_DEV_SESSION=1` (gra bez logowania, tylko debug) oraz
  `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` (pozwala sterować oknem skryptem CDP i robić zrzuty).
- Dane launchera są teraz w `D:\Minecraft\Kairos\Kairos` (wybór w Ustawieniach, zapisany w
  `%APPDATA%\dev.kairos.launcher\location.json`).

## 4. Mapa kodu

Backend (`src-tauri/src`): `auth/` (logowanie Microsoft, Xbox, tokeny w Menedżerze poświadczeń), `launch/` (budowa
polecenia, start procesu, log), `mojang/` (wersje, biblioteki, zasoby), `loaders/` (Fabric, Quilt, Forge, NeoForge),
`modrinth/` (katalog modów, instalacja z zależnościami, .mrpack), `instances/` (instancje), `skins/` (skiny, peleryny,
biblioteka), `texturepacks/` (paczki tekstur, łączenie z instancjami, `options.txt`), `crash.rs` (analiza awarii),
`diagnostics.rs` (raport o problemie), `memory.rs` (podpowiedź RAM), `location.rs` (przenoszenie danych),
`commands/` (cienkie komendy Tauri), `java.rs`, `download.rs`, `state.rs`, `paths.rs`.

Frontend (`src`): `app/App.tsx` (trasy), `features/` (home, library, instances, discover, skins/editor, textures,
settings, launch, accounts, welcome, updates, pixel = wspólny edytor pikseli), `components/` (powłoka, tło, bohater
Mascot, ui), `lib/tauri/` (typowane wywołania backendu), `i18n/locales.ts` (EN i PL, każdy tekst w obu językach).

Dokumenty: `README.md`, `docs/PRIVACY.md`, `docs/RELEASING.md` (wydawanie, po polsku), `docs/AZURE_SETUP.md`
(rejestracja aplikacji Microsoft, po polsku), `docs/index.html` (strona pobierania na GitHub Pages).

## 5. Co jest gotowe

Etapy 1-8 (wersje, instancje, Java, loadery, logowanie, skiny, mody i modpacki, strona główna), a do tego: edytor
skinów (model 3D w środku i płaska tekstura), kreator paczek tekstur (beta) z podglądem, czas gry i statystyki,
automatyczna aktualizacja modów przed grą, tła i animowany bohater, wybór miejsca zapisu, podpowiedź RAM, analiza
crashy, raport o problemie, kreator pierwszego uruchomienia, aktualizacje launchera, GitHub Actions (CI i wydania),
paleta poleceń (Ctrl+K).

## 6. Co zostało (w tej kolejności)

Stan na koniec sesji: Azure czeka na odpowiedź szkoły (właściciel liczy na odpowiedź następnego dnia).

1. **Azure i logowanie Microsoft.** Bez Application (client) ID logowanie nie działa. ID wkleić do
   `src-tauri/.cargo/config.toml` (`KAIROS_MS_CLIENT_ID`). Potem wysłać formularz zatwierdzenia aplikacji do Mojang:
   https://aka.ms/mce-reviewappid (czeka się dni lub tygodnie, bez tego błąd 403). Właściciel próbuje przez konto
   szkolne albo dorosłego. Instrukcja: `docs/AZURE_SETUP.md`.
2. ~~Test instalatora NSIS~~ zrobiony (11.10.2026): cicha instalacja do wskazanego folderu, skróty na pulpicie i w menu Start, wpis do odinstalowania, start aplikacji, odinstalowanie i usunięcie śladów, dane użytkownika zostają. Nie sprawdzone: okno instalatora z wyborem języka (tryb interaktywny) oraz tłumaczenia komunikatów NSIS po polsku (Tauri ostrzega, że część jest po angielsku).
3. **Repozytorium na GitHubie.** W `src-tauri/tauri.conf.json` zamienić `OWNER/REPO` w adresie aktualizacji. Dodać
   sekrety `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (pusty), `KAIROS_MS_CLIENT_ID`.
   Włączyć Pages z folderu `/docs`. Szczegóły w `docs/RELEASING.md`.
4. Pełny przebieg testowy: utworzenie instancji, prawdziwy start gry w trybie testowym, czas gry, okno po awarii.
5. Opcjonalnie: przenieść Rusta (`.cargo`, `.rustup`, ok. 2,4 GB) i cache npm z C: na D:, odinstalować Azure CLI.
6. Na później: jasny motyw, Linux i macOS, podpisany instalator, własny przekaźnik do gry ze znajomymi.

## 7. Sekrety i ważne ścieżki

- Klucz podpisu aktualizacji: `D:\Dev\secrets\kairos-updater.key` z hasłem w `kairos-updater.password.txt` (prywatne, **zrób kopię obu, nigdy do repozytorium**).
  Klucz publiczny jest w `tauri.conf.json`. Bez klucza nie da się wydać aktualizacji dla zainstalowanych launcherów.
- `npm run tauri build` wymaga `TAURI_SIGNING_PRIVATE_KEY` (ścieżka do klucza) i `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (hasło). Puste hasło w PowerShell nie działa (zmienna znika i build czeka na hasło), dlatego klucz ma prawdziwe hasło.
- Instalator po zbudowaniu: `src-tauri/target/release/bundle/nsis/`.

- Logo i ikona: źródło to \ssets/icon-source.svg\ (K z iskrą). Zmiana: wyrenderuj do PNG 1024 jako \ssets/icon-source.png\ (np. Chrome headless), potem px tauri icon assets/icon-source.png\ i usuń wygenerowane foldery \src-tauri/icons/android\ i \ios\. Znak w aplikacji: \src/components/Logo.tsx\.

## 8. Pułapki, na które już się trafiło

- PowerShell 5.1 psuje cytowanie i kodowanie. Do edycji plików używać narzędzia Write albo skryptów Pythona w
  trybie `newline=""`. Pliki projektu mają końce linii CRLF.
- Nie uruchamiać dwóch kompilacji naraz (patrz sekcja 3).
- W Git Bash ścieżki typu `/settings` są zamieniane na ścieżki Windows. Ustawić `MSYS_NO_PATHCONV=1` albo użyć PowerShell.
- Pliki gry są duże. Wersji do testów (np. 1.21.1) nie zostawiać w katalogu danych po teście.
