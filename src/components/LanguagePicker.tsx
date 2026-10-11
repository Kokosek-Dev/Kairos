import { useTranslation } from "react-i18next";
import { LANGUAGES, type LanguageCode } from "@/i18n/locales";
import { setLanguage } from "@/i18n";
import { cn } from "@/lib/utils";

export function LanguagePicker() {
  const { i18n } = useTranslation();

  return (
    <div className="flex flex-wrap gap-2">
      {LANGUAGES.map(({ code, label }) => (
        <button
          key={code}
          type="button"
          onClick={() => setLanguage(code as LanguageCode)}
          className={cn(
            "h-10 rounded-md border px-5 font-medium transition-all",
            i18n.language === code
              ? "border-accent bg-accent/15 text-foreground shadow-glow"
              : "text-muted hover:bg-surface-raised hover:text-foreground",
          )}
        >
          {label}
        </button>
      ))}
    </div>
  );
}
