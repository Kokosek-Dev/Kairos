# Wydawanie Kairos

Wydania buduje GitHub Actions z kodu, więc każdy może sprawdzić, że instalator pochodzi z tego repozytorium.
Poniżej jest wszystko, co trzeba zrobić raz, i to, co robisz przy każdym wydaniu.

## Jednorazowo

1. **Repozytorium na GitHubie.** Załóż publiczne repozytorium (najlepiej z dorosłym, jeśli konto wymaga pełnoletności)
   i wyślij tam kod.
2. **Adres aktualizacji.** W `src-tauri/tauri.conf.json`, w `plugins.updater.endpoints`, zamień `OWNER/REPO` na
   nazwę swojego konta i repozytorium, np. `https://github.com/twojenick/kairos/releases/latest/download/latest.json`.
3. **Klucz podpisu aktualizacji.** Para kluczy już jest: prywatny `D:\Dev\secrets\kairos-updater.key` (hasło w `kairos-updater.password.txt` w tym samym folderze),
   publiczny jest wpisany w `tauri.conf.json`. W GitHubie, w Settings, Secrets and variables, Actions, dodaj sekrety:
   - `TAURI_SIGNING_PRIVATE_KEY`: cała zawartość pliku `kairos-updater.key`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`: zawartość pliku `D:\Dev\secrets\kairos-updater.password.txt`
   - `KAIROS_MS_CLIENT_ID`: Application (client) ID z Azure (patrz `AZURE_SETUP.md`)
   - `KAIROS_DISCORD_APP_ID` (opcjonalnie): numer aplikacji Discord do statusu „Playing Kairos" (patrz `DISCORD_SETUP.md`)
4. **Kopia klucza.** Zrób kopię `kairos-updater.key` **i pliku z hasłem** w bezpiecznym miejscu, poza repozytorium. Jeśli go stracisz, nie
   wydasz aktualizacji, które zainstalują się w już zainstalowanych launcherach. Nigdy nie wrzucaj go do repozytorium.
5. **Strona z przyciskiem „Pobierz".** W repozytorium: Settings, Pages, źródło: gałąź `main`, folder `/docs`.
   Strona sama znajdzie najnowsze wydanie.

## Przy każdej zmianie w kodzie

Nic nie robisz ręcznie. Każde wysłanie zmian na gałąź `main` uruchamia GitHub Actions, które:

1. nadaje wersję `0.1.<numer uruchomienia>` (w `package.json`, `Cargo.toml` i `tauri.conf.json` tylko na czas budowania, w repozytorium numer się nie zmienia),
2. buduje instalator, podpisuje paczkę aktualizacji i tworzy wydanie `v0.1.<numer>` z `latest.json` i `SHA256SUMS.txt`,
3. a zainstalowane launchery po kilku minutach (przy następnym sprawdzeniu aktualizacji) same pokazują i instalują nową wersję.

Zmiany tylko w plikach `.md` i w `docs/` nie wypuszczają wydania. Budowa trwa kilkanaście minut, postęp widać w zakładce Actions.
Gdy chcesz zmienić pierwsze dwie liczby wersji (np. `0.2`), zmień `0.1.` w `.github/workflows/release.yml`.
Uwaga: każde wysłanie na `main` trafia do wszystkich użytkowników, więc wysyłaj tam tylko to, co jest sprawdzone.
## Budowanie lokalnie

`npm run tauri build` podpisuje paczkę aktualizacji, więc wymaga klucza w zmiennej środowiskowej:

```
$env:TAURI_SIGNING_PRIVATE_KEY = "D:\Dev\secrets\kairos-updater.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = (Get-Content "D:\Dev\secrets\kairos-updater.password.txt" -Raw).Trim()
npm run tauri build
```

## Podpis instalatora (opcjonalnie, później)

Bez certyfikatu podpisu kodu Windows SmartScreen ostrzega o „nieznanym wydawcy". README to wyjaśnia. Gdy będzie
budżet albo projekt dostanie darmowy podpis dla open source, dodaje się go w kroku budowania w
`.github/workflows/release.yml`.
