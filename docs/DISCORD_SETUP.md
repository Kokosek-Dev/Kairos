# Discord Rich Presence

Kairos może pokazywać na Twoim profilu na Discordzie, w co grasz: nazwę instancji, wersję gry, loader i liczbę
modów oraz czas gry. Żeby to działało, launcher potrzebuje własnej aplikacji na Discordzie. To darmowe, nie wymaga
karty ani żadnej płatności.

## Jednorazowo

1. Wejdź na https://discord.com/developers/applications i zaloguj się swoim kontem Discord.
2. Kliknij **New Application** i nazwij ją **Kairos**. Ta nazwa pojawi się na profilu jako „Playing Kairos".
3. W zakładce **General Information** skopiuj **Application ID** (długi numer, np. 18-19 cyfr).
4. W zakładce **Rich Presence → Art Assets** wgraj obrazek i nadaj mu nazwę **kairos** (małymi literami).
   Dobry jest `assets/icon-source.png`. Obrazek pojawia się po kilku minutach.
5. Wklej numer do `src-tauri/.cargo/config.toml` w polu `KAIROS_DISCORD_APP_ID` i zbuduj launcher. Do wydań
   dodaj ten sam numer jako sekret `KAIROS_DISCORD_APP_ID` w GitHubie (to publiczny identyfikator, nie tajemnica).

Bez numeru wszystko działa normalnie, tylko przełącznik „Pokazuj na Discordzie" w Ustawieniach się nie pojawia.

## Zasady

- Status jest widoczny tylko podczas gry i znika, gdy gra się zamknie.
- W trybie incognito Kairos nigdy niczego nie wysyła do Discorda.
- Graczowi włączasz i wyłączasz to w Ustawieniach, w sekcji Zachowanie.
