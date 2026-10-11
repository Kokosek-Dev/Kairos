import { Boxes, FileUp, KeyRound, Plus, Search } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { PageHeader } from "@/components/shell/PageHeader";
import { Button } from "@/components/ui/button";
import { ImportCodeDialog } from "@/features/share/ImportCodeDialog";
import { useJobsStore } from "@/features/discover/jobsStore";
import { pickMrpackFile } from "@/lib/tauri/modrinth";
import { DeleteInstanceDialog } from "@/features/instances/DeleteInstanceDialog";
import { InstanceCard } from "@/features/instances/InstanceCard";
import { NewInstanceDialog } from "@/features/instances/NewInstanceDialog";
import { useInstances } from "@/features/instances/useInstances";
import { errorMessage } from "@/lib/tauri/errors";
import { LOADER_HUE, loaderChipStyle } from "@/lib/loaderColors";
import type { Instance, LoaderKind } from "@/lib/tauri/instances";
import { cn } from "@/lib/utils";

type SortKey = "recent" | "name" | "version";

const SORT_KEYS: SortKey[] = ["recent", "name", "version"];
const LOADERS = Object.keys(LOADER_HUE) as LoaderKind[];

function sortInstances(list: Instance[], key: SortKey): Instance[] {
  const sorted = [...list];
  if (key === "name") return sorted.sort((a, b) => a.name.localeCompare(b.name));
  if (key === "version") {
    return sorted.sort((a, b) => b.gameVersion.localeCompare(a.gameVersion, undefined, { numeric: true }));
  }
  return sorted.sort((a, b) => (b.lastPlayedAt ?? "").localeCompare(a.lastPlayedAt ?? ""));
}

export function LibraryPage() {
  const { t } = useTranslation();
  const instances = useInstances();
  const [creating, setCreating] = useState(false);
  const [importingCode, setImportingCode] = useState(false);
  const [deleting, setDeleting] = useState<Instance | null>(null);
  const { jobs, importPack: runImport } = useJobsStore();
  const importing = jobs["mrpack-import"] !== undefined;
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SortKey>("recent");
  const [loader, setLoader] = useState<LoaderKind | null>(null);

  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const matching = (instances.data ?? []).filter(
      (instance) =>
        (loader === null || instance.loader === loader) &&
        (needle === "" || `${instance.name} ${instance.gameVersion}`.toLowerCase().includes(needle)),
    );
    return sortInstances(matching, sort);
  }, [instances.data, query, sort, loader]);

  const importPack = async () => {
    const path = await pickMrpackFile();
    if (path) await runImport(t, path);
  };

  return (
    <>
      <div className="flex items-start justify-between gap-4">
        <PageHeader title={t("library.title")} subtitle={t("library.subtitle")} />
        <div className="flex gap-2">
          <Button variant="secondary" disabled={importing} onClick={() => void importPack()}>
            <FileUp size={16} />
            {t("content.importPack")}
          </Button>
          <Button variant="secondary" onClick={() => setImportingCode(true)}>
            <KeyRound size={16} />
            {t("share.importButton")}
          </Button>
          <Button onClick={() => setCreating(true)}>
            <Plus size={16} />
            {t("instances.new")}
          </Button>
        </div>
      </div>

      {importing && <p className="mb-4 text-sm text-muted">{t("content.importing")}</p>}
      {instances.isPending && (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(18rem,1fr))] gap-4">
          {Array.from({ length: 3 }, (_, index) => (
            <div key={index} className="skeleton h-[168px] rounded-xl" />
          ))}
        </div>
      )}

      {instances.isError && (
        <div className="rounded-lg border border-danger/40 bg-danger/10 p-5">
          <p className="mb-3">{errorMessage(t, instances.error)}</p>
          <Button variant="secondary" size="sm" onClick={() => void instances.refetch()}>
            {t("versions.retry")}
          </Button>
        </div>
      )}

      {instances.data?.length === 0 && (
        <div className="grid place-items-center rounded-xl border border-dashed py-20 text-center">
          <Boxes size={40} className="mb-4 text-accent" />
          <h2 className="text-lg font-semibold">{t("library.emptyTitle")}</h2>
          <p className="mb-5 mt-1 text-muted">{t("library.emptyBody")}</p>
          <Button onClick={() => setCreating(true)}>
            <Plus size={16} />
            {t("instances.new")}
          </Button>
        </div>
      )}

      {instances.data && instances.data.length > 0 && (
        <>
          <div className="mb-5 flex flex-wrap items-center gap-3">
            <label className="panel flex h-10 min-w-56 flex-1 items-center gap-2 px-3 focus-within:border-accent/60">
              <Search size={16} className="text-muted" />
              <input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder={t("library.search")}
                className="min-w-0 flex-1 bg-transparent outline-none placeholder:text-muted"
              />
            </label>
            <div className="flex gap-1 rounded-lg border bg-surface/60 p-1">
              {SORT_KEYS.map((key) => (
                <button
                  key={key}
                  type="button"
                  onClick={() => setSort(key)}
                  className={cn(
                    "h-8 rounded-md px-3 text-xs font-medium text-muted transition-colors hover:text-foreground",
                    sort === key && "bg-brand text-accent-foreground hover:text-accent-foreground",
                  )}
                >
                  {t(`library.sort.${key}`)}
                </button>
              ))}
            </div>
          </div>
          <div className="mb-5 flex flex-wrap gap-2">
            {[null, ...LOADERS].map((kind) => (
              <button
                key={kind ?? "all"}
                type="button"
                onClick={() => setLoader(kind)}
                style={kind !== null && loader === kind ? loaderChipStyle(kind) : undefined}
                className={cn(
                  "rounded-full border px-3 py-1 text-xs font-medium transition-colors hover:border-accent/50",
                  loader === kind ? "border-accent/60 bg-accent/15 text-foreground" : "text-muted",
                )}
              >
                {kind === null ? t("library.allLoaders") : t(`instances.loader.${kind}`)}
              </button>
            ))}
          </div>
          {visible.length === 0 ? (
            <p className="py-12 text-center text-muted">{t("library.noMatches")}</p>
          ) : (
            <div className="grid grid-cols-[repeat(auto-fill,minmax(18rem,1fr))] gap-4">
              {visible.map((instance, index) => (
                <InstanceCard key={instance.id} instance={instance} index={index} onDelete={setDeleting} />
              ))}
            </div>
          )}
        </>
      )}

      <ImportCodeDialog open={importingCode} onOpenChange={setImportingCode} />
      <NewInstanceDialog open={creating} onOpenChange={setCreating} />
      <DeleteInstanceDialog instance={deleting} onClose={() => setDeleting(null)} />
    </>
  );
}
