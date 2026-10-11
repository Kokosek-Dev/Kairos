import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Bookmark, Globe2, History, Package, RotateCcw, Trash2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Reveal } from "@/components/Reveal";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { isBusy, useGameStore } from "@/features/launch/gameStore";
import { contentKey } from "@/features/discover/jobsStore";
import { INSTANCES_KEY } from "@/features/instances/useInstances";
import { formatBytes, formatRelative } from "@/lib/format";
import { errorMessage } from "@/lib/tauri/errors";
import {
  compareMoment,
  createMoment,
  deleteMoment,
  listMoments,
  momentSizes,
  restoreMoment,
  type Moment,
  type MomentReason,
} from "@/lib/tauri/moments";
import { cn } from "@/lib/utils";

/** Hue shift from the accent for each kind of moment, so the timeline reads at a glance. */
const REASON_SHIFT: Record<MomentReason, number> = { manual: 0, before_update: 70, before_change: 140, before_restore: 210 };

const momentsKey = (instanceId: string) => ["moments", instanceId];

function Difference({ instanceId, moment }: { instanceId: string; moment: Moment }) {
  const { t } = useTranslation();
  const difference = useQuery({ queryKey: ["moment-difference", instanceId, moment.id], queryFn: () => compareMoment(instanceId, moment.id) });
  if (!difference.data) return <div className="skeleton mt-3 h-10 rounded-lg" />;
  const { added, removed, versionChanged } = difference.data;
  if (added.length === 0 && removed.length === 0 && !versionChanged) {
    return <p className="mt-3 text-sm text-muted">{t("moments.nothingChanged")}</p>;
  }

  return (
    <div className="mt-3 space-y-2 text-sm">
      {versionChanged && <p className="text-warning">{t("moments.versionChanged")}</p>}
      {added.length > 0 && (
        <p>
          <span className="font-medium text-success">{t("moments.added")}: </span>
          <span className="text-muted">{added.join(", ")}</span>
        </p>
      )}
      {removed.length > 0 && (
        <p>
          <span className="font-medium text-danger">{t("moments.removed")}: </span>
          <span className="text-muted">{removed.join(", ")}</span>
        </p>
      )}
    </div>
  );
}

