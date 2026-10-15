import { FlipHorizontal, FlipVertical, Moon, RotateCw, Sun } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/input";
import { Segmented } from "@/components/ui/segmented";
import type { PixelDocument } from "@/features/pixel/PixelDocument";
import { canRotate, flip, resized, rotate, shade } from "@/features/pixel/transform";

const SIZES = [8, 16, 32, 64, 128, 256];
const SHADE_STEP = 0.12;

interface TextureToolsProps {
  doc: PixelDocument;
  /** The picture at its new size takes the place of the old one. */
  onResize: (next: PixelDocument) => void;
}

/** What can be done to the whole picture at once: turn it, make it lighter, give it another resolution. */
export function TextureTools({ doc, onResize }: TextureToolsProps) {
  const { t } = useTranslation();
  const sizes = [...new Set([...SIZES, doc.width])].sort((a, b) => a - b);

  const change = (width: number) => {
    if (width !== doc.width) onResize(resized(doc, width, Math.max(1, Math.round((width * doc.height) / doc.width))));
  };

  return (
    <div className="space-y-5">
      <Field label={t("editor.transform")}>
        <div className="grid gap-2">
          {[
            { label: t("editor.flipHorizontal"), icon: FlipVertical, run: () => flip(doc, "horizontal"), off: false },
            { label: t("editor.flipVertical"), icon: FlipHorizontal, run: () => flip(doc, "vertical"), off: false },
            { label: t("editor.rotate"), icon: RotateCw, run: () => rotate(doc), off: !canRotate(doc) },
            { label: t("editor.lighter"), icon: Sun, run: () => shade(doc, SHADE_STEP), off: false },
            { label: t("editor.darker"), icon: Moon, run: () => shade(doc, -SHADE_STEP), off: false },
          ].map(({ label, icon: Icon, run, off }) => (
            <Button key={label} variant="secondary" size="sm" className="justify-start" disabled={off} onClick={run}>
              <Icon size={15} />
              {label}
            </Button>
          ))}
        </div>
      </Field>

      <Field label={t("editor.resolution")} hint={t("editor.resolutionHint")}>
        <Segmented value={doc.width} onChange={change} options={sizes.map((value) => ({ value, label: `${value}` }))} />
      </Field>
    </div>
  );
}
