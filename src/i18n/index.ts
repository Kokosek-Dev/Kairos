import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { en, pl, type LanguageCode } from "./locales";

const STORAGE_KEY = "zenith.language";

export function getStoredLanguage(): LanguageCode | null {
  const value = localStorage.getItem(STORAGE_KEY);
  return value === "en" || value === "pl" ? value : null;
}

export function setLanguage(code: LanguageCode): void {
  localStorage.setItem(STORAGE_KEY, code);
  void i18n.changeLanguage(code);
}

function detectLanguage(): LanguageCode {
  return navigator.language.toLowerCase().startsWith("pl") ? "pl" : "en";
}

void i18n.use(initReactI18next).init({
  resources: { en: { translation: en }, pl: { translation: pl } },
  lng: getStoredLanguage() ?? detectLanguage(),
  fallbackLng: "en",
  interpolation: { escapeValue: false },
});

export default i18n;
