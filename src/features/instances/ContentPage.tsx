import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, ArrowUpCircle, Compass, Package, RefreshCw, Search, Trash2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { useDetectiveStore } from "@/features/detective/detectiveStore";
import { contentKey, useJobsStore } from "@/features/discover/jobsStore";
import { createSafetyMoment } from "@/lib/tauri/moments";
import { formatBytes } from "@/lib/format";
import { errorMessage } from "@/lib/tauri/errors";
import {
  checkContentUpdates,
  listContent,
  removeContent,
  setContentEnabled,
  type ContentItem,
  type ContentKind,
  type UpdateInfo,
} from "@/lib/tauri/modrinth";
import { cn } from "@/lib/utils";
import { useInstance } from "./useInstances";

const KINDS: ContentKind[] = ["mod", "resource_pack", "shader"];

function Toggle({ on, onChange, label }: { on: boolean; onChange: (on: boolean) => void; label: string }) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={on}
      aria-label={label}
      onClick={() => onChange(!on)}
      className={cn("relative h-5 w-9 shrink-0 rounded-full transition-colors", on ? "bg-accent" : "bg-surface-raised")}
    >
      <span className={cn("absolute top-0.5 size-4 rounded-full bg-white transition-all", on ? "left-[18px]" : "left-0.5")} />
    </button>
  );
}

function Row({ item, instanceId, update }: { item: ContentItem; instanceId: string; update?: UpdateInfo }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const refresh = () => queryClient.invalidateQueries({ queryKey: contentKey(instanceId) });
  const toggle = useMutation({
    mutationFn: (enabled: boolean) => setContentEnabled(instanceId, item.kind, item.fileName, enabled),
    onSuccess: refresh,
  });
  const remove = useMutation({
    mutationFn: () => removeContent(instanceId, item.kind, item.fileName),
    onSuccess: refresh,
  });
  const entry = item.entry;

  return (
    <li className={cn("flex items-center gap-3 rounded-lg border bg-surface px-3 py-2.5", !item.enabled && "opacity-60")}>
      {entry?.iconUrl ? (
        <img src={entry.iconUrl} alt="" className="size-10 shrink-0 rounded-md bg-surface-raised object-cover" draggable={false} />
      ) : (
        <div className="grid size-10 shrink-0 place-items-center rounded-md bg-surface-raised text-muted">
          <Package size={18} />
        </div>
      )}
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="truncate font-medium">{entry?.title ?? item.fileName}</span>
          {entry?.dependency && (
            <span className="rounded-full bg-surface-raised px-2 py-0.5 text-[11px] text-muted">{t("content.dependency")}</span>
          )}
          {update && (
            <span className="rounded-full bg-accent/15 px-2 py-0.5 text-[11px] font-medium text-accent">
              {t("content.updateTo", { version: update.newVersion })}
            </span>
          )}
        </div>
        <p className="truncate text-xs text-muted">
          {entry?.versionNumber ? `${entry.versionNumber} · ` : ""}
          {item.fileName} · {formatBytes(item.size)}
        </p>
      </div>
      <Toggle on={item.enabled} onChange={(on) => toggle.mutate(on)} label={t("content.enabled")} />
      <Button
        variant="ghost"
        size="sm"
        className="size-8 px-0 hover:text-danger"
        aria-label={t("content.remove")}
        title={t("content.remove")}
        onClick={() => remove.mutate()}
      >
        <Trash2 size={15} />
      </Button>
    </li>
  );
}

export function ContentPage() {
  const { id = "" } = useParams();
  const { t } = useTranslation();
  const navigate = useNavigate();
  const instance = useInstance(id);
  const openDetective = useDetectiveStore((state) => state.open);
  const items = useQuery({ queryKey: contentKey(id), queryFn: () => listContent(id) });
  const [updates, setUpdates] = useState<UpdateInfo[] | null>(null);
  const { jobs, update } = useJobsStore();
  const updateJob = jobs[`update:${id}`];

  const check = useMutation({ mutationFn: () => checkContentUpdates(id), onSuccess: setUpdates });
  const updateAll = async () => {
    if (!updates) return;
    await createSafetyMoment(id, "before_update").catch(() => undefined);
    await update(t, id, updates.map((entry) => ({ projectId: entry.projectId, versionId: entry.newVersionId })));
    setUpdates(null);
  };

  const modsBlocked = instance.data?.loader === "vanilla";

  return (
    <div className="max-w-3xl space-y-5 pb-10">
      <Link to={`/instances/${id}`} className="inline-flex items-center gap-2 text-muted transition-colors hover:text-foreground">
        <ArrowLeft size={16} />
        {t("content.back")}
      </Link>

      <div className="flex flex-wrap items-center justify-between gap-3">
        <h1 className="text-2xl font-bold tracking-tight">{t("content.title", { name: instance.data?.name ?? "" })}</h1>
        <div className="flex gap-2">
          <Button
            variant="secondary"
            disabled={modsBlocked || !instance.data}
            onClick={() => instance.data && void openDetective(id, instance.data.name)}
          >
            <Search size={15} />
            {t("detective.button")}
          </Button>
          <Button variant="secondary" disabled={check.isPending || modsBlocked} onClick={() => check.mutate()}>
            <RefreshCw size={15} className={cn(check.isPending && "animate-spin")} />
            {t("content.checkUpdates")}
          </Button>
          <Button onClick={() => navigate(`/discover?instance=${id}`)}>
            <Compass size={15} />
            {t("content.browse")}
          </Button>
        </div>
      </div>

      {modsBlocked && <p className="rounded-lg border bg-surface p-3 text-sm text-muted">{t("content.needsLoader")}</p>}
      {check.isError && <p className="text-sm text-danger">{errorMessage(t, check.error)}</p>}

      {updates !== null && (
        <div className="rounded-xl border border-accent/40 bg-accent/10 p-4">
          {updates.length === 0 ? (
            <p>{t("content.upToDate")}</p>
          ) : (
            <div className="flex flex-wrap items-center justify-between gap-3">
              <p className="font-medium">{t("content.updatesAvailable", { count: updates.length })}</p>
              <Button size="sm" disabled={updateJob !== undefined} onClick={() => void updateAll()}>
                <ArrowUpCircle size={14} />
                {t("content.updateAll")}
              </Button>
            </div>
          )}
        </div>
      )}
      {items.isPending && Array.from({ length: 4 }, (_, index) => <div key={index} className="skeleton h-[60px] rounded-lg" />)}
      {items.isError && <p className="text-danger">{errorMessage(t, items.error)}</p>}
      {items.data?.length === 0 && (
        <div className="grid place-items-center rounded-xl border border-dashed py-14 text-center">
          <Package size={36} className="mb-3 text-accent" />
          <p className="font-medium">{t("content.empty")}</p>
          <p className="mt-1 text-muted">{t("content.emptyHint")}</p>
        </div>
      )}

      {KINDS.map((kind) => {
        const group = items.data?.filter((item) => item.kind === kind) ?? [];
        if (group.length === 0) return null;
        return (
          <section key={kind}>
            <h2 className="mb-2 text-sm font-semibold uppercase tracking-wide text-muted">
              {t(`content.kind.${kind}`)} ({group.length})
            </h2>
            <ul className="space-y-2">
              {group.map((item) => (
                <Row
                  key={`${item.kind}:${item.fileName}`}
                  item={item}
                  instanceId={id}
                  update={updates?.find((candidate) => candidate.kind === item.kind && candidate.fileName === item.fileName)}
                />
              ))}
            </ul>
          </section>
        );
      })}
    </div>
  );
}
