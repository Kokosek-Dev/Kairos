import { Circle, Eraser, Minus, Pencil, PaintBucket, Pipette, Redo2, Square, Undo2, type LucideIcon } from "lucide-react";
import { useSyncExternalStore } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/input";
import { Segmented } from "@/components/ui/segmented";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import type { PixelDocument } from "./PixelDocument";
import type { Mirror, Tool } from "./tools";
import { MAX_BRUSH, TOOL_KEYS, type Brush } from "./useBrush";

const PALETTE = [
  "#000000", "#4b4b4b", "#9a9a9a", "#ffffff",
  "#7a1f1f", "#d93a3a", "#f08a2e", "#f2d233",
  "#7bc043", "#1f7a3a", "#2fb5a8", "#2f8fd8",
  "#3a3fa8", "#8a4bd0", "#e255a8", "#8a5a36",
];
const TOOLS: { tool: Tool; icon: LucideIcon }[] = [
  { tool: "pencil", icon: Pencil },
  { tool: "eraser", icon: Eraser },
  { tool: "fill", icon: PaintBucket },
  { tool: "picker", icon: Pipette },
];
const SHAPES: { tool: Tool; icon: LucideIcon }[] = [
  { tool: "line", icon: Minus },
  { tool: "rect", icon: Square },
  { tool: "ellipse", icon: Circle },
];
const MIRRORS: Mirror[] = ["none", "x", "y", "both"];
const SHORTCUT = Object.fromEntries(Object.entries(TOOL_KEYS).map(([key, tool]) => [tool, key.toUpperCase()]));

function Swatches({ colors, active, onChoose }: { colors: string[]; active?: string; onChoose: (color: string) => void }) {
  return (
    <div className="grid grid-cols-8 gap-1.5">
      {colors.map((swatch) => (
        <button
          key={swatch}
          type="button"
          aria-label={swatch}
          onClick={() => onChoose(swatch)}
          style={{ background: swatch }}
          className={cn("aspect-square rounded-md ring-1 ring-foreground/20 transition-transform hover:scale-110", active === swatch && "ring-2 ring-accent")}
        />
      ))}
    </div>
  );
}

/** Tools, undo and redo, brush size and color: everything shared by the pixel editors. `advanced` adds shapes, mirroring and bigger brushes. */
export function BrushPanel({ doc, brush: state, advanced = false }: { doc: PixelDocument; brush: Brush; advanced?: boolean }) {
  const { t } = useTranslation();
  useSyncExternalStore(doc.onHistory, () => doc.revision);
  const { brush, setTool, setSize, setMirror, setFilled, chooseColor, recent } = state;

  return (
    <div className="space-y-5">
      <div className={cn("grid gap-2", advanced ? "grid-cols-7" : "grid-cols-4")}>
        {(advanced ? [...TOOLS, ...SHAPES] : TOOLS).map(({ tool, icon: Icon }) => (
          <button
            key={tool}
            type="button"
            title={`${t(`editor.tool.${tool}`)} (${SHORTCUT[tool]})`}
            aria-label={t(`editor.tool.${tool}`)}
            onClick={() => setTool(tool)}
            className={cn(
              "grid h-11 place-items-center rounded-lg border text-muted transition-all hover:border-accent/50 hover:text-foreground",
              brush.tool === tool && "border-transparent bg-brand text-accent-foreground hover:text-accent-foreground",
            )}
          >
            <Icon size={18} />
          </button>
        ))}
      </div>
      <p className="-mt-3 text-xs text-muted">{t(`editor.toolHints.${brush.tool}`)}</p>

      <div className="grid grid-cols-2 gap-2">
        <Button variant="secondary" size="sm" disabled={!doc.canUndo} onClick={() => doc.undo()}>
          <Undo2 size={14} />
          {t("editor.undo")}
        </Button>
        <Button variant="secondary" size="sm" disabled={!doc.canRedo} onClick={() => doc.redo()}>
          <Redo2 size={14} />
          {t("editor.redo")}
        </Button>
      </div>

      <Field label={t("editor.size")}>
        {advanced ? (
          <label className="flex items-center gap-3">
            <input
              type="range"
              min={1}
              max={MAX_BRUSH}
              value={brush.size}
              aria-label={t("editor.size")}
              onChange={(event) => setSize(Number(event.target.value))}
              className="h-2 flex-1 cursor-pointer accent-accent"
            />
            <span className="w-12 text-right font-mono text-sm text-muted">{brush.size} px</span>
          </label>
        ) : (
          <Segmented value={brush.size} onChange={setSize} options={[1, 2, 3].map((value) => ({ value, label: `${value} px` }))} />
        )}
      </Field>

      {advanced && (
        <>
          <Field label={t("editor.mirror")} hint={t("editor.mirrorHint")}>
            <Segmented value={brush.mirror} onChange={setMirror} options={MIRRORS.map((value) => ({ value, label: t(`editor.mirrors.${value}`) }))} />
          </Field>
          <div className="flex items-center justify-between gap-3 text-sm">
            <span>{t("editor.filled")}</span>
            <Switch checked={brush.filled} onChange={setFilled} label={t("editor.filled")} />
          </div>
        </>
      )}

      <Field label={t("editor.color")}>
        <div className="space-y-3">
          <label className="flex items-center gap-3">
            <input
              type="color"
              value={brush.color}
              onChange={(event) => chooseColor(event.target.value)}
              className="size-10 cursor-pointer rounded-md border bg-transparent p-0.5"
            />
            <span className="font-mono text-sm uppercase text-muted">{brush.color}</span>
          </label>
          <Swatches colors={PALETTE} active={brush.color} onChoose={chooseColor} />
          {recent.length > 0 && <Swatches colors={recent} onChoose={chooseColor} />}
        </div>
      </Field>
    </div>
  );
}
