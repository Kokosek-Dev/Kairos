import { Check, ImagePlus, Palette } from "lucide-react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ACCENT_COLORS, applyAccent, currentAccent } from "@/lib/accent";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { prepareBackground } from "./backgroundImage";
import { ThemePicker } from "./ThemePicker";
import { usePreferences, type BackgroundStyle } from "./preferences";

const BACKGROUNDS: BackgroundStyle[] = ["aurora", "calm", "plain", "image"];

export function AppearanceSection() {
  const { t } = useTranslation();
  const [hue, setHue] = useState(currentAccent);
  const { background, backgroundImage, backgroundDim, update } = usePreferences();
  const picker = useRef<HTMLInputElement>(null);
  const [imageFailed, setImageFailed] = useState(false);

  const chooseImage = async (file: File | undefined) => {
    if (!file) return;
    try {
      update({ background: "image", backgroundImage: await prepareBackground(file) });
      setImageFailed(false);
    } catch {
      setImageFailed(true);
    }
  };

  return (
    <section className="panel p-5">
      <h2 className="flex items-center gap-2 font-semibold">
        <Palette size={16} className="text-accent" />
        {t("appearance.title")}
      </h2>
      <p className="mt-1 text-muted">{t("appearance.hint")}</p>
      <h3 className="mb-3 mt-4 font-medium">{t("appearance.theme")}</h3>
      <ThemePicker />
      <h3 className="mb-3 mt-6 font-medium">{t("appearance.accent")}</h3>
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

      <h3 className="mb-3 mt-6 font-medium">{t("appearance.background")}</h3>
      <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
        {BACKGROUNDS.map((style) => (
          <button
            key={style}
            type="button"
            onClick={() => (style === "image" && !backgroundImage ? picker.current?.click() : update({ background: style }))}
            className={cn(
              "h-10 rounded-lg border text-sm font-medium text-muted transition-colors hover:border-accent/50 hover:text-foreground",
              background === style && "border-accent/60 bg-accent/15 text-foreground",
            )}
          >
            {t(`appearance.backgrounds.${style}`)}
          </button>
        ))}
      </div>
      <input ref={picker} type="file" accept="image/*" className="hidden" onChange={(event) => void chooseImage(event.target.files?.[0])} />
      {background === "image" && (
        <div className="mt-4 flex flex-wrap items-center gap-4">
          <Button variant="secondary" size="sm" onClick={() => picker.current?.click()}>
            <ImagePlus size={14} />
            {t("appearance.chooseImage")}
          </Button>
          <label className="flex items-center gap-3 text-sm text-muted">
            {t("appearance.dim")}
            <input type="range" min={0.2} max={0.9} step={0.05} value={backgroundDim} onChange={(event) => update({ backgroundDim: Number(event.target.value) })} />
          </label>
        </div>
      )}
      {imageFailed && <p className="mt-2 text-sm text-danger">{t("appearance.imageFailed")}</p>}
    </section>
  );
}
