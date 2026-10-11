import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Download, RotateCcw, Save, Search } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import { useTranslation } from "react-i18next";
import { Link, useParams } from "react-router-dom";
import { toast } from "sonner";
import { PageHeader } from "@/components/shell/PageHeader";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/input";
import { Segmented } from "@/components/ui/segmented";
import { useInstances } from "@/features/instances/useInstances";
import { BrushPanel } from "@/features/pixel/BrushPanel";
import { PixelCanvas } from "@/features/pixel/PixelCanvas";
import { PixelDocument } from "@/features/pixel/PixelDocument";
import { useBrush } from "@/features/pixel/useBrush";
import { loadImage } from "@/features/skins/editor/layout";
import { errorMessage } from "@/lib/tauri/errors";
import {
  exportTexturePack,
  listPackTextures,
  readPackTexture,
  resetPackTexture,
  setTexturePackInstances,
  writePackTexture,
} from "@/lib/tauri/texturepacks";
import { cn } from "@/lib/utils";
import { InstanceChecklist } from "./InstanceChecklist";
import { TexturePreview3D } from "./TexturePreview3D";
import { textureKey, TextureThumb } from "./TextureThumb";
import { PACKS_KEY } from "./usePacks";
import { usePacks } from "./usePacks";

const PAGE_SIZE = 240;
const ALL = "all";
const EDITOR_SIZE = 288;

type Sort = "name" | "edited";

function categoryOf(path: string): string {
  const [first] = path.split("/");
  return path.includes("/") ? first : "other";
}

const emptyDocument = new PixelDocument(16, 16);

