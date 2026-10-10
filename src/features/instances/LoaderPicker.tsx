import { useQuery } from "@tanstack/react-query";
import { Check } from "lucide-react";
import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { listLoaderVersions, type LoaderKind } from "@/lib/tauri/instances";
import { cn } from "@/lib/utils";

export const LOADER_KINDS: LoaderKind[] = ["vanilla", "fabric", "quilt", "forge", "neoforge"];

interface LoaderPickerProps {
  gameVersion: string;
  kind: LoaderKind;
  version: string | null;
  onChange: (kind: LoaderKind, version: string | null) => void;
}

export function LoaderPicker({ gameVersion, kind, version, onChange }: LoaderPickerProps) {
  const { t } = useTranslation();
  const versions = useQuery({
    queryKey: ["loader-versions", kind, gameVersion],
    queryFn: () => listLoaderVersions(kind, gameVersion),
    enabled: kind !== "vanilla" && gameVersion !== "",
    staleTime: 5 * 60_000,
  });

  const previousGameVersion = useRef(gameVersion);
  useEffect(() => {
    if (previousGameVersion.current !== gameVersion) {
      previousGameVersion.current = gameVersion;
      if (version !== null) onChange(kind, null);
    }
  }, [gameVersion, kind, version, onChange]);

  useEffect(() => {
    const available = versions.data;
    if (kind === "vanilla" || !available || available.length === 0) return;
    if (!available.some((entry) => entry.version === version)) {
      const preferred = available.find((entry) => entry.recommended) ?? available[0];
      onChange(kind, preferred.version);
    }
  }, [kind, versions.data, version, onChange]);

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap gap-1 rounded-md bg-surface p-1">
        {LOADER_KINDS.map((item) => (
          <button
            key={item}
            type="button"
            onClick={() => item !== kind && onChange(item, null)}
            className={cn(
              "h-8 rounded-sm px-3 text-xs font-medium transition-colors",
              kind === item ? "bg-accent text-accent-foreground" : "text-muted hover:text-foreground",
            )}
          >
            {t(`instances.loader.${item}`)}
          </button>
        ))}
      </div>

      {kind !== "vanilla" && (
        <div className="h-36 overflow-y-auto rounded-md border bg-background/40 p-1">
          {versions.isPending &&
            Array.from({ length: 4 }, (_, index) => <div key={index} className="skeleton mb-1 h-9 rounded-md" />)}
          {versions.isError && <p className="p-3 text-sm text-danger">{t("instances.loaderLoadFailed")}</p>}
          {versions.data?.length === 0 && (
            <p className="p-3 text-sm text-muted">
              {t("instances.noLoaderVersions", { loader: t(`instances.loader.${kind}`), version: gameVersion })}
            </p>
          )}
          {versions.data?.map((entry) => (
            <button
              key={entry.version}
              type="button"
              onClick={() => onChange(kind, entry.version)}
              className={cn(
                "flex h-9 w-full items-center justify-between rounded-md px-3 text-sm transition-colors hover:bg-surface-raised",
                entry.version === version && "bg-accent/15",
              )}
            >
              <span className="flex items-center gap-2">
                {entry.version}
                {entry.recommended && (
                  <span className="rounded-full bg-accent/15 px-2 py-0.5 text-[11px] font-medium text-accent">
                    {t("instances.recommended")}
                  </span>
                )}
              </span>
              {entry.version === version && <Check size={14} className="text-accent" />}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
