import { useInfiniteQuery, useQuery } from "@tanstack/react-query";
import { Search } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSearchParams } from "react-router-dom";
import { PageHeader } from "@/components/shell/PageHeader";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { LOADER_KINDS } from "@/features/instances/LoaderPicker";
import { useInstances } from "@/features/instances/useInstances";
import { useVersionManifest } from "@/features/instances/VersionPicker";
import { errorMessage } from "@/lib/tauri/errors";
import {
  modrinthCategories,
  modrinthSearch,
  type ProjectType,
  type SortIndex,
} from "@/lib/tauri/modrinth";
import { cn } from "@/lib/utils";
import { ProjectCard } from "./ProjectCard";

const TABS: ProjectType[] = ["mod", "modpack", "resourcepack", "shader"];
const SORTS: SortIndex[] = ["relevance", "downloads", "follows", "newest", "updated"];
const VISIBLE_CATEGORIES = 14;
const SEARCH_DEBOUNCE_MS = 300;

function useDebounced<T>(value: T, delay: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delay);
    return () => clearTimeout(timer);
  }, [value, delay]);
  return debounced;
}

export function DiscoverPage() {
  const { t } = useTranslation();
  const [params, setParams] = useSearchParams();
  const instances = useInstances();
  const manifest = useVersionManifest();

  const instanceId = params.get("instance");
  const instance = instances.data?.find((candidate) => candidate.id === instanceId) ?? null;

  const [type, setType] = useState<ProjectType>("mod");
  const [text, setText] = useState("");
  const [sort, setSort] = useState<SortIndex>("relevance");
  const [categories, setCategories] = useState<string[]>([]);
  const [gameVersion, setGameVersion] = useState<string | null>(null);
  const [loader, setLoader] = useState<string | null>(null);
  const query = useDebounced(text, SEARCH_DEBOUNCE_MS);

  useEffect(() => {
    setGameVersion(instance?.gameVersion ?? null);
    setLoader(instance && instance.loader !== "vanilla" ? instance.loader : null);
  }, [instance?.id]); // eslint-disable-line react-hooks/exhaustive-deps

  const releases = useMemo(
    () => (manifest.data?.versions ?? []).filter((entry) => entry.type === "release").map((entry) => entry.id),
    [manifest.data],
  );
  const allCategories = useQuery({ queryKey: ["modrinth-categories"], queryFn: modrinthCategories, staleTime: 60 * 60_000 });
  const visibleCategories = (allCategories.data ?? [])
    .filter((category) => category.projectType === type && category.header === "categories")
    .slice(0, VISIBLE_CATEGORIES);

  const effectiveLoader = type === "mod" ? loader : null;
  const results = useInfiniteQuery({
    queryKey: ["modrinth-search", type, query, sort, categories, gameVersion, effectiveLoader],
    queryFn: ({ pageParam }) =>
      modrinthSearch({ query, projectType: type, gameVersion, loader: effectiveLoader, categories, sort, offset: pageParam }),
    initialPageParam: 0,
    getNextPageParam: (last) => (last.offset + last.limit < last.totalHits ? last.offset + last.limit : undefined),
    staleTime: 60_000,
  });
  const hits = results.data?.pages.flatMap((page) => page.hits) ?? [];

  const switchType = (next: ProjectType) => {
    setType(next);
    setCategories([]);
  };
  const toggleCategory = (name: string) =>
    setCategories((current) => (current.includes(name) ? current.filter((entry) => entry !== name) : [...current, name]));

  return (
    <>
      <PageHeader title={t("discover.title")} subtitle={t("discover.subtitle")} />

      <div className="mb-4 flex flex-wrap gap-1 rounded-md bg-surface p-1 sm:w-fit">
        {TABS.map((tab) => (
          <button
            key={tab}
            type="button"
            onClick={() => switchType(tab)}
            className={cn(
              "h-8 rounded-sm px-4 text-sm font-medium transition-colors",
              type === tab ? "bg-accent text-accent-foreground" : "text-muted hover:text-foreground",
            )}
          >
            {t(`discover.type.${tab}`)}
          </button>
        ))}
      </div>

      <div className="mb-3 flex flex-wrap items-center gap-3">
        <label className="relative min-w-56 flex-1">
          <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-muted" />
          <input
            value={text}
            onChange={(event) => setText(event.target.value)}
            placeholder={t("discover.search")}
            className="h-10 w-full select-text rounded-md border bg-surface pl-9 pr-3 outline-none transition-colors placeholder:text-muted focus:border-accent"
          />
        </label>
        <Select value={sort} onChange={(event) => setSort(event.target.value as SortIndex)} aria-label={t("discover.sort.label")}>
          {SORTS.map((option) => (
            <option key={option} value={option}>
              {t(`discover.sort.${option}`)}
            </option>
          ))}
        </Select>
      </div>

      <div className="mb-3 flex flex-wrap items-center gap-3">
        {type !== "modpack" && (
          <Select
            value={instanceId ?? ""}
            onChange={(event) => setParams(event.target.value ? { instance: event.target.value } : {})}
            aria-label={t("discover.installTo")}
          >
            <option value="">{t("discover.chooseInstance")}</option>
            {instances.data?.map((candidate) => (
              <option key={candidate.id} value={candidate.id}>
                {candidate.name}
              </option>
            ))}
          </Select>
        )}
        <Select
          value={gameVersion ?? ""}
          onChange={(event) => setGameVersion(event.target.value || null)}
          aria-label={t("instances.gameVersion")}
        >
          <option value="">{t("discover.anyVersion")}</option>
          {gameVersion && !releases.includes(gameVersion) && <option value={gameVersion}>{gameVersion}</option>}
          {releases.slice(0, 80).map((version) => (
            <option key={version} value={version}>
              {version}
            </option>
          ))}
        </Select>
        {type === "mod" && (
          <Select value={loader ?? ""} onChange={(event) => setLoader(event.target.value || null)} aria-label={t("instances.loaderLabel")}>
            <option value="">{t("discover.anyLoader")}</option>
            {LOADER_KINDS.filter((kind) => kind !== "vanilla").map((kind) => (
              <option key={kind} value={kind}>
                {t(`instances.loader.${kind}`)}
              </option>
            ))}
          </Select>
        )}
      </div>

      {visibleCategories.length > 0 && (
        <div className="mb-5 flex flex-wrap gap-2">
          {visibleCategories.map((category) => (
            <button
              key={category.name}
              type="button"
              onClick={() => toggleCategory(category.name)}
              className={cn(
                "rounded-full border px-3 py-1 text-xs font-medium capitalize transition-colors",
                categories.includes(category.name)
                  ? "border-accent bg-accent/15 text-foreground"
                  : "text-muted hover:text-foreground",
              )}
            >
              {t(`discover.categories.${category.name}`, { defaultValue: category.name.replace(/-/g, " ") })}
            </button>
          ))}
        </div>
      )}

      {results.isPending && (
        <div className="space-y-3">
          {Array.from({ length: 6 }, (_, index) => (
            <div key={index} className="skeleton h-[108px] rounded-xl" />
          ))}
        </div>
      )}

      {results.isError && (
        <div className="rounded-lg border border-danger/40 bg-danger/10 p-5">
          <p className="mb-3">{errorMessage(t, results.error)}</p>
          <Button variant="secondary" size="sm" onClick={() => void results.refetch()}>
            {t("versions.retry")}
          </Button>
        </div>
      )}

      {results.isSuccess && hits.length === 0 && <p className="py-10 text-center text-muted">{t("discover.empty")}</p>}

      <div className="space-y-3">
        {hits.map((hit) => (
          <ProjectCard key={hit.projectId} hit={hit} instance={instance} />
        ))}
      </div>

      {results.hasNextPage && (
        <div className="mt-5 flex justify-center">
          <Button variant="secondary" disabled={results.isFetchingNextPage} onClick={() => void results.fetchNextPage()}>
            {t("discover.loadMore")}
          </Button>
        </div>
      )}
    </>
  );
}
