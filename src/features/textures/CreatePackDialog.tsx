import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Field, Input } from "@/components/ui/input";
import { Select } from "@/components/ui/select";
import { errorMessage } from "@/lib/tauri/errors";
import { useInstances } from "@/features/instances/useInstances";
import { createTexturePack, listTextureVersions, setTexturePackInstances } from "@/lib/tauri/texturepacks";
import { InstanceChecklist } from "./InstanceChecklist";
import { useEverywhere } from "./useEverywhere";
import { PACKS_KEY } from "./usePacks";

interface CreatePackDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CreatePackDialog({ open, onOpenChange }: CreatePackDialogProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [chosen, setChosen] = useState("");
  const [unchecked, setUnchecked] = useState<string[]>([]);
  const instances = useInstances();
  const everywhere = useEverywhere();
  const versions = useQuery({ queryKey: ["texture-versions"], queryFn: listTextureVersions, enabled: open });

  const available = [...(versions.data ?? [])].sort((a, b) => b.localeCompare(a, undefined, { numeric: true }));
  const gameVersion = chosen || available[0] || "";
  const matching = (instances.data ?? []).filter((instance) => instance.gameVersion === gameVersion);
  const selected = matching.map((instance) => instance.id).filter((id) => !unchecked.includes(id));

  const create = useMutation({
    mutationFn: async () => {
      const pack = await createTexturePack(name, gameVersion);
      return !everywhere && selected.length > 0 ? setTexturePackInstances(pack.id, selected) : pack;
    },
    onSuccess: async (pack) => {
      await queryClient.invalidateQueries({ queryKey: PACKS_KEY });
      onOpenChange(false);
      setName("");
      setUnchecked([]);
      navigate(`/textures/${pack.id}`);
    },
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(28rem,calc(100vw-2rem))]">
        <DialogTitle>{t("textures.createTitle")}</DialogTitle>
        <DialogDescription>{t("textures.createSubtitle")}</DialogDescription>

        <div className="mt-5 space-y-4">
          <Field label={t("textures.name")}>
            <Input value={name} maxLength={48} autoFocus onChange={(event) => setName(event.target.value)} />
          </Field>
          <Field label={t("instances.gameVersion")}>
            {available.length === 0 && !versions.isPending ? (
              <p className="text-sm text-muted">
                {t("textures.noVersions")}{" "}
                <Link to="/library" className="text-accent hover:underline" onClick={() => onOpenChange(false)}>
                  {t("nav.library")}
                </Link>
              </p>
            ) : (
              <Select className="w-full" value={gameVersion} onChange={(event) => setChosen(event.target.value)}>
                {available.map((version) => (
                  <option key={version} value={version}>
                    {version}
                  </option>
                ))}
              </Select>
            )}
          </Field>
        </div>

        <div className="mt-4">
          <p className="mb-2 text-sm font-medium">{t("textures.useIn")}</p>
          {everywhere ? (
            <p className="text-sm text-muted">{t("textures.everywhereNote", { version: gameVersion })}</p>
          ) : (
            <InstanceChecklist
              instances={instances.data ?? []}
              gameVersion={gameVersion}
              selected={selected}
              onChange={(ids) => setUnchecked(matching.map((instance) => instance.id).filter((id) => !ids.includes(id)))}
            />
          )}
        </div>

        {create.isError && <p className="mt-3 text-sm text-danger">{errorMessage(t, create.error)}</p>}

        <div className="mt-6 flex justify-end gap-2">
          <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t("common.cancel")}
          </Button>
          <Button disabled={!name.trim() || !gameVersion || create.isPending} onClick={() => create.mutate()}>
            {t("textures.create")}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
