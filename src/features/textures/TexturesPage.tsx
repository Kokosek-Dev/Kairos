import { useTranslation } from "react-i18next";
import { PageHeader } from "@/components/shell/PageHeader";
import { useIncognito } from "@/features/incognito/useIncognito";
import { SkinHero } from "@/features/skins/SkinHero";
import { SkinLibrary } from "@/features/skins/SkinLibrary";
import { useActiveAccount } from "@/features/skins/useProfile";
import { TexturePacksSection } from "./TexturePacksSection";

export function TexturesPage() {
  const { t } = useTranslation();
  const { active } = useActiveAccount();
  const incognito = useIncognito();
  const signedIn = active !== null && !incognito;

  return (
    <div className="space-y-8 pb-6">
      <PageHeader title={t("nav.textures")} subtitle={t("textures.pageSubtitle")} />
      {signedIn && <SkinHero />}
      <SkinLibrary signedIn={signedIn} />
      <TexturePacksSection />
    </div>
  );
}
