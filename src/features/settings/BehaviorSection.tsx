import { SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { useMascotRequests } from "@/components/Mascot";
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
  const showMascot = useMascotRequests((state) => state.show);

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
        <div className="flex items-center gap-2 py-3">
          <span className="mr-2 text-muted">{t("behavior.mascotTry")}</span>
          <Button variant="secondary" size="sm" onClick={() => showMascot("run")}>
            {t("behavior.mascotRun")}
          </Button>
          <Button variant="secondary" size="sm" onClick={() => showMascot("peek")}>
            {t("behavior.mascotWave")}
          </Button>
          <Button variant="secondary" size="sm" onClick={() => showMascot("fly")}>
            {t("behavior.mascotFly")}
          </Button>
        </div>
      </div>
    </section>
  );
}
