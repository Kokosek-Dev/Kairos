import { useMutation, useQueryClient } from "@tanstack/react-query";
import { FileUp, User } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Field, Input } from "@/components/ui/input";
import { errorMessage } from "@/lib/tauri/errors";
import { addSkinFromFile, addSkinFromPlayer, pickSkinFile, type SkinVariant } from "@/lib/tauri/skins";
import { cn } from "@/lib/utils";
import { SKINS_KEY } from "./useProfile";

type Mode = "file" | "player";

function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function AddSkinDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [mode, setMode] = useState<Mode>("file");
  const [path, setPath] = useState<string | null>(null);
  const [name, setName] = useState("");
  const [player, setPlayer] = useState("");
  const [variant, setVariant] = useState<SkinVariant>("classic");

  const reset = () => {
    setPath(null);
    setName("");
    setPlayer("");
    setVariant("classic");
    add.reset();
  };

  const add = useMutation({
    mutationFn: () =>
      mode === "file" && path
        ? addSkinFromFile(path, name.trim() || null, variant)
        : addSkinFromPlayer(player.trim()),
    onSuccess: async (skin) => {
      await queryClient.invalidateQueries({ queryKey: SKINS_KEY });
      toast.success(t("skins.added", { name: skin.name }));
      onOpenChange(false);
      reset();
    },
  });

  const choose = async () => {
    const picked = await pickSkinFile();
    if (picked) {
      setPath(picked);
      if (!name) setName(fileName(picked).replace(/\.png$/i, ""));
    }
  };

  const ready = mode === "file" ? path !== null : player.trim().length > 0;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(30rem,calc(100vw-2rem))]">
        <DialogTitle>{t("skins.addTitle")}</DialogTitle>
        <DialogDescription>{t("skins.addSubtitle")}</DialogDescription>

        <div className="mt-4 flex gap-1 rounded-md bg-surface p-1">
          {(["file", "player"] as Mode[]).map((item) => (
            <button
              key={item}
              type="button"
              onClick={() => setMode(item)}
              className={cn(
                "flex h-8 flex-1 items-center justify-center gap-2 rounded-sm text-sm font-medium transition-colors",
                mode === item ? "bg-accent text-accent-foreground" : "text-muted hover:text-foreground",
              )}
            >
              {item === "file" ? <FileUp size={14} /> : <User size={14} />}
              {t(`skins.mode.${item}`)}
            </button>
          ))}
        </div>

        <div className="mt-5 space-y-4">
          {mode === "file" ? (
            <>
              <Button variant="secondary" className="w-full" onClick={() => void choose()}>
                <FileUp size={16} />
                {path ? fileName(path) : t("skins.chooseFile")}
              </Button>
              <Field label={t("skins.name")}>
                <Input value={name} maxLength={32} onChange={(event) => setName(event.target.value)} />
              </Field>
              <Field label={t("skins.model")} hint={t("skins.modelHint")}>
                <div className="flex gap-2">
                  {(["classic", "slim"] as SkinVariant[]).map((item) => (
                    <button
                      key={item}
                      type="button"
                      onClick={() => setVariant(item)}
                      className={cn(
                        "h-10 flex-1 rounded-md border text-sm font-medium transition-all",
                        variant === item ? "border-accent bg-accent/15 text-foreground" : "text-muted hover:text-foreground",
                      )}
                    >
                      {t(`skins.variant.${item}`)}
                    </button>
                  ))}
                </div>
              </Field>
            </>
          ) : (
            <Field label={t("skins.playerName")} hint={t("skins.playerHint")}>
              <Input value={player} maxLength={16} placeholder="Notch" onChange={(event) => setPlayer(event.target.value)} />
            </Field>
          )}
        </div>

        {add.isError && <p className="mt-3 text-sm text-danger">{errorMessage(t, add.error)}</p>}

        <div className="mt-6 flex justify-end gap-2">
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t("common.cancel")}
          </Button>
          <Button disabled={!ready || add.isPending} onClick={() => add.mutate()}>
            {t("skins.add")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
