import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { motion } from "framer-motion";
import { Boxes, LogIn, Plus, Puzzle, RotateCcw, Save, Shirt, Trash2, Upload } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { useSignInStore } from "@/features/accounts/signInStore";
import { DeleteInstanceDialog } from "@/features/instances/DeleteInstanceDialog";
import { InstanceCard } from "@/features/instances/InstanceCard";
import { useInstances } from "@/features/instances/useInstances";
import { AddSkinDialog } from "@/features/skins/AddSkinDialog";
import { SkinPreview } from "@/features/skins/SkinPreview";
import { SkinViewer } from "@/features/skins/SkinViewer";
import { SKINS_KEY, useActiveAccount, useProfile, useSetProfile } from "@/features/skins/useProfile";
import { errorMessage } from "@/lib/tauri/errors";
import type { Instance } from "@/lib/tauri/instances";
import { applySkin, listSkins, removeSkin, resetSkin, saveCurrentSkin, setCape, type SavedSkin } from "@/lib/tauri/skins";

const RECENT_COUNT = 3;
const VIEWER = { width: 240, height: 340 };

function SignedOutHero() {
  const { t } = useTranslation();
  const startSignIn = useSignInStore((state) => state.start);
  return (
    <section className="relative overflow-hidden rounded-2xl border bg-gradient-to-br from-accent/20 via-surface to-surface p-10">
      <div className="pointer-events-none absolute -right-16 -top-16 size-64 rounded-full bg-accent/20 blur-3xl" />
      <h1 className="text-3xl font-bold tracking-tight">{t("home.welcome")}</h1>
      <p className="mt-2 max-w-md text-muted">{t("home.signInHint")}</p>
      <Button size="lg" className="mt-6" onClick={() => void startSignIn()}>
        <LogIn size={18} />
        {t("home.signInButton")}
      </Button>
      <ul className="mt-8 flex flex-wrap gap-3 text-sm text-muted">
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

function SkinHero() {
  const { t } = useTranslation();
  const { active } = useActiveAccount();
  const profile = useProfile();
  const setProfile = useSetProfile();
  const queryClient = useQueryClient();
  const [adding, setAdding] = useState(false);
  const onError = (error: unknown) => toast.error(errorMessage(t, error));

  const cape = useMutation({
    mutationFn: (capeId: string | null) => setCape(capeId),
    onSuccess: (updated) => setProfile(updated),
    onError,
  });
  const reset = useMutation({
    mutationFn: resetSkin,
    onSuccess: (updated) => {
      setProfile(updated);
      toast.success(t("skins.resetDone"));
    },
    onError,
  });
  const keep = useMutation({
    mutationFn: () => saveCurrentSkin(profile.data?.name ?? "Skin"),
    onSuccess: async (skin) => {
      await queryClient.invalidateQueries({ queryKey: SKINS_KEY });
      toast.success(t("skins.added", { name: skin.name }));
    },
    onError,
  });

  const data = profile.data;
  const activeCape = data?.capes.find((entry) => entry.active) ?? null;

  return (
    <section className="relative overflow-hidden rounded-2xl border bg-gradient-to-br from-accent/15 via-surface to-surface">
      <div className="pointer-events-none absolute -left-10 bottom-0 size-72 rounded-full bg-accent/15 blur-3xl" />
      <div className="relative flex flex-wrap items-center gap-8 p-6">
        <div className="grid shrink-0 place-items-center" style={{ width: VIEWER.width, height: VIEWER.height }}>
          {profile.isPending && <div className="skeleton h-full w-40 rounded-xl" />}
          {data && (
            <SkinViewer
              skinUrl={data.skinUrl}
              variant={data.variant}
              capeUrl={activeCape?.url ?? null}
              width={VIEWER.width}
              height={VIEWER.height}
            />
          )}
          {profile.isError && <p className="px-4 text-center text-sm text-muted">{t("home.profileFailed")}</p>}
        </div>

        <div className="min-w-64 flex-1">
          <p className="text-sm text-muted">{t("home.welcomeBack")}</p>
          <h1 className="text-3xl font-bold tracking-tight">{data?.name ?? active?.username}</h1>
          {data && (
            <p className="mt-1 text-xs text-muted">
              {t(`skins.variant.${data.variant}`)} · {activeCape ? activeCape.alias : t("skins.noCape")}
            </p>
          )}
          {profile.isError && (
            <div className="mt-3">
              <p className="mb-2 text-sm text-danger">{errorMessage(t, profile.error)}</p>
              <Button variant="secondary" size="sm" onClick={() => void profile.refetch()}>
                {t("versions.retry")}
              </Button>
            </div>
          )}

          {data && (
            <div className="mt-6 flex flex-wrap gap-2">
              <Button onClick={() => setAdding(true)}>
                <Upload size={16} />
                {t("skins.addButton")}
              </Button>
              <Button variant="secondary" disabled={keep.isPending || !data.skinUrl} onClick={() => keep.mutate()}>
                <Save size={16} />
                {t("skins.saveCurrent")}
              </Button>
              <Button variant="secondary" disabled={reset.isPending} onClick={() => reset.mutate()}>
                <RotateCcw size={16} />
                {t("skins.reset")}
              </Button>
            </div>
          )}

          {data && data.capes.length > 0 && (
            <label className="mt-5 flex items-center gap-3 text-sm">
              <span className="text-muted">{t("skins.cape")}</span>
              <Select
                value={activeCape?.id ?? ""}
                disabled={cape.isPending}
                onChange={(event) => cape.mutate(event.target.value || null)}
              >
                <option value="">{t("skins.noCape")}</option>
                {data.capes.map((entry) => (
                  <option key={entry.id} value={entry.id}>
                    {entry.alias}
                  </option>
                ))}
              </Select>
            </label>
          )}
        </div>
      </div>
      <AddSkinDialog open={adding} onOpenChange={setAdding} />
    </section>
  );
}

function SkinLibrary({ signedIn }: { signedIn: boolean }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const setProfile = useSetProfile();
  const [adding, setAdding] = useState(false);
  const skins = useQuery({ queryKey: SKINS_KEY, queryFn: listSkins });
  const onError = (error: unknown) => toast.error(errorMessage(t, error));

  const apply = useMutation({
    mutationFn: (skin: SavedSkin) => applySkin(skin.id),
    onSuccess: (profile, skin) => {
      setProfile(profile);
      toast.success(t("skins.applied", { name: skin.name }));
    },
    onError,
  });
  const remove = useMutation({
    mutationFn: (skin: SavedSkin) => removeSkin(skin.id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SKINS_KEY }),
    onError,
  });

  return (
    <section>
      <div className="mb-3 flex items-center justify-between">
        <h2 className="flex items-center gap-2 text-lg font-semibold">
          <Shirt size={18} className="text-accent" />
          {t("skins.library")}
        </h2>
        <Button variant="secondary" size="sm" onClick={() => setAdding(true)}>
          <Plus size={14} />
          {t("skins.addButton")}
        </Button>
      </div>

      {skins.isPending && (
        <div className="grid grid-cols-[repeat(auto-fill,minmax(9.5rem,1fr))] gap-3">
          {Array.from({ length: 4 }, (_, index) => (
            <div key={index} className="skeleton h-56 rounded-xl" />
          ))}
        </div>
      )}
      {skins.data?.length === 0 && (
        <div className="rounded-xl border border-dashed p-8 text-center text-muted">{t("skins.libraryEmpty")}</div>
      )}
      <div className="grid grid-cols-[repeat(auto-fill,minmax(9.5rem,1fr))] gap-3">
        {skins.data?.map((skin) => (
          <motion.article
            key={skin.id}
            layout
            initial={{ opacity: 0, scale: 0.97 }}
            animate={{ opacity: 1, scale: 1 }}
            className="group relative flex flex-col items-center gap-3 rounded-xl border bg-surface p-4 transition-colors hover:border-accent/40"
          >
            <button
              type="button"
              aria-label={t("skins.delete")}
              title={t("skins.delete")}
              onClick={() => remove.mutate(skin)}
              className="absolute right-2 top-2 rounded-md p-1 text-muted opacity-0 transition-opacity hover:text-danger group-hover:opacity-100"
            >
              <Trash2 size={14} />
            </button>
            <SkinPreview src={skin.dataUrl} variant={skin.variant} scale={5} />
            <div className="w-full text-center">
              <p className="truncate font-medium">{skin.name}</p>
              <p className="text-xs text-muted">{t(`skins.variant.${skin.variant}`)}</p>
            </div>
            <Button
              size="sm"
              className="w-full"
              disabled={!signedIn || apply.isPending}
              title={signedIn ? undefined : t("home.signInToApply")}
              onClick={() => apply.mutate(skin)}
            >
              {t("skins.apply")}
            </Button>
          </motion.article>
        ))}
      </div>
      <AddSkinDialog open={adding} onOpenChange={setAdding} />
    </section>
  );
}

export function HomePage() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { active, isPending } = useActiveAccount();
  const instances = useInstances();
  const [deleting, setDeleting] = useState<Instance | null>(null);
  const recent = instances.data?.slice(0, RECENT_COUNT) ?? [];

  return (
    <div className="space-y-8 pb-6">
      {isPending ? <div className="skeleton h-[390px] rounded-2xl" /> : active ? <SkinHero /> : <SignedOutHero />}

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

      <SkinLibrary signedIn={active !== null} />
      <DeleteInstanceDialog instance={deleting} onClose={() => setDeleting(null)} />
    </div>
  );
}
