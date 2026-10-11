import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Save } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { PageHeader } from "@/components/shell/PageHeader";
import { Button } from "@/components/ui/button";
import { Field, Input } from "@/components/ui/input";
import { Segmented } from "@/components/ui/segmented";
import { Select } from "@/components/ui/select";
import { BrushPanel } from "@/features/pixel/BrushPanel";
import { PixelCanvas } from "@/features/pixel/PixelCanvas";
import { useBrush } from "@/features/pixel/useBrush";
import { errorMessage } from "@/lib/tauri/errors";
import { applySkin, listSkins, saveSkinPng, type SavedSkin, type SkinVariant } from "@/lib/tauri/skins";
import { SKINS_KEY, useActiveAccount, useSetProfile } from "../useProfile";
import { drawSkinImage, faceRects, loadImage, partLabels } from "./layout";
import { ModelPane, type Layer } from "./ModelPane";
import { SkinDocument } from "./SkinDocument";

const BLANK = "blank";
const VIEW_SIZE = 768;
const SMALL_VIEW_SIZE = 256;

type Mode = "texture" | "model";

function drawLabel(ctx: CanvasRenderingContext2D, text: string, centerX: number, centerY: number, fontSize: number) {
  const [first, ...rest] = text.split(" ");
  const lines = rest.length > 0 ? [first, rest.join(" ")] : [first];
  ctx.font = `600 ${fontSize}px Inter, sans-serif`;
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  lines.forEach((line, index) => {
    const y = centerY + (index - (lines.length - 1) / 2) * fontSize * 1.15;
    ctx.fillStyle = "rgba(0, 0, 0, 0.7)";
    ctx.fillText(line, centerX + 1, y + 1);
    ctx.fillStyle = "rgba(255, 255, 255, 0.85)";
    ctx.fillText(line, centerX, y);
  });
}

