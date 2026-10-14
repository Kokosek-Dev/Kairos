# Opinie i zgłoszenia błędów

W launcherze jest panel **Wyślij opinię**: w Ustawieniach (Pomoc), w oknie po crashu, w Ctrl+K i pod ikoną w lewym dolnym rogu.
Gracz wybiera Problem, Pomysł albo Inne, pisze wiadomość, opcjonalnie zostawia kontakt i może dołączyć szczegóły
techniczne (system, instancje, lista modów). Szczegóły może wcześniej przeczytać, nazwa użytkownika i dane logowania są z nich usuwane.

## Jak wiadomości do Ciebie docierają

Przez usługę formularzy **Web3Forms**: launcher wysyła wiadomość, a usługa wysyła ją jako mail na Twoją skrzynkę.
Twój adres e-mail nie jest nigdzie w launcherze ani w kodzie. W launcherze jest tylko publiczny klucz formularza
(według dokumentacji Web3Forms to nie jest sekret). Darmowy plan to 250 wiadomości miesięcznie.
Sprawdź na https://web3forms.com, czy ich regulamin dopuszcza taki użytek, zanim polegasz na tym na dłużej.

Zastrzeżenie: ktoś, kto wyciągnie klucz z launchera, może wysyłać na Twoją skrzynkę do limitu miesięcznego.
Dlatego launcher dopuszcza jedną wiadomość na 20 sekund i usługa ma własny filtr spamu.

## Ustawienie (raz)

1. Wejdź na https://web3forms.com, wpisz adres, na który mają przychodzić wiadomości, i potwierdź. Klucz przyjdzie mailem.
2. Dodaj klucz w repozytorium: Settings, Secrets and variables, Actions, nowy sekret `KAIROS_FEEDBACK_KEY`.
   (Do testów lokalnych możesz wpisać go w `src-tauri/.cargo/config.toml` w `KAIROS_FEEDBACK_KEY` albo ustawić zmienną środowiskową o tej nazwie.)
3. Uruchom wydanie (wysłanie zmian na `main` albo ręcznie z zakładki Actions). Od tej wersji przycisk Wyślij działa.

Bez klucza launcher nie ukrywa panelu: pokazuje komunikat, że ta wersja nie wysyła sama, i pozwala skopiować raport
albo otworzyć gotowe zgłoszenie na GitHubie (https://github.com/Kokosek-Dev/Kairos/issues, wymaga konta GitHub).