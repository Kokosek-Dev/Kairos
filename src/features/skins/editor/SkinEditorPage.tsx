import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Eraser, Pencil, PaintBucket, Pipette, Redo2, Save, Undo2, type LucideIcon } from "lucide-react";
import { useCallback, useEffect, useState, useSyncExternalStore } from "react";
import { useTranslation } from "react-i18next";
import { Link, useSearchParams } from "react-router-dom";
import { toast } from "sonner";
import { PageHeader } from "@/components/shell/PageHeader";
import { Button } from "@/components/ui/button";
import { Field, Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { errorMessage } from "@/lib/tauri/errors";
import { applySkin, listSkins, saveSkinPng, type SavedSkin, type SkinVariant } from "@/lib/tauri/skins";
import { cn } from "@/lib/utils";
import { SKINS_KEY, useActiveAccount, useSetProfile } from "../useProfile";
import { SkinDocument } from "./document";
import { FlatPane } from "./FlatPane";
import { drawSkinImage, loadImage } from "./layout";
import { ModelPane } from "./ModelPane";
import type { BrushSettings, Layer, Tool } from "./tools";

const BLANK = "blank";
const MAX_RECENT = 10;
const PALETTE = [
  "#000000", "#4b4b4b", "#9a9a9a", "#ffffff",
  "#7a1f1f", "#d93a3a", "#f08a2e", "#f2d233",
  "#7bc043", "#1f7a3a", "#2fb5a8", "#2f8fd8",
  "#3a3fa8", "#8a4bd0", "#e255a8", "#8a5a36",
];
const TOOLS: { tool: Tool; icon: LucideIcon; key: string }[] = [
  { tool: "pencil", icon: Pencil, key: "b" },
  { tool: "eraser", icon: Eraser, key: "e" },
  { tool: "fill", icon: PaintBucket, key: "g" },
  { tool: "picker", icon: Pipette, key: "i" },
];

function Segmented<T extends string | number>({
  value,
  options,
  onChange,
}: {
  value: T;
  options: { value: T; label: string }[];
  onChange: (value: T) => void;
}) {
  return (
    <div className="flex gap-1 rounded-lg border bg-surface/60 p-1">
      {options.map((option) => (
        <button
          key={option.value}
          type="button"
          onClick={() => onChange(option.value)}
          className={cn(
            "h-8 flex-1 rounded-md px-3 text-xs font-medium text-muted transition-colors hover:text-foreground",
            value === option.value && "bg-brand text-accent-foreground hover:text-accent-foreground",
          )}
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

export function SkinEditorPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const setProfile = useSetProfile();
  const { active } = useActiveAccount();
  const [params] = useSearchParams();
  const [doc] = useState(() => new SkinDocument());
  useSyncExternalStore(doc.onHistory, () => doc.revision);
  const skins = useQuery({ queryKey: SKINS_KEY, queryFn: listSkins });

  const [tool, setTool] = useState<Tool>("pencil");
  const [color, setColor] = useState("#2f8fd8");
  const [recent, setRecent] = useState<string[]>([]);
  const [size, setSize] = useState(1);
  const [layer, setLayer] = useState<Layer>("base");
  const [showOverlay, setShowOverlay] = useState(true);
  const [name, setName] = useState("");
  const [source, setSource] = useState(BLANK);

  const brush: BrushSettings = { tool, color, size };

  const chooseColor = useCallback((next: string) => {
    setColor(next);
    setRecent((previous) => [next, ...previous.filter((item) => item !== next)].slice(0, MAX_RECENT));
  }, []);

  const pickColor = useCallback(
    (next: string) => {
      chooseColor(next);
      setTool("pencil");
    },
    [chooseColor],
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
      doc.replace((ctx) => drawSkinImage(ctx, image), skin.variant);
      setName(`${skin.name} (edit)`);
    },
    [doc],
  );

  const requested = params.get("skin");
  useEffect(() => {
    if (requested && skins.data) void startFrom(requested, skins.data).catch(() => undefined);
  }, [requested, skins.data, startFrom]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement) return;
      const key = event.key.toLowerCase();
      if (event.ctrlKey && key === "z") {
        event.preventDefault();
        if (event.shiftKey) doc.redo();
        else doc.undo();
      } else if (event.ctrlKey && key === "y") {
        event.preventDefault();
        doc.redo();
      } else if (!event.ctrlKey && !event.altKey) {
        const match = TOOLS.find((item) => item.key === key);
        if (match) setTool(match.tool);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [doc]);

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

  const changeVariant = (variant: SkinVariant) => doc.setVariant(variant);
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
          <div className="grid grid-cols-4 gap-2">
            {TOOLS.map(({ tool: item, icon: Icon, key }) => (
              <button
                key={item}
                type="button"
                title={`${t(`editor.tool.${item}`)} (${key.toUpperCase()})`}
                aria-label={t(`editor.tool.${item}`)}
                onClick={() => setTool(item)}
                className={cn(
                  "grid h-11 place-items-center rounded-lg border text-muted transition-all hover:border-accent/50 hover:text-foreground",
                  tool === item && "border-transparent bg-brand text-accent-foreground hover:text-accent-foreground",
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
            <Segmented value={size} onChange={setSize} options={[1, 2, 3].map((value) => ({ value, label: `${value} px` }))} />
          </Field>

          <Field label={t("editor.color")}>
            <div className="space-y-3">
              <label className="flex items-center gap-3">
                <input
                  type="color"
                  value={color}
                  onChange={(event) => chooseColor(event.target.value)}
                  className="size-10 cursor-pointer rounded-md border bg-transparent p-0.5"
                />
                <span className="font-mono text-sm uppercase text-muted">{color}</span>
              </label>
              <div className="grid grid-cols-8 gap-1.5">
                {PALETTE.map((swatch) => (
                  <button
                    key={swatch}
                    type="button"
                    aria-label={swatch}
                    onClick={() => chooseColor(swatch)}
                    style={{ background: swatch }}
                    className={cn("aspect-square rounded-md ring-1 ring-white/15 transition-transform hover:scale-110", color === swatch && "ring-2 ring-accent")}
                  />
                ))}
              </div>
              {recent.length > 0 && (
                <div className="grid grid-cols-8 gap-1.5">
                  {recent.map((swatch) => (
                    <button
                      key={swatch}
                      type="button"
                      aria-label={swatch}
                      onClick={() => chooseColor(swatch)}
                      style={{ background: swatch }}
                      className="aspect-square rounded-md ring-1 ring-white/15 transition-transform hover:scale-110"
                    />
                  ))}
                </div>
              )}
            </div>
          </Field>

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
                <input type="checkbox" checked={showOverlay || layer === "overlay"} disabled={layer === "overlay"} onChange={(event) => setShowOverlay(event.target.checked)} />
                {t("editor.showOverlay")}
              </label>
            </div>
          </Field>
        </aside>

        <section className="panel relative min-h-[34rem] overflow-hidden">
          <ModelPane doc={doc} brush={brush} layer={layer} showOverlay={showOverlay} onPick={pickColor} />
        </section>

        <aside className="space-y-4">
          <FlatPane doc={doc} brush={brush} onPick={pickColor} />
          <div className="panel space-y-4 p-4">
            <Field label={t("editor.model")} hint={t("skins.modelHint")}>
              <Segmented
                value={doc.variant}
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
