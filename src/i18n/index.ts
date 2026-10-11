import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { en, LANGUAGES, pl, type LanguageCode } from "./locales";
import { de } from "./translations/de";
import { es } from "./translations/es";
import { fr } from "./translations/fr";
import { pt } from "./translations/pt";
import { ru } from "./translations/ru";

const STORAGE_KEY = "kairos.language";
const CODES: string[] = LANGUAGES.map((language) => language.code);

export function getStoredLanguage(): LanguageCode | null {
  const value = localStorage.getItem(STORAGE_KEY);
  return value !== null && CODES.includes(value) ? (value as LanguageCode) : null;
}

export function setLanguage(code: LanguageCode): void {
  localStorage.setItem(STORAGE_KEY, code);
  void i18n.changeLanguage(code);
}

/** The first language of the system that Kairos speaks, otherwise English. */
function detectLanguage(): LanguageCode {
  for (const tag of navigator.languages) {
    const code = tag.toLowerCase().split("-")[0];
    if (CODES.includes(code)) return code as LanguageCode;
  }
  return "en";
}

void i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    pl: { translation: pl },
    de: { translation: de },
    es: { translation: es },
    fr: { translation: fr },
    pt: { translation: pt },
    ru: { translation: ru },
  },
  lng: getStoredLanguage() ?? detectLanguage(),
  fallbackLng: "en",
  interpolation: { escapeValue: false },
});

export default i18n;
