import { Boxes } from "lucide-react";
import { useTranslation } from "react-i18next";
import { PageHeader } from "@/components/shell/PageHeader";

export function LibraryPage() {
  const { t } = useTranslation();

  return (
    <>
      <PageHeader title={t("library.title")} subtitle={t("library.subtitle")} />
      <div className="grid place-items-center rounded-xl border border-dashed py-20 text-center">
        <Boxes size={40} className="mb-4 text-accent" />
        <h2 className="text-lg font-semibold">{t("library.emptyTitle")}</h2>
        <p className="mt-1 text-muted">{t("library.emptyBody")}</p>
      </div>
    </>
  );
}