export function SkinEditorPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const setProfile = useSetProfile();
  const { active } = useActiveAccount();
  const [params] = useSearchParams();
  const [doc] = useState(() => new SkinDocument());
  const brush = useBrush(doc);
  const skins = useQuery({ queryKey: SKINS_KEY, queryFn: listSkins });

  const [mode, setMode] = useState<Mode>("texture");
  const [layer, setLayer] = useState<Layer>("base");
  const [showOverlay, setShowOverlay] = useState(true);
  const [showLabels, setShowLabels] = useState(true);
  const [variant, setVariant] = useState<SkinVariant>(doc.variant);
  const [name, setName] = useState("");
  const [source, setSource] = useState(BLANK);

  const guides = useCallback(
    (ctx: CanvasRenderingContext2D, scale: number) => {
      const size = doc.width * scale;
      ctx.fillStyle = "rgba(0, 0, 0, 0.6)";
      ctx.fillRect(0, 0, size, size);
      const rects = faceRects(doc.variant);
      for (const { x, y, w, h } of rects) ctx.clearRect(x * scale, y * scale, w * scale, h * scale);
      ctx.strokeStyle = "rgba(255, 255, 255, 0.35)";
      ctx.lineWidth = 1;
      for (const { x, y, w, h } of rects) ctx.strokeRect(x * scale + 0.5, y * scale + 0.5, w * scale - 1, h * scale - 1);
      if (!showLabels) return;
      for (const { part, overlay, rect } of partLabels(doc.variant)) {
        const label = overlay ? `${t(`editor.parts.${part}`)} 2` : t(`editor.parts.${part}`);
        drawLabel(ctx, label, (rect.x + rect.w / 2) * scale, (rect.y + rect.h / 2) * scale, Math.min(scale * 1.1, 14));
      }
    },
    [doc, showLabels, t],
  );

  const startFrom = useCallback(
    async (id: string, library: SavedSkin[]) => {
      setSource(id);
      const skin = library.find((item) => item.id === id);
      if (!skin) {
        doc.startFromTemplate(doc.variant);
        return;
      }
      const image = await loadImage(skin.dataUrl);
      doc.replaceSkin((ctx) => drawSkinImage(ctx, image), skin.variant);
      setVariant(skin.variant);
      setName(`${skin.name} (edit)`);
    },
    [doc],
  );

  const requested = params.get("skin");
  useEffect(() => {
    if (requested && skins.data) void startFrom(requested, skins.data).catch(() => undefined);
  }, [requested, skins.data, startFrom]);

  const changeVariant = (next: SkinVariant) => {
    doc.setVariant(next);
    setVariant(next);
  };

  const save = useMutation({
    mutationFn: async (apply: boolean) => {
      const saved = await saveSkinPng(name.trim(), doc.variant, doc.toPngBase64());
      await queryClient.invalidateQueries({ queryKey: SKINS_KEY });
      if (apply) setProfile(await applySkin(saved.id));
      return { saved, apply };
    },
    onSuccess: ({ saved, apply }) => toast.success(t(apply ? "skins.applied" : "skins.added", { name: saved.name })),
    onError: (error) => toast.error(errorMessage(t, error)),
  });

  const ready = name.trim().length > 0 && !save.isPending;

  return (
    <>
      <div className="flex items-start justify-between gap-4">
        <PageHeader title={t("editor.title")} subtitle={t("editor.subtitle")} />
        <Button variant="ghost" asChild>
          <Link to="/">
            <ArrowLeft size={16} />
            {t("editor.back")}
          </Link>
        </Button>
      </div>

      <div className="mb-4 max-w-sm">
        <Segmented
          value={mode}
          onChange={setMode}
          options={[
            { value: "texture", label: t("editor.modeTexture") },
            { value: "model", label: t("editor.modeModel") },
          ]}
        />
      </div>

      <div className="grid gap-4 lg:grid-cols-[14rem_minmax(0,1fr)_minmax(0,19rem)]">
        <aside className="panel space-y-5 p-4">
          <BrushPanel doc={doc} brush={brush} />
          {mode === "model" && (
            <Field label={t("editor.layer")} hint={t("editor.layerHint")}>
              <Segmented
                value={layer}
                onChange={setLayer}
                options={[
                  { value: "base", label: t("editor.layerBase") },
                  { value: "overlay", label: t("editor.layerOverlay") },
                ]}
              />
            </Field>
          )}
        </aside>

        {mode === "texture" ? (
          <section className="space-y-3">
            <PixelCanvas doc={doc} brush={brush.brush} onPick={brush.pickColor} guides={guides} viewSize={VIEW_SIZE} />
            <p className="text-center text-xs text-muted">{t("editor.overlayHint")}</p>
            <label className="flex items-center justify-center gap-2 text-sm text-muted">
              <input type="checkbox" checked={showLabels} onChange={(event) => setShowLabels(event.target.checked)} />
              {t("editor.showLabels")}
            </label>
          </section>
        ) : (
          <section className="panel relative min-h-[36rem] overflow-hidden">
            <ModelPane doc={doc} showOverlay={showOverlay} paint={{ brush: brush.brush, layer, onPick: brush.pickColor }} />
            <p className="pointer-events-none absolute inset-x-0 bottom-2 text-center text-xs text-muted">{t("editor.modelHint")}</p>
          </section>
        )}

        <aside className="space-y-4">
          {mode === "texture" ? (
            <div className="panel relative h-80 overflow-hidden">
              <ModelPane doc={doc} showOverlay={showOverlay} />
              <p className="pointer-events-none absolute inset-x-0 bottom-2 text-center text-xs text-muted">{t("editor.previewHint")}</p>
            </div>
          ) : (
            <PixelCanvas doc={doc} brush={brush.brush} onPick={brush.pickColor} guides={guides} viewSize={SMALL_VIEW_SIZE} />
          )}
          <label className="flex items-center gap-2 px-1 text-sm text-muted">
            <input type="checkbox" checked={showOverlay} onChange={(event) => setShowOverlay(event.target.checked)} />
            {t("editor.showOverlay")}
          </label>
          <div className="panel space-y-4 p-4">
            <Field label={t("editor.model")} hint={t("skins.modelHint")}>
              <Segmented
                value={variant}
                onChange={changeVariant}
                options={(["classic", "slim"] as SkinVariant[]).map((value) => ({ value, label: t(`skins.variant.${value}`) }))}
              />
            </Field>
            <Field label={t("editor.startFrom")}>
              <Select
                className="w-full"
                value={source}
                onChange={(event) => void startFrom(event.target.value, skins.data ?? []).catch(() => undefined)}
              >
                <option value={BLANK}>{t("editor.blank")}</option>
                {skins.data?.map((skin) => (
                  <option key={skin.id} value={skin.id}>
                    {skin.name}
                  </option>
                ))}
              </Select>
            </Field>
            <Field label={t("skins.name")}>
              <Input value={name} maxLength={32} placeholder={t("editor.namePlaceholder")} onChange={(event) => setName(event.target.value)} />
            </Field>
            <div className="grid gap-2">
              <Button disabled={!ready} onClick={() => save.mutate(false)}>
                <Save size={16} />
                {t("editor.save")}
              </Button>
              <Button
                variant="secondary"
                disabled={!ready || active === null}
                title={active === null ? t("home.signInToApply") : undefined}
                onClick={() => save.mutate(true)}
              >
                {t("editor.saveApply")}
              </Button>
            </div>
          </div>
        </aside>
      </div>
    </>
  );
}
