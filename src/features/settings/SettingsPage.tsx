import { Languages } from "lucide-react";
import { useTranslation } from "react-i18next";
import { LanguagePicker } from "@/components/LanguagePicker";
import { PageHeader } from "@/components/shell/PageHeader";

import { AppearanceSection } from "@/features/settings/AppearanceSection";
import { BehaviorSection } from "@/features/settings/BehaviorSection";
import { StorageSection } from "@/features/settings/StorageSection";
import { AccountsSection } from "@/features/accounts/AccountsSection";

export function SettingsPage() {
  const { t } = useTranslation();

  return (
    <>
      <PageHeader title={t("settings.title")} />
      <div className="max-w-3xl space-y-5">
        <AccountsSection />
        <AppearanceSection />
        <StorageSection />
        <BehaviorSection />
        <section className="panel p-5">
          <h2 className="flex items-center gap-2 font-semibold">
            <Languages size={16} className="text-accent" />
            {t("settings.language")}
          </h2>
          <p className="mb-4 mt-1 text-muted">{t("settings.languageHint")}</p>
          <LanguagePicker />
        </section>
      </div>
    </>
  );
}
