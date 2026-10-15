import { Boxes, FileUp, KeyRound, Plus, Search } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { PageHeader } from "@/components/shell/PageHeader";
import { Button } from "@/components/ui/button";
import { BedrockCard } from "@/features/bedrock/BedrockCard";
import { BedrockPacksDialog } from "@/features/bedrock/BedrockPacksDialog";
import { DeleteBedrockDialog } from "@/features/bedrock/DeleteBedrockDialog";
import { useBedrockInstances } from "@/features/bedrock/useBedrockInstances";
import { ImportCodeDialog } from "@/features/share/ImportCodeDialog";
import { useJobsStore } from "@/features/discover/jobsStore";
import { pickMrpackFile } from "@/lib/tauri/modrinth";
import { DeleteInstanceDialog } from "@/features/instances/DeleteInstanceDialog";
import { InstanceCard } from "@/features/instances/InstanceCard";
import { NewInstanceDialog } from "@/features/instances/NewInstanceDialog";
import { useInstances } from "@/features/instances/useInstances";
import { errorMessage } from "@/lib/tauri/errors";
import { LOADER_HUE, loaderChipStyle } from "@/lib/loaderColors";
import type { BedrockInstance } from "@/lib/tauri/bedrock";
import type { Instance, LoaderKind } from "@/lib/tauri/instances";
import { cn } from "@/lib/utils";

type SortKey = "recent" | "name" | "version";

/** A Java instance or a Bedrock one: the library shows them side by side. */
type Entry = { kind: "java"; instance: Instance } | { kind: "bedrock"; instance: BedrockInstance };
type Filter = LoaderKind | "bedrock" | null;

const SORT_KEYS: SortKey[] = ["recent", "name", "version"];
const LOADERS = Object.keys(LOADER_HUE) as LoaderKind[];

const versionOf = (entry: Entry) => (entry.kind === "java" ? entry.instance.gameVersion : "");
const lastUsedOf = (entry: Entry) => (entry.kind === "java" ? (entry.instance.lastPlayedAt ?? "") : entry.instance.createdAt);

function sortEntries(list: Entry[], key: SortKey): Entry[] {
  const sorted = [...list];
  if (key === "name") return sorted.sort((a, b) => a.instance.name.localeCompare(b.instance.name));
  if (key === "version") return sorted.sort((a, b) => versionOf(b).localeCompare(versionOf(a), undefined, { numeric: true }));
  return sorted.sort((a, b) => lastUsedOf(b).localeCompare(lastUsedOf(a)));
}

export function LibraryPage() {
  const { t } = useTranslation();
  const instances = useInstances();
  const bedrock = useBedrockInstances();
  const [managing, setManaging] = useState<BedrockInstance | null>(null);
  const [deletingBedrock, setDeletingBedrock] = useState<BedrockInstance | null>(null);
  const [creating, setCreating] = useState(false);
  const [importingCode, setImportingCode] = useState(false);
  const [deleting, setDeleting] = useState<Instance | null>(null);
  const { jobs, importPack: runImport } = useJobsStore();
  const importing = jobs["mrpack-import"] !== undefined;
  const [query, setQuery] = useState("");
  const [sort, setSort] = useState<SortKey>("recent");
  const [loader, setLoader] = useState<Filter>(null);

  const total = (instances.data?.length ?? 0) + (bedrock.data?.length ?? 0);
  const visible = useMemo(() => {
    const needle = query.trim().toLowerCase();
    const entries: Entry[] = [
      ...(instances.data ?? []).map((instance): Entry => ({ kind: "java", instance })),
      ...(bedrock.data ?? []).map((instance): Entry => ({ kind: "bedrock", instance })),
    ];
    const matching = entries.filter(
      (entry) =>
        (loader === null || (loader === "bedrock" ? entry.kind === "bedrock" : entry.kind === "java" && entry.instance.loader === loader)) &&
        (needle === "" || `${entry.instance.name} ${versionOf(entry)}`.toLowerCase().includes(needle)),
    );
    return sortEntries(matching, sort);
  }, [instances.data, bedrock.data, query, sort, loader]);

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

      {instances.data && bedrock.data && total === 0 && (
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

      {total > 0 && (
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
            {([null, ...LOADERS, "bedrock"] as Filter[]).map((kind) => (
              <button
                key={kind ?? "all"}
                type="button"
                onClick={() => setLoader(kind)}
                style={kind !== null && kind !== "bedrock" && loader === kind ? loaderChipStyle(kind) : undefined}
                className={cn(
                  "rounded-full border px-3 py-1 text-xs font-medium transition-colors hover:border-accent/50",
                  loader === kind ? "border-accent/60 bg-accent/15 text-foreground" : "text-muted",
                )}
              >
                {kind === null ? t("library.allLoaders") : kind === "bedrock" ? t("servers.bedrock") : t(`instances.loader.${kind}`)}
              </button>
            ))}
          </div>
          {visible.length === 0 ? (
            <p className="py-12 text-center text-muted">{t("library.noMatches")}</p>
          ) : (
            <div className="grid grid-cols-[repeat(auto-fill,minmax(18rem,1fr))] gap-4">
              {visible.map((entry, index) =>
                entry.kind === "java" ? (
                  <InstanceCard key={entry.instance.id} instance={entry.instance} index={index} onDelete={setDeleting} />
                ) : (
                  <BedrockCard key={entry.instance.id} instance={entry.instance} index={index} onManage={setManaging} onDelete={setDeletingBedrock} />
                ),
              )}
            </div>
          )}
        </>
      )}

      <ImportCodeDialog open={importingCode} onOpenChange={setImportingCode} />
      <NewInstanceDialog open={creating} onOpenChange={setCreating} />
      <DeleteInstanceDialog instance={deleting} onClose={() => setDeleting(null)} />
      <BedrockPacksDialog instance={bedrock.data?.find((item) => item.id === managing?.id) ?? null} onClose={() => setManaging(null)} />
      <DeleteBedrockDialog instance={deletingBedrock} onClose={() => setDeletingBedrock(null)} />
    </>
  );
}
