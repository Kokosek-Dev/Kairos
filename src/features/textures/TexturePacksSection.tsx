import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Brush, Pencil, Plus, Trash2 } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import { Reveal } from "@/components/Reveal";
import { Button } from "@/components/ui/button";
import { formatRelative } from "@/lib/format";
import { deleteTexturePack } from "@/lib/tauri/texturepacks";
import { CreatePackDialog } from "./CreatePackDialog";
import { PACKS_KEY, usePacks } from "./usePacks";

export function TexturePacksSection() {
  const { t, i18n } = useTranslation();
  const queryClient = useQueryClient();
  const packs = usePacks();
  const [creating, setCreating] = useState(false);
  const remove = useMutation({
    mutationFn: deleteTexturePack,
    onSuccess: () => queryClient.invalidateQueries({ queryKey: PACKS_KEY }),
  });

  return (
    <section className="mt-10">
      <div className="mb-3 flex items-center justify-between">
        <h2 className="flex items-center gap-2 text-lg font-semibold">
          <Brush size={18} className="text-accent" />
          {t("textures.packs")}
          <span className="rounded-full bg-accent/15 px-2 py-0.5 text-[11px] font-semibold text-accent">{t("textures.beta")}</span>
        </h2>
        <Button variant="secondary" size="sm" onClick={() => setCreating(true)}>
          <Plus size={14} />
          {t("textures.createOwn")}
        </Button>
      </div>
      {packs.data?.length === 0 && <div className="rounded-xl border border-dashed p-8 text-center text-muted">{t("textures.packsEmpty")}</div>}
      <div className="grid grid-cols-[repeat(auto-fill,minmax(16rem,1fr))] gap-3">
        {packs.data?.map((pack, index) => (
          <Reveal key={pack.id} index={index} className="panel flex items-center gap-3 p-4">
            <div className="min-w-0 flex-1">
              <p className="truncate font-medium">{pack.name}</p>
              <p className="text-xs text-muted">
                {pack.gameVersion} · {formatRelative(pack.createdAt, i18n.language)}
              </p>
            </div>
            <Button variant="secondary" size="sm" asChild>
              <Link to={`/textures/${pack.id}`}>
                <Pencil size={14} />
                {t("skins.edit")}
              </Link>
            </Button>
            <Button
              variant="ghost"
              size="sm"
              className="size-8 px-0 hover:text-danger"
              aria-label={t("textures.delete")}
              title={t("textures.delete")}
              onClick={() => remove.mutate(pack.id)}
            >
              <Trash2 size={15} />
            </Button>
          </Reveal>
        ))}
      </div>
      <CreatePackDialog open={creating} onOpenChange={setCreating} />
    </section>
  );
}
