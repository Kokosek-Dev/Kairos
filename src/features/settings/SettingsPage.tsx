import { useTranslation } from "react-i18next";
import { LanguagePicker } from "@/components/LanguagePicker";
import { PageHeader } from "@/components/shell/PageHeader";

export function SettingsPage() {
  const { t } = useTranslation();

  return (
    <>
      <PageHeader title={t("settings.title")} />
      <section className="rounded-xl border bg-surface p-5">
        <h2 className="font-semibold">{t("settings.language")}</h2>
        <p className="mb-4 mt-1 text-muted">{t("settings.languageHint")}</p>
        <LanguagePicker />
      </section>
    </>
  );
}
