import { Eraser, Pencil, PaintBucket, Pipette, Redo2, Undo2, type LucideIcon } from "lucide-react";
import { useSyncExternalStore } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/input";
import { Segmented } from "@/components/ui/segmented";
import { cn } from "@/lib/utils";
import type { PixelDocument } from "./PixelDocument";
import type { Tool } from "./tools";
import { TOOL_KEYS, type Brush } from "./useBrush";

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

/** Tools, undo and redo, brush size and color: everything shared by the pixel editors. */
export function BrushPanel({ doc, brush: state }: { doc: PixelDocument; brush: Brush }) {
  const { t } = useTranslation();
  useSyncExternalStore(doc.onHistory, () => doc.revision);
  const { brush, setTool, setSize, chooseColor, recent } = state;

  return (
    <div className="space-y-5">
      <div className="grid grid-cols-4 gap-2">
        {TOOLS.map(({ tool, icon: Icon }) => (
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
        <Segmented value={brush.size} onChange={setSize} options={[1, 2, 3].map((value) => ({ value, label: `${value} px` }))} />
      </Field>

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
