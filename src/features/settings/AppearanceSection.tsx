import { Check } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { ACCENT_COLORS, applyAccent, currentAccent } from "@/lib/accent";
import { cn } from "@/lib/utils";

export function AppearanceSection() {
  const { t } = useTranslation();
  const [hue, setHue] = useState(currentAccent);

  return (
    <section className="rounded-xl border bg-surface p-5">
      <h2 className="font-semibold">{t("appearance.title")}</h2>
      <p className="mb-4 mt-1 text-muted">{t("appearance.hint")}</p>
      <div className="flex flex-wrap gap-3">
        {ACCENT_COLORS.map((color) => (
          <button
            key={color.key}
            type="button"
            title={t(`appearance.colors.${color.key}`)}
            aria-label={t(`appearance.colors.${color.key}`)}
            onClick={() => {
              applyAccent(color.hue);
              setHue(color.hue);
            }}
            className={cn(
              "grid size-10 place-items-center rounded-full ring-offset-2 ring-offset-surface transition-all hover:scale-110",
              hue === color.hue && "ring-2 ring-foreground",
            )}
            style={{ background: `linear-gradient(135deg, oklch(0.75 0.17 ${color.hue}), oklch(0.55 0.21 ${color.hue + 6}))` }}
          >
            {hue === color.hue && <Check size={16} className="text-white" />}
          </button>
        ))}
      </div>
    </section>
  );
}
