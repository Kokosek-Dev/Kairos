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
import { drawSkinImage, faceRects, loadImage } from "./layout";
import { ModelPane, type Layer } from "./ModelPane";
import { SkinDocument } from "./SkinDocument";

const BLANK = "blank";

export function SkinEditorPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const setProfile = useSetProfile();
  const { active } = useActiveAccount();
  const [params] = useSearchParams();
  const [doc] = useState(() => new SkinDocument());
  const brush = useBrush(doc);
  const skins = useQuery({ queryKey: SKINS_KEY, queryFn: listSkins });

  const [layer, setLayer] = useState<Layer>("base");
  const [showOverlay, setShowOverlay] = useState(true);
  const [variant, setVariant] = useState<SkinVariant>(doc.variant);
  const [name, setName] = useState("");
  const [source, setSource] = useState(BLANK);

  const guides = useCallback(
    (ctx: CanvasRenderingContext2D, scale: number) => {
      const size = doc.width * scale;
      ctx.fillStyle = "rgba(0, 0, 0, 0.55)";
      ctx.fillRect(0, 0, size, size);
      const rects = faceRects(doc.variant);
      for (const { x, y, w, h } of rects) ctx.clearRect(x * scale, y * scale, w * scale, h * scale);
      ctx.strokeStyle = "rgba(255, 255, 255, 0.35)";
      ctx.lineWidth = 1;
      for (const { x, y, w, h } of rects) ctx.strokeRect(x * scale + 0.5, y * scale + 0.5, w * scale - 1, h * scale - 1);
    },
    [doc],
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

      <div className="grid gap-4 lg:grid-cols-[15rem_minmax(0,1fr)_minmax(0,22rem)]">
        <aside className="panel space-y-5 p-4">
          <BrushPanel doc={doc} brush={brush} />
          <Field label={t("editor.layer")} hint={t("editor.layerHint")}>
            <div className="space-y-2">
              <Segmented
                value={layer}
                onChange={setLayer}
                options={[
                  { value: "base", label: t("editor.layerBase") },
                  { value: "overlay", label: t("editor.layerOverlay") },
                ]}
              />
              <label className="flex items-center gap-2 text-sm text-muted">
                <input
                  type="checkbox"
                  checked={showOverlay || layer === "overlay"}
                  disabled={layer === "overlay"}
                  onChange={(event) => setShowOverlay(event.target.checked)}
                />
                {t("editor.showOverlay")}
              </label>
            </div>
          </Field>
        </aside>

        <section className="panel relative min-h-[34rem] overflow-hidden">
          <ModelPane doc={doc} brush={brush.brush} layer={layer} showOverlay={showOverlay} onPick={brush.pickColor} />
        </section>

        <aside className="space-y-4">
          <PixelCanvas doc={doc} brush={brush.brush} onPick={brush.pickColor} guides={guides} />
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
