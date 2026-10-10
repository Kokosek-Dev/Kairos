export const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "pl", label: "Polski" },
] as const;

export type LanguageCode = (typeof LANGUAGES)[number]["code"];

export const en = {
  nav: { library: "Library", versions: "Versions", discover: "Discover", settings: "Settings" },
  library: {
    title: "Library",
    subtitle: "Your instances will appear here.",
    emptyTitle: "No instances yet",
    emptyBody: "Create your first instance to start playing.",
  },
  versions: {
    title: "Game versions",
    subtitle: "Download official game versions from Mojang servers.",
    filter: { release: "Releases", snapshot: "Snapshots", old: "Old" },
    kind: { release: "Release", snapshot: "Snapshot", old_beta: "Beta", old_alpha: "Alpha" },
    search: "Search versions",
    install: "Install",
    play: "Play",
    installed: "Installed",
    cancel: "Cancel",
    resolving: "Preparing download...",
    progress: "{{done}} / {{total}} · {{files}} / {{totalFiles}} files",
    empty: "No versions match your search.",
    loadFailed: "Could not load the version list",
    loadFailedHint: "Check your internet connection and try again.",
    retry: "Try again",
  },
  console: {
    title: "Game console",
    step: { version: "Downloading game files", java: "Preparing Java", files: "Preparing files" },
    running: "Game is running",
    exited: "Game exited (code {{code}})",
    stop: "Stop",
    copy: "Copy log",
    clear: "Clear",
    close: "Hide console",
  },
  discover: {
    title: "Discover",
    subtitle: "Browse mods and modpacks from Modrinth.",
    comingSoon: "Coming in a later stage.",
  },
  settings: {
    title: "Settings",
    language: "Language",
    languageHint: "Choose the language of the launcher interface.",
  },
  welcome: {
    title: "Welcome to Zenith",
    body: "Choose your language. You can change it later in Settings.",
    continue: "Continue",
  },
  window: { minimize: "Minimize", maximize: "Maximize", close: "Close" },
};

export const pl: typeof en = {
  nav: { library: "Biblioteka", versions: "Wersje", discover: "Odkrywaj", settings: "Ustawienia" },
  library: {
    title: "Biblioteka",
    subtitle: "Tutaj pojawią się Twoje instancje.",
    emptyTitle: "Brak instancji",
    emptyBody: "Utwórz pierwszą instancję, aby zacząć grać.",
  },
  versions: {
    title: "Wersje gry",
    subtitle: "Pobieraj oficjalne wersje gry z serwerów Mojang.",
    filter: { release: "Wydania", snapshot: "Snapshoty", old: "Stare" },
    kind: { release: "Wydanie", snapshot: "Snapshot", old_beta: "Beta", old_alpha: "Alfa" },
    search: "Szukaj wersji",
    install: "Zainstaluj",
    play: "Graj",
    installed: "Zainstalowana",
    cancel: "Anuluj",
    resolving: "Przygotowywanie pobierania...",
    progress: "{{done}} / {{total}} · {{files}} / {{totalFiles}} plików",
    empty: "Żadna wersja nie pasuje do wyszukiwania.",
    loadFailed: "Nie udało się wczytać listy wersji",
    loadFailedHint: "Sprawdź połączenie z internetem i spróbuj ponownie.",
    retry: "Spróbuj ponownie",
  },
  console: {
    title: "Konsola gry",
    step: { version: "Pobieranie plików gry", java: "Przygotowywanie Javy", files: "Przygotowywanie plików" },
    running: "Gra działa",
    exited: "Gra zakończona (kod {{code}})",
    stop: "Zatrzymaj",
    copy: "Kopiuj log",
    clear: "Wyczyść",
    close: "Ukryj konsolę",
  },
  discover: {
    title: "Odkrywaj",
    subtitle: "Przeglądaj mody i paczki modów z Modrinth.",
    comingSoon: "Pojawi się w jednym z kolejnych etapów.",
  },
  settings: {
    title: "Ustawienia",
    language: "Język",
    languageHint: "Wybierz język interfejsu launchera.",
  },
  welcome: {
    title: "Witaj w Zenith",
    body: "Wybierz język. Możesz go później zmienić w Ustawieniach.",
    continue: "Dalej",
  },
  window: { minimize: "Minimalizuj", maximize: "Maksymalizuj", close: "Zamknij" },
};
