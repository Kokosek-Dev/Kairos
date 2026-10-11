import { useMutation, useQueryClient } from "@tanstack/react-query";
import { RotateCcw, Save, Upload } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Select } from "@/components/ui/select";
import { errorMessage } from "@/lib/tauri/errors";
import { resetSkin, saveCurrentSkin, setCape } from "@/lib/tauri/skins";
import { AddSkinDialog } from "./AddSkinDialog";
import { SkinViewer } from "./SkinViewer";
import { SKINS_KEY, useActiveAccount, useProfile, useSetProfile } from "./useProfile";

const VIEWER = { width: 240, height: 340 };

export function SkinHero() {
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
    <section className="relative overflow-hidden rounded-3xl border bg-gradient-to-br from-accent/20 via-surface to-accent-2/15">
      <div className="grid-pattern pointer-events-none absolute inset-0" />
      <div className="animate-drift pointer-events-none absolute -left-10 bottom-0 size-72 rounded-full bg-accent/20 blur-3xl" />
      <div className="animate-drift pointer-events-none absolute -right-10 -top-10 size-60 rounded-full bg-accent-strong/20 blur-3xl [animation-delay:-5s]" />
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
          <h1 className="text-gradient text-4xl font-extrabold tracking-tight">{data?.name ?? active?.username}</h1>
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
