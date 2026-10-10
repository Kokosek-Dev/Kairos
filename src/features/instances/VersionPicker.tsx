import { useQuery } from "@tanstack/react-query";
import { Check } from "lucide-react";
import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import { getVersionManifest, type VersionKind } from "@/lib/tauri/versions";
import { cn } from "@/lib/utils";

type Filter = "release" | "snapshot" | "old";

const FILTERS: Filter[] = ["release", "snapshot", "old"];

function matches(kind: VersionKind, filter: Filter): boolean {
  return filter === "old" ? kind === "old_beta" || kind === "old_alpha" : kind === filter;
}

export function useVersionManifest() {
  return useQuery({ queryKey: ["version-manifest"], queryFn: getVersionManifest, staleTime: 5 * 60_000 });
}

interface VersionPickerProps {
  value: string;
  onChange: (versionId: string) => void;
}

export function VersionPicker({ value, onChange }: VersionPickerProps) {
  const { t } = useTranslation();
  const manifest = useVersionManifest();
  const [filter, setFilter] = useState<Filter>("release");
  const [search, setSearch] = useState("");

  const visible = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return (manifest.data?.versions ?? []).filter(
      (entry) => matches(entry.type, filter) && entry.id.toLowerCase().includes(needle),
    );
  }, [manifest.data, filter, search]);

  return (
    <div className="space-y-2">
      <div className="flex gap-2">
        <div className="flex gap-1 rounded-md bg-surface p-1">
          {FILTERS.map((item) => (
            <button
              key={item}
              type="button"
              onClick={() => setFilter(item)}
              className={cn(
                "h-8 rounded-sm px-3 text-xs font-medium transition-colors",
                filter === item ? "bg-accent text-accent-foreground" : "text-muted hover:text-foreground",
              )}
            >
              {t(`versions.filter.${item}`)}
            </button>
          ))}
        </div>
        <Input
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder={t("versions.search")}
          className="h-10 flex-1"
        />
      </div>

      <div className="h-48 overflow-y-auto rounded-md border bg-background/40 p-1">
        {manifest.isPending &&
          Array.from({ length: 5 }, (_, index) => <div key={index} className="skeleton mb-1 h-9 rounded-md" />)}
        {manifest.isError && <p className="p-3 text-sm text-danger">{t("versions.loadFailed")}</p>}
        {manifest.data && visible.length === 0 && (
          <p className="p-3 text-center text-sm text-muted">{t("versions.empty")}</p>
        )}
        {visible.map((entry) => (
          <button
            key={entry.id}
            type="button"
            onClick={() => onChange(entry.id)}
            className={cn(
              "flex h-9 w-full items-center justify-between rounded-md px-3 text-sm transition-colors hover:bg-surface-raised",
              entry.id === value && "bg-accent/15 text-foreground",
            )}
          >
            {entry.id}
            {entry.id === value && <Check size={14} className="text-accent" />}
          </button>
        ))}
      </div>
    </div>
  );
}
