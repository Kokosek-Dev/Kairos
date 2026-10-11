import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { motion } from "framer-motion";
import { Paintbrush, Plus, Shirt, Trash2 } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { errorMessage } from "@/lib/tauri/errors";
import { applySkin, listSkins, removeSkin, type SavedSkin } from "@/lib/tauri/skins";
import { AddSkinDialog } from "./AddSkinDialog";
import { SkinPreview } from "./SkinPreview";
import { SKINS_KEY, useSetProfile } from "./useProfile";

export function SkinLibrary({ signedIn }: { signedIn: boolean }) {
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
        <div className="flex gap-2">
          <Button variant="secondary" size="sm" asChild>
            <Link to="/skins/editor">
              <Paintbrush size={14} />
              {t("skins.create")}
            </Link>
          </Button>
          <Button variant="secondary" size="sm" onClick={() => setAdding(true)}>
            <Plus size={14} />
            {t("skins.addButton")}
          </Button>
        </div>
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
            <Button variant="ghost" size="sm" className="w-full" asChild>
              <Link to={`/skins/editor?skin=${skin.id}`}>
                <Paintbrush size={14} />
                {t("skins.edit")}
              </Link>
            </Button>
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
