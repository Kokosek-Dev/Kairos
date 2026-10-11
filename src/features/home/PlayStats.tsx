import { Clock, Crown, Layers } from "lucide-react";
import { useTranslation } from "react-i18next";
import { formatPlayTime } from "@/lib/format";
import type { Instance } from "@/lib/tauri/instances";

interface StatProps {
  icon: typeof Clock;
  label: string;
  value: string;
  detail?: string;
  shift: number;
}

function Stat({ icon: Icon, label, value, detail, shift }: StatProps) {
  return (
    <div className="panel flex items-center gap-4 p-4">
      <div
        className="grid size-11 shrink-0 place-items-center rounded-xl"
        style={{
          color: `oklch(0.85 0.14 calc(var(--accent-hue) + ${shift}))`,
          background: `oklch(0.5 0.12 calc(var(--accent-hue) + ${shift}) / 0.22)`,
        }}
      >
        <Icon size={20} />
      </div>
      <div className="min-w-0">
        <p className="text-xs text-muted">{label}</p>
        <p className="truncate text-lg font-semibold">{value}</p>
        {detail && <p className="truncate text-xs text-muted">{detail}</p>}
      </div>
    </div>
  );
}

/** Totals across all instances; hidden until there is something to show. */
export function PlayStats({ instances }: { instances: Instance[] }) {
  const { t, i18n } = useTranslation();
  const total = instances.reduce((sum, instance) => sum + instance.playTimeSecs, 0);
  if (total < 60) return null;
  const favorite = instances.reduce((best, instance) => (instance.playTimeSecs > best.playTimeSecs ? instance : best));

  return (
    <section className="grid gap-4 sm:grid-cols-3">
      <Stat icon={Clock} shift={0} label={t("stats.total")} value={formatPlayTime(total, i18n.language)} />
      <Stat
        icon={Crown}
        shift={60}
        label={t("stats.favorite")}
        value={favorite.name}
        detail={formatPlayTime(favorite.playTimeSecs, i18n.language)}
      />
      <Stat icon={Layers} shift={120} label={t("stats.instances")} value={String(instances.length)} />
    </section>
  );
}
