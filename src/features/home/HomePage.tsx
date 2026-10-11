import { Boxes, LogIn, Plus, Puzzle, Shirt, VenetianMask } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate } from "react-router-dom";
import { HeroCube } from "@/components/HeroCube";
import { Button } from "@/components/ui/button";
import { useSignInStore } from "@/features/accounts/signInStore";
import { useIncognito } from "@/features/incognito/useIncognito";
import { usePreferences } from "@/features/settings/preferences";
import { PlayStats } from "@/features/home/PlayStats";
import { DeleteInstanceDialog } from "@/features/instances/DeleteInstanceDialog";
import { InstanceCard } from "@/features/instances/InstanceCard";
import { useInstances } from "@/features/instances/useInstances";
import { SkinHead } from "@/features/skins/SkinHead";
import { useActiveAccount, useProfile } from "@/features/skins/useProfile";
import type { Instance } from "@/lib/tauri/instances";

const RECENT_COUNT = 3;

function SignedOutHero() {
  const { t } = useTranslation();
  const startSignIn = useSignInStore((state) => state.start);
  return (
    <section className="relative overflow-hidden rounded-3xl border bg-gradient-to-br from-accent/25 via-surface to-accent-2/15 p-10">
      <div className="grid-pattern pointer-events-none absolute inset-0" />
      <HeroCube size={130} className="absolute right-16 top-1/2 hidden -translate-y-1/2 md:block" />
      <div className="animate-drift pointer-events-none absolute -right-16 -top-16 size-72 rounded-full bg-accent/25 blur-3xl" />
      <div className="animate-drift pointer-events-none absolute -bottom-24 left-1/3 size-56 rounded-full bg-accent-strong/20 blur-3xl [animation-delay:-7s]" />
      <h1 className="text-gradient relative text-4xl font-extrabold tracking-tight">{t("home.welcome")}</h1>
      <p className="relative mt-2 max-w-md text-muted">{t("home.signInHint")}</p>
      <Button size="lg" className="relative mt-6" onClick={() => void startSignIn()}>
        <LogIn size={18} />
        {t("home.signInButton")}
      </Button>
      <ul className="relative mt-8 flex flex-wrap gap-3 text-sm text-muted">
        {[
          [Shirt, t("home.perks.skins")],
          [Boxes, t("home.perks.instances")],
          [Puzzle, t("home.perks.mods")],
        ].map(([Icon, label]) => (
          <li key={label as string} className="flex items-center gap-2 rounded-full border bg-background/40 px-3 py-1.5">
            <Icon size={14} className="text-accent" />
            {label as string}
          </li>
        ))}
      </ul>
    </section>
  );
}

function IncognitoHero() {
  const { t } = useTranslation();
  const update = usePreferences((state) => state.update);
  return (
    <section className="relative overflow-hidden rounded-3xl border bg-surface/80 p-10 text-center">
      <div className="grid-pattern pointer-events-none absolute inset-0" />
      <VenetianMask size={72} className="relative mx-auto text-accent drop-shadow-[0_0_18px_var(--accent)]" />
      <h1 className="text-gradient relative mt-4 text-3xl font-extrabold tracking-tight">{t("incognito.heroTitle")}</h1>
      <p className="relative mx-auto mt-2 max-w-lg text-muted">{t("incognito.heroBody")}</p>
      <Button variant="secondary" className="relative mt-6" onClick={() => update({ incognito: false })}>
        {t("incognito.leave")}
      </Button>
    </section>
  );
}

function WelcomeHero() {
  const { t } = useTranslation();
  const { active } = useActiveAccount();
  const profile = useProfile();
  return (
    <section className="relative overflow-hidden rounded-3xl border bg-gradient-to-br from-accent/20 via-surface to-accent-2/15 p-6">
      <div className="grid-pattern pointer-events-none absolute inset-0" />
      <div className="animate-drift pointer-events-none absolute -right-10 -top-10 size-60 rounded-full bg-accent-strong/20 blur-3xl" />
      <div className="relative flex items-center gap-5">
        <SkinHead url={profile.data?.skinUrl} name={active?.username ?? ""} size={64} className="size-16 rounded-xl" />
        <div className="min-w-0 flex-1">
          <p className="text-sm text-muted">{t("home.welcomeBack")}</p>
          <h1 className="text-gradient truncate text-4xl font-extrabold tracking-tight">{profile.data?.name ?? active?.username}</h1>
        </div>
        <Button variant="secondary" asChild>
          <Link to="/textures">
            <Shirt size={16} />
            {t("nav.textures")}
          </Link>
        </Button>
      </div>
    </section>
  );
}

export function HomePage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { active, isPending } = useActiveAccount();
  const incognito = useIncognito();
  const instances = useInstances();
  const [deleting, setDeleting] = useState<Instance | null>(null);
  const recent = instances.data?.slice(0, RECENT_COUNT) ?? [];

  return (
    <div className="space-y-8 pb-6">
      {incognito ? <IncognitoHero /> : isPending ? <div className="skeleton h-[390px] rounded-2xl" /> : active ? <WelcomeHero /> : <SignedOutHero />}

      <PlayStats instances={instances.data ?? []} />

      <section>
        <div className="mb-3 flex items-center justify-between">
          <h2 className="text-lg font-semibold">{t("home.continue")}</h2>
          <Button variant="ghost" size="sm" onClick={() => navigate("/library")}>
            {t("home.allInstances")}
          </Button>
        </div>
        {instances.isPending && <div className="skeleton h-40 rounded-xl" />}
        {instances.data?.length === 0 && (
          <div className="grid place-items-center rounded-xl border border-dashed py-10 text-center">
            <p className="mb-4 text-muted">{t("home.noInstances")}</p>
            <Button onClick={() => navigate("/library")}>
              <Plus size={16} />
              {t("instances.new")}
            </Button>
          </div>
        )}
        <div className="grid grid-cols-[repeat(auto-fill,minmax(18rem,1fr))] gap-4">
          {recent.map((instance, index) => (
            <InstanceCard key={instance.id} instance={instance} index={index} onDelete={setDeleting} />
          ))}
        </div>
      </section>

      <DeleteInstanceDialog instance={deleting} onClose={() => setDeleting(null)} />
    </div>
  );
}
