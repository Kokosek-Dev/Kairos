# Opinie i zgłoszenia błędów

W launcherze jest panel **Wyślij opinię**: w Ustawieniach (Pomoc), w oknie po crashu, w Ctrl+K i pod ikoną w lewym dolnym rogu.
Gracz wybiera Problem, Pomysł albo Inne, pisze wiadomość, opcjonalnie zostawia kontakt i może dołączyć szczegóły
techniczne (system, instancje, lista modów). Szczegóły może wcześniej przeczytać, nazwa użytkownika i dane logowania są z nich usuwane.

## Jak wiadomości do Ciebie docierają

Przez formularz w usłudze **Formspree**: launcher wysyła wiadomość na adres formularza, a Formspree przesyła ją jako mail
na Twoją skrzynkę. Twój adres e-mail nie jest nigdzie w launcherze ani w kodzie, jest tylko identyfikator formularza
(`https://formspree.io/f/xxxxxxxx`), który z założenia jest publiczny. Darmowy plan ma limit wiadomości w miesiącu,
sprawdź go na https://formspree.io/plans.

Dlaczego nie Web3Forms: ich serwer odrzuca zgłoszenia, które nie przychodzą z przeglądarki (ochrona przed botami, błąd 403),
więc aplikacja desktopowa nie może z niego korzystać. Formspree, Formspark i Getform odpowiadają aplikacjom normalnie.

Zastrzeżenie: ktoś, kto wyciągnie adres formularza z launchera, może wysyłać na Twoją skrzynkę do limitu miesięcznego.
Dlatego launcher dopuszcza jedną wiadomość na 20 sekund, a Formspree ma własny filtr spamu. Gdyby się zdarzyło nadużycie,
usuń formularz i załóż nowy.

## Ustawienie (raz)

1. Wejdź na https://formspree.io, załóż konto na adres, na który mają przychodzić wiadomości, i potwierdź go.
2. Utwórz nowy formularz (New Form), nazwij go `Kairos`. Skopiuj jego adres, wygląda jak `https://formspree.io/f/xxxxxxxx`.
3. Dodaj adres w repozytorium: Settings, Secrets and variables, Actions, nowy sekret `KAIROS_FEEDBACK_URL`.
   (Do testów lokalnych możesz wpisać go w `src-tauri/.cargo/config.toml` w `KAIROS_FEEDBACK_URL` albo ustawić zmienną środowiskową o tej nazwie.)
4. Uruchom wydanie (wysłanie zmian na `main` albo ręcznie z zakładki Actions). Od tej wersji przycisk Wyślij działa.

Bez adresu launcher nie ukrywa panelu: pokazuje komunikat, że ta wersja nie wysyła sama, i pozwala skopiować raport
albo otworzyć gotowe zgłoszenie na GitHubie (https://github.com/Kokosek-Dev/Kairos/issues, wymaga konta GitHub).