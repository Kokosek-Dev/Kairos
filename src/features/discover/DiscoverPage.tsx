import { useTranslation } from "react-i18next";
import { PageHeader } from "@/components/shell/PageHeader";

export function DiscoverPage() {
  const { t } = useTranslation();

  return (
    <>
      <PageHeader title={t("discover.title")} subtitle={t("discover.subtitle")} />
      <p className="text-muted">{t("discover.comingSoon")}</p>
    </>
  );
}
