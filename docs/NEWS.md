# Newsy w launcherze

Po prawej stronie okna jest zwijany panel z newsami. Bez dodatkowej konfiguracji pokazuje trzy newsy
wbudowane w launcher. Żeby pokazywać własne, bez wydawania nowej wersji, podaj adres listy.

## Jak podłączyć własną listę

1. Wystaw plik JSON pod adresem `https://...` (np. plik w repozytorium na GitHubie, widok „Raw").
2. Wpisz ten adres w `src-tauri/.cargo/config.toml` w `KAIROS_NEWS_URL` (w buildach na GitHubie
   ustaw tę samą zmienną w `release.yml`).
3. Zbuduj launcher. Lista odświeża się co 30 minut. Gdy jest pusta albo nie da się jej pobrać,
   launcher pokazuje newsy wbudowane. W trybie Incognito nic się nie pobiera.

## Format

Lista obiektów, maksymalnie 12 sztuk, cały plik do 64 KB:

```json
[
  {
    "id": "wersja-0-2",
    "kind": "news",
    "date": "2026-10-12",
    "title": { "en": "Version 0.2", "pl": "Wersja 0.2" },
    "body": { "en": "A new Journal and a mod detective.", "pl": "Nowy Dziennik i detektyw modów." },
    "url": "https://example.com/changelog"
  }
]
```

- `id`: unikalny, do 64 znaków.
- `kind`: `news` (domyślnie) albo `sponsored`. Wpis `sponsored` dostaje etykietę „Sponsorowane" i jest
  to miejsce na reklamy. Reklamy trzeba wyraźnie oznaczać, dlatego etykieta jest wymuszona.
- `date`: `RRRR-MM-DD`, opcjonalnie.
- `title`, `body`: zwykły tekst albo obiekt z tekstem na język (`en`, `pl`, `de`, `es`, `fr`, `pt`, `ru`).
  Gdy brakuje języka użytkownika, pokazuje się `en`. Tytuł do 120 znaków, treść do 400.
- `url`: opcjonalnie, tylko `https`. Otwiera się w przeglądarce.

Wpisy z błędnym formatem są pomijane, reszta listy działa.
