import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Download, FolderInput, RotateCcw, Save, Search } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";
import { useTranslation } from "react-i18next";
import { Link, useParams } from "react-router-dom";
import { toast } from "sonner";
import { PageHeader } from "@/components/shell/PageHeader";
import { Button } from "@/components/ui/button";
import { Field } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { useInstances } from "@/features/instances/useInstances";
import { BrushPanel } from "@/features/pixel/BrushPanel";
import { PixelCanvas } from "@/features/pixel/PixelCanvas";
import { PixelDocument } from "@/features/pixel/PixelDocument";
import { useBrush } from "@/features/pixel/useBrush";
import { loadImage } from "@/features/skins/editor/layout";
import { errorMessage } from "@/lib/tauri/errors";
import {
  exportTexturePack,
  installTexturePack,
  listPackTextures,
  readPackTexture,
  resetPackTexture,
  writePackTexture,
} from "@/lib/tauri/texturepacks";
import { cn } from "@/lib/utils";
import { textureKey, TextureThumb } from "./TextureThumb";
import { usePacks } from "./usePacks";

const PAGE_SIZE = 240;
const ALL = "all";

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
  const [instanceId, setInstanceId] = useState("");
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

  const install = useMutation({
    mutationFn: async () => {
      await persist(doc, selected);
      return installTexturePack(packId, instanceId);
    },
    onSuccess: () => {
      const name = instances.data?.find((item) => item.id === instanceId)?.name ?? "";
      toast.success(t("textures.installed", { name }));
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
    return (listing.data?.textures ?? []).filter(
      (path) => (category === ALL || categoryOf(path) === category) && (needle === "" || path.includes(needle)),
    );
  }, [listing.data, category, query]);

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
          <div className="grid flex-1 grid-cols-[repeat(auto-fill,minmax(4.25rem,1fr))] content-start gap-2 overflow-y-auto pr-1">
            {listing.isPending && Array.from({ length: 24 }, (_, index) => <div key={index} className="skeleton aspect-square rounded-lg" />)}
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
              <PixelCanvas key={selected} doc={doc} brush={brush.brush} onPick={brush.pickColor} />
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

          <div className="panel p-4">
            <BrushPanel doc={doc} brush={brush} />
          </div>

          <div className="panel space-y-3 p-4">
            <Field label={t("textures.addToInstance")} hint={t("textures.addHint")}>
              <div className="flex gap-2">
                <Select className="min-w-0 flex-1" value={instanceId} onChange={(event) => setInstanceId(event.target.value)}>
                  <option value="">{t("textures.chooseInstance")}</option>
                  {instances.data?.map((instance) => (
                    <option key={instance.id} value={instance.id}>
                      {instance.name}
                    </option>
                  ))}
                </Select>
                <Button variant="secondary" disabled={!instanceId || install.isPending} onClick={() => install.mutate()}>
                  <FolderInput size={16} />
                </Button>
              </div>
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