export function MomentsSection({ instanceId }: { instanceId: string }) {
  const { t, i18n } = useTranslation();
  const queryClient = useQueryClient();
  const { instanceId: runningId, phase } = useGameStore();
  const running = runningId === instanceId && isBusy(phase);
  const moments = useQuery({ queryKey: momentsKey(instanceId), queryFn: () => listMoments(instanceId) });
  const sizes = useQuery({ queryKey: ["moment-sizes", instanceId], queryFn: () => momentSizes(instanceId) });
  const [note, setNote] = useState("");
  const [worlds, setWorlds] = useState(false);
  const [comparing, setComparing] = useState<string | null>(null);
  const [restoring, setRestoring] = useState<Moment | null>(null);

  const refresh = () => queryClient.invalidateQueries({ queryKey: momentsKey(instanceId) });
  const onError = (error: unknown) => toast.error(errorMessage(t, error));

  const save = useMutation({
    mutationFn: () => createMoment(instanceId, note, worlds),
    onSuccess: async () => {
      setNote("");
      await refresh();
      toast.success(t("moments.saved"));
    },
    onError,
  });
  const restore = useMutation({
    mutationFn: (moment: Moment) => restoreMoment(instanceId, moment.id),
    onSuccess: async () => {
      setRestoring(null);
      await Promise.all([refresh(), queryClient.invalidateQueries({ queryKey: INSTANCES_KEY }), queryClient.invalidateQueries({ queryKey: contentKey(instanceId) })]);
      toast.success(t("moments.restored"));
    },
    onError,
  });
  const remove = useMutation({ mutationFn: (moment: Moment) => deleteMoment(instanceId, moment.id), onSuccess: refresh, onError });

  return (
    <section className="panel space-y-5 p-5">
      <div>
        <h2 className="flex items-center gap-2 font-semibold">
          <History size={16} className="text-accent" />
          {t("moments.title")}
        </h2>
        <p className="mt-1 text-muted">{t("moments.hint")}</p>
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <Input
          value={note}
          maxLength={120}
          placeholder={t("moments.notePlaceholder")}
          onChange={(event) => setNote(event.target.value)}
          className="min-w-56 flex-1"
        />
        <label className="flex items-center gap-2 text-sm text-muted">
          <input type="checkbox" checked={worlds} onChange={(event) => setWorlds(event.target.checked)} />
          {t("moments.includeWorlds", { size: formatBytes(sizes.data?.worldsBytes ?? 0) })}
        </label>
        <Button disabled={running || save.isPending} title={running ? t("moments.gameRunning") : undefined} onClick={() => save.mutate()}>
          <Bookmark size={16} />
          {t("moments.save")}
        </Button>
      </div>

      {moments.data?.length === 0 && <p className="rounded-xl border border-dashed p-6 text-center text-muted">{t("moments.empty")}</p>}

      <ol className="relative space-y-3 border-l border-border pl-6">
        {moments.data?.map((moment, index) => {
          const shift = REASON_SHIFT[moment.reason];
          return (
            <li key={moment.id} className="relative">
              <span
                className="absolute -left-[1.95rem] top-4 size-3 rounded-full ring-4 ring-background"
                style={{ background: `oklch(0.75 0.16 calc(var(--accent-hue) + ${shift}))` }}
              />
              <Reveal index={index} className="rounded-xl border bg-background/40 p-4">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="min-w-0">
                    <p className="truncate font-medium">{moment.note || t(`moments.reason.${moment.reason}`)}</p>
                    <p className="flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted">
                      <span>{formatRelative(moment.createdAt, i18n.language)}</span>
                      {moment.note && <span>{t(`moments.reason.${moment.reason}`)}</span>}
                      <span className="inline-flex items-center gap-1">
                        <Package size={12} />
                        {t("moments.modsCount", { count: moment.mods.length })}
                      </span>
                      <span>{formatBytes(moment.sizeBytes)}</span>
                      {moment.includesWorlds && (
                        <span className="inline-flex items-center gap-1 text-success">
                          <Globe2 size={12} />
                          {t("moments.worlds")}
                        </span>
                      )}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button variant="ghost" size="sm" onClick={() => setComparing(comparing === moment.id ? null : moment.id)}>
                      {comparing === moment.id ? t("moments.hideChanges") : t("moments.compare")}
                    </Button>
                    <Button
                      variant="secondary"
                      size="sm"
                      disabled={running}
                      title={running ? t("moments.gameRunning") : undefined}
                      onClick={() => setRestoring(moment)}
                    >
                      <RotateCcw size={14} />
                      {t("moments.restore")}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className={cn("size-8 px-0 hover:text-danger")}
                      aria-label={t("moments.delete")}
                      title={t("moments.delete")}
                      onClick={() => remove.mutate(moment)}
                    >
                      <Trash2 size={15} />
                    </Button>
                  </div>
                </div>
                {comparing === moment.id && <Difference instanceId={instanceId} moment={moment} />}
              </Reveal>
            </li>
          );
        })}
      </ol>

      <Dialog open={restoring !== null} onOpenChange={(open) => !open && !restore.isPending && setRestoring(null)}>
        <DialogContent className="w-[min(30rem,calc(100vw-2rem))]">
          <DialogTitle>{t("moments.restoreTitle")}</DialogTitle>
          <DialogDescription>{t("moments.restoreBody")}</DialogDescription>
          {restoring?.includesWorlds && <p className="mt-3 text-sm text-warning">{t("moments.restoreWorlds")}</p>}
          <div className="mt-6 flex justify-end gap-2">
            <Button variant="ghost" disabled={restore.isPending} onClick={() => setRestoring(null)}>
              {t("common.cancel")}
            </Button>
            <Button disabled={restore.isPending} onClick={() => restoring && restore.mutate(restoring)}>
              <RotateCcw size={16} />
              {t("moments.restore")}
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </section>
  );
}