export function TexturePackPage() {
  const { t } = useTranslation();
  const { packId = "" } = useParams();
  const queryClient = useQueryClient();
  const packs = usePacks();
  const instances = useInstances();
  const pack = packs.data?.find((item) => item.id === packId);
  const listing = useQuery({ queryKey: ["pack-textures", packId], queryFn: () => listPackTextures(packId) });

  const [doc, setDoc] = useState<PixelDocument>(emptyDocument);
  const [selected, setSelected] = useState<string | null>(null);
  const [category, setCategory] = useState(ALL);
  const [query, setQuery] = useState("");
  const [shown, setShown] = useState(PAGE_SIZE);
  const [sort, setSort] = useState<Sort>("name");
  const [editedOnly, setEditedOnly] = useState(false);
  const brush = useBrush(doc);
  useSyncExternalStore(doc.onHistory, () => doc.revision);
  const current = useRef({ doc, selected });
  useEffect(() => {
    current.current = { doc, selected };
  }, [doc, selected]);

  const refresh = useCallback(
    (path: string) =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: ["pack-textures", packId] }),
        queryClient.invalidateQueries({ queryKey: textureKey(packId, path) }),
      ]),
    [queryClient, packId],
  );

  const persist = useCallback(
    async (target: PixelDocument, path: string | null) => {
      if (!path || !target.dirty) return;
      await writePackTexture(packId, path, target.toPngBase64());
      target.markSaved();
      await refresh(path);
    },
    [packId, refresh],
  );

  useEffect(
    () => () => {
      void persist(current.current.doc, current.current.selected).catch(() => undefined);
    },
    [persist],
  );

  const open = async (path: string) => {
    try {
      await persist(doc, selected);
      const image = await loadImage(await readPackTexture(packId, path));
      const next = new PixelDocument(image.naturalWidth, image.naturalHeight);
      next.load((ctx) => ctx.drawImage(image, 0, 0));
      setDoc(next);
      setSelected(path);
    } catch (error) {
      toast.error(errorMessage(t, error));
    }
  };

  const save = useMutation({
    mutationFn: () => persist(doc, selected),
    onSuccess: () => toast.success(t("textures.saved")),
    onError: (error) => toast.error(errorMessage(t, error)),
  });

  const reset = useMutation({
    mutationFn: async (path: string) => {
      await resetPackTexture(packId, path);
      await refresh(path);
      return path;
    },
    onSuccess: async (path) => {
      const image = await loadImage(await readPackTexture(packId, path));
      doc.load((ctx) => ctx.drawImage(image, 0, 0));
    },
    onError: (error) => toast.error(errorMessage(t, error)),
  });

  const exportZip = useMutation({
    mutationFn: async () => {
      await persist(doc, selected);
      return exportTexturePack(packId);
    },
    onSuccess: (path) => path && toast.success(t("textures.exported", { path })),
    onError: (error) => toast.error(errorMessage(t, error)),
  });

  const useInInstances = useMutation({
    mutationFn: async (instanceIds: string[]) => {
      await persist(doc, selected);
      return setTexturePackInstances(packId, instanceIds);
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: PACKS_KEY });
      toast.success(t("textures.usingSaved"));
    },
    onError: (error) => toast.error(errorMessage(t, error)),
  });

  const edited = useMemo(() => new Set(listing.data?.edited ?? []), [listing.data]);
  const categories = useMemo(() => {
    const counts = new Map<string, number>();
    for (const path of listing.data?.textures ?? []) counts.set(categoryOf(path), (counts.get(categoryOf(path)) ?? 0) + 1);
    return [...counts.keys()].sort();
  }, [listing.data]);
  const matching = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const found = (listing.data?.textures ?? []).filter(
      (path) =>
        (category === ALL || categoryOf(path) === category) &&
        (needle === "" || path.includes(needle)) &&
        (!editedOnly || edited.has(path)),
    );
    return sort === "edited" ? [...found].sort((a, b) => Number(edited.has(b)) - Number(edited.has(a))) : found;
  }, [listing.data, category, query, editedOnly, sort, edited]);

  if (packs.data && !pack) {
    return <p className="py-12 text-center text-muted">{t("textures.missing")}</p>;
  }

  return (
    <>
      <div className="flex items-start justify-between gap-4">
        <PageHeader title={pack?.name ?? t("textures.title")} subtitle={pack ? t("textures.subtitle", { version: pack.gameVersion }) : undefined} />
        <Button variant="ghost" asChild>
          <Link to="/library">
            <ArrowLeft size={16} />
            {t("nav.library")}
          </Link>
        </Button>
      </div>

      <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,24rem)]">
        <section className="panel flex min-h-[32rem] flex-col p-4 lg:max-h-[calc(100vh-12rem)]">
          <div className="mb-3 flex flex-wrap items-center gap-2">
            <label className="flex h-9 min-w-40 flex-1 items-center gap-2 rounded-md border bg-surface px-3 focus-within:border-accent/60">
              <Search size={15} className="text-muted" />
              <input
                value={query}
                onChange={(event) => {
                  setQuery(event.target.value);
                  setShown(PAGE_SIZE);
                }}
                placeholder={t("textures.search")}
                className="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted"
              />
            </label>
            <span className="text-xs text-muted">{t("textures.editedCount", { count: edited.size })}</span>
          </div>
          <div className="mb-3 flex flex-wrap items-center gap-3">
            <label className="flex cursor-pointer items-center gap-2 text-sm text-muted">
              <input
                type="checkbox"
                checked={editedOnly}
                onChange={(event) => {
                  setEditedOnly(event.target.checked);
                  setShown(PAGE_SIZE);
                }}
              />
              {t("textures.editedOnly")}
            </label>
            <div className="ml-auto w-56">
              <Segmented
                value={sort}
                onChange={setSort}
                options={[
                  { value: "name", label: t("textures.sortName") },
                  { value: "edited", label: t("textures.sortEdited") },
                ]}
              />
            </div>
          </div>
          <div className="mb-3 flex flex-wrap gap-1.5">
            {[ALL, ...categories].map((key) => (
              <button
                key={key}
                type="button"
                onClick={() => {
                  setCategory(key);
                  setShown(PAGE_SIZE);
                }}
                className={cn(
                  "rounded-full border px-3 py-1 text-xs font-medium transition-colors hover:border-accent/50",
                  category === key ? "border-accent/60 bg-accent/15 text-foreground" : "text-muted",
                )}
              >
                {key === ALL ? t("library.allLoaders") : t(`textures.category.${key}`, { defaultValue: key })}
              </button>
            ))}
          </div>
          <div className="grid flex-1 grid-cols-[repeat(auto-fill,minmax(5.25rem,1fr))] content-start gap-2 overflow-y-auto pr-1">
            {listing.isPending && Array.from({ length: 24 }, (_, index) => <div key={index} className="skeleton h-24 rounded-lg" />)}
            {listing.data && matching.length === 0 && <p className="col-span-full py-10 text-center text-muted">{t("textures.noResults")}</p>}
            {matching.slice(0, shown).map((path) => (
              <TextureThumb key={path} packId={packId} path={path} selected={path === selected} edited={edited.has(path)} onSelect={() => void open(path)} />
            ))}
            {matching.length > shown && (
              <Button variant="secondary" size="sm" className="col-span-full" onClick={() => setShown((count) => count + PAGE_SIZE)}>
                {t("textures.showMore", { count: matching.length - shown })}
              </Button>
            )}
          </div>
        </section>

        <aside className="space-y-4">
          {selected ? (
            <div className="panel space-y-4 p-4">
              <div>
                <p className="truncate font-semibold" title={selected}>
                  {selected}
                </p>
                <p className="text-xs text-muted">
                  {doc.width}×{doc.height} px
                  {doc.dirty && ` · ${t("textures.unsaved")}`}
                </p>
              </div>
              <PixelCanvas key={selected} doc={doc} brush={brush.brush} onPick={brush.pickColor} viewSize={EDITOR_SIZE} />
              <div className="grid grid-cols-2 gap-2">
                <Button disabled={!doc.dirty || save.isPending} onClick={() => save.mutate()}>
                  <Save size={16} />
                  {t("textures.save")}
                </Button>
                <Button variant="secondary" disabled={!edited.has(selected) || reset.isPending} onClick={() => reset.mutate(selected)}>
                  <RotateCcw size={16} />
                  {t("textures.restore")}
                </Button>
              </div>
            </div>
          ) : (
            <div className="panel p-6 text-center text-muted">{t("textures.pick")}</div>
          )}

          {selected && (
            <div className="panel relative h-48 overflow-hidden">
              <TexturePreview3D doc={doc} block={selected.startsWith("block/")} />
              <p className="pointer-events-none absolute left-3 top-2 text-xs font-medium text-muted">{t("textures.preview")}</p>
              <p className="pointer-events-none absolute inset-x-0 bottom-2 text-center text-xs text-muted">{t("textures.previewHint")}</p>
            </div>
          )}

          <div className="panel p-4">
            <BrushPanel doc={doc} brush={brush} />
          </div>

          <div className="panel space-y-3 p-4">
            <Field label={t("textures.useIn")} hint={t("textures.useInHint")}>
              <InstanceChecklist
                instances={instances.data ?? []}
                gameVersion={pack?.gameVersion ?? ""}
                selected={pack?.instances ?? []}
                onChange={(ids) => useInInstances.mutate(ids)}
              />
            </Field>
            <Button variant="secondary" className="w-full" disabled={exportZip.isPending} onClick={() => exportZip.mutate()}>
              <Download size={16} />
              {t("textures.export")}
            </Button>
          </div>
        </aside>
      </div>
    </>
  );
}
