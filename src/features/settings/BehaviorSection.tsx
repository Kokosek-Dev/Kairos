import { SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Switch } from "@/components/ui/switch";
import { usePreferences } from "./preferences";

function Row({ title, hint, checked, onChange }: { title: string; hint: string; checked: boolean; onChange: (checked: boolean) => void }) {
  return (
    <div className="flex items-center justify-between gap-6 py-3">
      <div>
        <p className="font-medium">{title}</p>
        <p className="text-muted">{hint}</p>
      </div>
      <Switch checked={checked} onChange={onChange} label={title} />
    </div>
  );
}

export function BehaviorSection() {
  const { t } = useTranslation();
  const { autoUpdateMods, mascot, update } = usePreferences();

  return (
    <section className="panel p-5">
      <h2 className="flex items-center gap-2 font-semibold">
        <SlidersHorizontal size={16} className="text-accent" />
        {t("behavior.title")}
      </h2>
      <div className="mt-2 divide-y">
        <Row
          title={t("behavior.autoUpdate")}
          hint={t("behavior.autoUpdateHint")}
          checked={autoUpdateMods}
          onChange={(checked) => update({ autoUpdateMods: checked })}
        />
        <Row
          title={t("behavior.mascot")}
          hint={t("behavior.mascotHint")}
          checked={mascot}
          onChange={(checked) => update({ mascot: checked })}
        />
      </div>
    </section>
  );
}
