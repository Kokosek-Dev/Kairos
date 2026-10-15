import { ChevronLeft, ChevronRight, Minimize2, RotateCcw, Save } from "lucide-react";
import { useEffect, useState, useSyncExternalStore } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Segmented } from "@/components/ui/segmented";
import { BrushPanel } from "@/features/pixel/BrushPanel";
import type { PixelDocument } from "@/features/pixel/PixelDocument";
import { PixelCanvas } from "@/features/pixel/PixelCanvas";
import type { Brush } from "@/features/pixel/useBrush";
import { TexturePreview } from "./TexturePreview";
import { TextureTools } from "./TextureTools";

const SIDE_SPACE = 640;
const TOP_SPACE = 150;
const MIN_VIEW = 256;

interface TextureStudioProps {
  packId: string;
  path: string;
  doc: PixelDocument;
  brush: Brush;
  edited: boolean;
  busy: boolean;
  onSave: () => void;
  onRestore: () => void;
  onResize: (next: PixelDocument) => void;
  onStep: (direction: -1 | 1) => void;
  onClose: () => void;
}

function roomForCanvas(): number {
  return Math.max(MIN_VIEW, Math.min(window.innerWidth - SIDE_SPACE, window.innerHeight - TOP_SPACE));
}

/** The editor over the whole window: tools on the left, the picture as large as it fits, the shape in the game on the right. */
export function TextureStudio({ packId, path, doc, brush, edited, busy, onSave, onRestore, onResize, onStep, onClose }: TextureStudioProps) {
  const { t } = useTranslation();
  const [room, setRoom] = useState(roomForCanvas);
  const [pixel, setPixel] = useState<{ x: number; y: number } | null>(null);
  const [tab, setTab] = useState<"draw" | "picture">("draw");
  useSyncExternalStore(doc.onHistory, () => doc.revision);

  useEffect(() => {
    const onResizeWindow = () => setRoom(roomForCanvas());
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("resize", onResizeWindow);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("resize", onResizeWindow);
      window.removeEventListener("keydown", onKey);
    };
  }, [onClose]);

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-background">
      <header className="flex flex-wrap items-center gap-3 border-b px-4 py-2">
        <Button variant="ghost" size="sm" className="size-8 px-0" aria-label={t("textures.previousTexture")} title={t("textures.previousTexture")} onClick={() => onStep(-1)}>
          <ChevronLeft size={18} />
        </Button>
        <Button variant="ghost" size="sm" className="size-8 px-0" aria-label={t("textures.nextTexture")} title={t("textures.nextTexture")} onClick={() => onStep(1)}>
          <ChevronRight size={18} />
        </Button>
        <div className="min-w-0">
          <p className="truncate font-semibold" title={path}>
            {path}
          </p>
          <p className="text-xs text-muted">
            {doc.width}×{doc.height} px
            {doc.dirty && ` · ${t("textures.unsaved")}`}
            {pixel && ` · ${pixel.x}, ${pixel.y}`}
          </p>
        </div>
        <div className="ml-auto flex gap-2">
          <Button size="sm" disabled={!doc.dirty || busy} onClick={onSave}>
            <Save size={15} />
            {t("textures.save")}
          </Button>
          <Button size="sm" variant="secondary" disabled={!edited || busy} onClick={onRestore}>
            <RotateCcw size={15} />
            {t("textures.restore")}
          </Button>
          <Button size="sm" variant="ghost" onClick={onClose}>
            <Minimize2 size={15} />
            {t("textures.closeFullscreen")}
          </Button>
        </div>
      </header>

      <div className="grid min-h-0 flex-1 grid-cols-[19rem_minmax(0,1fr)_minmax(0,26rem)]">
        <aside className="space-y-6 overflow-y-auto overflow-x-hidden border-r p-4">
          <Segmented
            value={tab}
            onChange={setTab}
            options={[
              { value: "draw", label: t("textures.tabs.draw") },
              { value: "picture", label: t("textures.tabs.picture") },
            ]}
          />
          {tab === "draw" ? <BrushPanel doc={doc} brush={brush} advanced /> : <TextureTools doc={doc} onResize={onResize} />}
        </aside>
        <main className="grid place-items-center overflow-auto p-4">
          <PixelCanvas key={path} doc={doc} brush={brush.brush} onPick={brush.pickColor} viewSize={room} maxHeight={room} onHover={setPixel} />
        </main>
        <aside className="flex min-h-0 flex-col border-l p-4">
          <TexturePreview packId={packId} path={path} doc={doc} className="min-h-64 flex-1" />
        </aside>
      </div>
    </div>
  );
}
