import { Check } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { usePreferences, type ThemeChoice } from "./preferences";

const CHOICES: ThemeChoice[] = ["dark", "light", "system"];

const DARK = { page: "oklch(0.14 0.02 260)", card: "oklch(0.22 0.03 260)", line: "oklch(0.5 0.03 260)" };
const LIGHT = { page: "oklch(0.975 0.008 250)", card: "oklch(0.995 0.003 250)", line: "oklch(0.75 0.02 260)" };

/** A small drawing of the launcher in the given look, so the choice can be made by eye. */
function Preview({ choice }: { choice: ThemeChoice }) {
  const palette = choice === "light" ? LIGHT : DARK;
  return (
    <div
      className="relative h-20 overflow-hidden rounded-lg border"
      style={{ background: choice === "system" ? `linear-gradient(135deg, ${DARK.page} 50%, ${LIGHT.page} 50%)` : palette.page }}
    >
      {choice === "system" ? (
        <>
          <div className="absolute left-2 top-2 h-3 w-8 rounded" style={{ background: DARK.card }} />
          <div className="absolute bottom-2 right-2 h-3 w-8 rounded" style={{ background: LIGHT.card }} />
        </>
      ) : (
        <>
          <div className="absolute inset-y-0 left-0 w-1/4" style={{ background: palette.card }} />
          <div className="absolute left-[32%] top-3 h-2 w-1/3 rounded-full bg-accent" />
          <div className="absolute left-[32%] top-8 h-6 w-[58%] rounded" style={{ background: palette.card }} />
          <div className="absolute left-[32%] top-[3.6rem] h-1.5 w-1/2 rounded-full" style={{ background: palette.line }} />
        </>
      )}
    </div>
  );
}

export function ThemePicker() {
  const { t } = useTranslation();
  const theme = usePreferences((state) => state.theme);
  const update = usePreferences((state) => state.update);

  return (
    <div className="grid grid-cols-3 gap-3">
      {CHOICES.map((choice) => (
        <button
          key={choice}
          type="button"
          onClick={() => update({ theme: choice })}
          className={cn(
            "rounded-xl border p-2 text-left text-sm font-medium transition-all hover:border-accent/50",
            theme === choice ? "border-accent bg-accent/10 shadow-glow" : "text-muted hover:text-foreground",
          )}
        >
          <Preview choice={choice} />
          <span className="mt-2 flex items-center justify-between px-1">
            {t(`appearance.themes.${choice}`)}
            {theme === choice && <Check size={14} className="text-accent" />}
          </span>
        </button>
      ))}
    </div>
  );
}
