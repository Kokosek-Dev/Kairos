export const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "pl", label: "Polski" },
] as const;

export type LanguageCode = (typeof LANGUAGES)[number]["code"];

export const en = {
  nav: { library: "Library", discover: "Discover", settings: "Settings" },
  library: {
    title: "Library",
    subtitle: "Your instances will appear here.",
    emptyTitle: "No instances yet",
    emptyBody: "Create your first instance to start playing.",
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
    title: "Welcome to Voidlight",
    body: "Choose your language. You can change it later in Settings.",
    continue: "Continue",
  },
  window: { minimize: "Minimize", maximize: "Maximize", close: "Close" },
};

export const pl: typeof en = {
  nav: { library: "Biblioteka", discover: "Odkrywaj", settings: "Ustawienia" },
  library: {
    title: "Biblioteka",
    subtitle: "Tutaj pojawią się Twoje instancje.",
    emptyTitle: "Brak instancji",
    emptyBody: "Utwórz pierwszą instancję, aby zacząć grać.",
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
    title: "Witaj w Voidlight",
    body: "Wybierz język. Możesz go później zmienić w Ustawieniach.",
    continue: "Dalej",
  },
  window: { minimize: "Minimalizuj", maximize: "Maksymalizuj", close: "Zamknij" },
};
