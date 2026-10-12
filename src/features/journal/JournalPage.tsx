import { useQuery } from "@tanstack/react-query";
import { Clock, Crown, Flame, Timer } from "lucide-react";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Reveal } from "@/components/Reveal";
import { PageHeader } from "@/components/shell/PageHeader";
import { InstanceIcon } from "@/features/instances/InstanceIcon";
import { useInstances } from "@/features/instances/useInstances";
import { formatPlayTime } from "@/lib/format";
import { listSessions } from "@/lib/tauri/journal";
import { cn } from "@/lib/utils";
import { buildCalendar, recordsOf, secondsPerDay, secondsPerInstance, WEEKS } from "./stats";

const SESSIONS_KEY = ["sessions"];
const CELL_BACKGROUND = ["var(--surface-raised)", ...[22, 42, 68, 100].map((share) => `color-mix(in oklch, var(--accent) ${share}%, var(--surface-raised))`)];

function Tile({ icon: Icon, label, value, detail, index }: { icon: typeof Clock; label: string; value: ReactNode; detail?: ReactNode; index: number }) {
  return (
    <Reveal index={index} className="panel flex items-center gap-4 p-4">
      <div className="grid size-11 shrink-0 place-items-center rounded-xl bg-accent/15 text-accent">
        <Icon size={20} />
      </div>
      <div className="min-w-0">
        <p className="text-xs text-muted">{label}</p>
        <p className="truncate text-lg font-semibold">{value}</p>
        {detail && <p className="truncate text-xs text-muted">{detail}</p>}
      </div>
    </Reveal>
  );
}

export function JournalPage() {
  const { t, i18n } = useTranslation();
  const sessions = useQuery({ queryKey: SESSIONS_KEY, queryFn: listSessions });
  const instances = useInstances();
  const time = (seconds: number) => formatPlayTime(seconds, i18n.language);
  const nameOf = (id: string) => instances.data?.find((instance) => instance.id === id)?.name ?? id;

  const all = sessions.data ?? [];
  const today = new Date();
  const perDay = secondsPerDay(all);
  const calendar = buildCalendar(perDay, today);
  const records = recordsOf(all, perDay, today);
  const perInstance = secondsPerInstance(all);
  const favorite = perInstance[0];
  const longestDate = records.longest ? new Date(records.longest.startedAt * 1000).toLocaleDateString(i18n.language, { day: "numeric", month: "short" }) : null;
  const weekdayNames = calendar[0].map((day) => day.date.toLocaleDateString(i18n.language, { weekday: "short" }));
  const monthLabel = (week: number) => {
    const first = calendar[week][0].date;
    return week === 0 || first.getDate() <= 7 ? first.toLocaleDateString(i18n.language, { month: "short" }) : "";
  };

  return (
    <div className="space-y-6 pb-6">
      <PageHeader title={t("nav.journal")} subtitle={t("journal.subtitle")} />

      <section className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        <Tile index={0} icon={Clock} label={t("journal.thisWeek")} value={time(records.weekSeconds)} />
        <Tile index={1} icon={Flame} label={t("journal.streak")} value={records.streak} />
        <Tile index={2} icon={Timer} label={t("journal.longest")} value={records.longest ? time(records.longest.durationSecs) : "–"} detail={longestDate} />
        <Tile index={3} icon={Crown} label={t("journal.favorite")} value={favorite ? nameOf(favorite[0]) : "–"} detail={favorite && time(favorite[1])} />
      </section>

      <section className="panel p-5">
        <div className="overflow-x-auto pb-1">
          <div className="inline-flex gap-2">
            <div className="flex flex-col gap-1 pt-5 text-[11px] leading-none text-muted">
              {weekdayNames.map((name, index) => (
                <span key={name} className={cn("flex h-4 items-center", index % 2 === 1 && "invisible")}>
                  {name}
                </span>
              ))}
            </div>
            <div className="flex gap-1">
              {calendar.map((week, weekIndex) => (
                <div key={week[0].key} className="flex flex-col gap-1">
                  <span className="h-4 whitespace-nowrap text-[11px] leading-4 text-muted">{monthLabel(weekIndex)}</span>
                  {week.map((day) => (
                    <div
                      key={day.key}
                      title={day.future ? undefined : `${day.date.toLocaleDateString(i18n.language, { day: "numeric", month: "short", year: "numeric" })} · ${time(day.seconds)}`}
                      className={cn("size-4 rounded-[4px] transition-transform", day.future ? "opacity-0" : "hover:scale-125")}
                      style={{ background: CELL_BACKGROUND[day.level] }}
                    />
                  ))}
                </div>
              ))}
            </div>
          </div>
        </div>
        <div className="mt-3 flex items-center justify-between gap-3 text-xs text-muted">
          <span>{all.length === 0 ? t("journal.empty") : t("journal.note", { weeks: WEEKS })}</span>
          <span className="flex shrink-0 items-center gap-1.5">
            {t("journal.less")}
            {CELL_BACKGROUND.map((background) => (
              <span key={background} className="size-3.5 rounded-[3px]" style={{ background }} />
            ))}
            {t("journal.more")}
          </span>
        </div>
      </section>

      {perInstance.length > 0 && (
        <section className="panel space-y-3 p-5">
          <h2 className="font-semibold">{t("journal.perInstance")}</h2>
          {perInstance.map(([id, seconds]) => {
            const instance = instances.data?.find((candidate) => candidate.id === id);
            return (
              <div key={id} className="flex items-center gap-3">
                {instance ? <InstanceIcon icon={instance.icon} instanceId={id} className="size-7 rounded-md" iconSize={14} /> : <div className="size-7" />}
                <span className="w-40 shrink-0 truncate text-sm">{nameOf(id)}</span>
                <div className="h-2 flex-1 overflow-hidden rounded-full bg-surface-raised">
                  <div className="h-full bg-brand" style={{ width: `${(seconds / perInstance[0][1]) * 100}%` }} />
                </div>
                <span className="w-20 shrink-0 text-right text-sm text-muted">{time(seconds)}</span>
              </div>
            );
          })}
        </section>
      )}
    </div>
  );
}
