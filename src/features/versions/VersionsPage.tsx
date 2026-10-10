import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Search } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { PageHeader } from "@/components/shell/PageHeader";
import {
  getVersionManifest,
  listInstalledVersions,
  type VersionKind,
} from "@/lib/tauri/versions";
import { cn } from "@/lib/utils";
import { useInstallStore } from "./installStore";
import { VersionRow } from "./VersionRow";

type Filter = "release" | "snapshot" | "old";

const FILTERS: Filter[] = ["release", "snapshot", "old"];
const OLD_KINDS: VersionKind[] = ["old_beta", "old_alpha"];

function matchesFilter(kind: VersionKind, filter: Filter): boolean {
  return filter === "old" ? OLD_KINDS.includes(kind) : kind === filter;
}

const INSTALLED_KEY = ["installed-versions"];

export function VersionsPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [filter, setFilter] = useState<Filter>("release");
  const [search, setSearch] = useState("");
  const { progress, errors, start, cancel } = useInstallStore();

  const manifest = useQuery({ queryKey: ["version-manifest"], queryFn: getVersionManifest, staleTime: 5 * 60_000 });
  const installed = useQuery({ queryKey: INSTALLED_KEY, queryFn: listInstalledVersions });
  const installedSet = useMemo(() => new Set(installed.data), [installed.data]);

  const visible = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return (manifest.data?.versions ?? []).filter(
      (entry) => matchesFilter(entry.type, filter) && entry.id.toLowerCase().includes(needle),
    );
  }, [manifest.data, filter, search]);

  const refreshInstalled = () => void queryClient.invalidateQueries({ queryKey: INSTALLED_KEY });

  return (
    <>
      <PageHeader title={t("versions.title")} subtitle={t("versions.subtitle")} />

      <div className="mb-4 flex flex-wrap items-center gap-3">
        <div className="flex gap-1 rounded-md bg-surface p-1">
          {FILTERS.map((item) => (
            <button
              key={item}
              type="button"
              onClick={() => setFilter(item)}
              className={cn(
                "h-8 rounded-sm px-4 text-sm font-medium transition-colors",
                filter === item ? "bg-accent text-accent-foreground" : "text-muted hover:text-foreground",
              )}
            >
              {t(`versions.filter.${item}`)}
            </button>
          ))}
        </div>
        <label className="relative min-w-48 flex-1">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-muted" />
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder={t("versions.search")}
            className="h-10 w-full select-text rounded-md border bg-surface pl-9 pr-3 outline-none transition-colors placeholder:text-muted focus:border-accent"
          />
        </label>
      </div>

      {manifest.isPending && (
        <div className="space-y-2">
          {Array.from({ length: 8 }, (_, index) => (
            <div key={index} className="skeleton h-[60px] rounded-lg" />
          ))}
        </div>
      )}

      {manifest.isError && (
        <div className="rounded-lg border border-danger/40 bg-danger/10 p-5">
          <p className="font-medium">{t("versions.loadFailed")}</p>
          <p className="mb-3 mt-1 text-muted">{t("versions.loadFailedHint")}</p>
          <Button variant="secondary" size="sm" onClick={() => void manifest.refetch()}>
            {t("versions.retry")}
          </Button>
        </div>
      )}

      {manifest.data && (
        <div className="space-y-2">
          {visible.length === 0 && <p className="py-10 text-center text-muted">{t("versions.empty")}</p>}
          {visible.map((entry) => (
            <VersionRow
              key={entry.id}
              entry={entry}
              installed={installedSet.has(entry.id)}
              progress={progress[entry.id]}
              error={errors[entry.id]}
              onInstall={() => void start(entry.id, refreshInstalled)}
              onCancel={() => void cancel(entry.id)}
            />
          ))}
        </div>
      )}
    </>
  );
}
