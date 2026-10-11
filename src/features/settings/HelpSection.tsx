import { LifeBuoy } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { useProblemReport } from "./useProblemReport";

export function HelpSection() {
  const { t } = useTranslation();
  const report = useProblemReport();

  return (
    <section className="panel p-5">
      <h2 className="flex items-center gap-2 font-semibold">
        <LifeBuoy size={16} className="text-accent" />
        {t("help.title")}
      </h2>
      <p className="mb-4 mt-1 text-muted">{t("help.hint")}</p>
      <Button variant="secondary" disabled={report.isPending} onClick={() => report.mutate(null)}>
        {t("help.saveReport")}
      </Button>
    </section>
  );
}
