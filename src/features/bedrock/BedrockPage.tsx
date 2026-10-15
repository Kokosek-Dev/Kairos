import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Blocks, ExternalLink, FilePlus2, Play, Plus, Trash2, X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { PageHeader } from "@/components/shell/PageHeader";
import { Reveal } from "@/components/Reveal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  addBedrockPack,
  createBedrockInstance,
  deleteBedrockInstance,
  listBedrockInstances,
  playBedrockInstance,
  removeBedrockPack,
  type BedrockInstance,
} from "@/lib/tauri/bedrock";
import { errorMessage } from "@/lib/tauri/errors";
import { openExternalLink } from "@/lib/tauri/modrinth";
import { bedrockInstalled } from "@/lib/tauri/servers";

const INSTANCES_KEY = ["bedrock-instances"];
const ADDONS_URL = "https://www.curseforge.com/minecraft-bedrock/addons";

function InstanceCard({ instance, index, installed }: { instance: BedrockInstance; index: number; installed: boolean }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const refresh = () => queryClient.invalidateQueries({ queryKey: INSTANCES_KEY });
  const failed = (error: unknown) => toast.error(errorMessage(t, error));

  const play = useMutation({ mutationFn: () => playBedrockInstance(instance.id), onSuccess: () => toast.success(t("bedrock.ready")), onError: failed });
  const add = useMutation({ mutationFn: () => addBedrockPack(instance.id), onSuccess: refresh, onError: failed });
  const remove = useMutation({ mutationFn: (folder: string) => removeBedrockPack(instance.id, folder), onSuccess: refresh, onError: failed });
  const destroy = useMutation({ mutationFn: () => deleteBedrockInstance(instance.id), onSuccess: refresh, onError: failed });

  return (
    <Reveal index={index} className="panel space-y-4 p-4">
      <div className="flex items-center gap-4">
        <div className="grid size-14 shrink-0 place-items-center rounded-xl bg-brand text-accent-foreground">
          <Blocks size={26} />
        </div>
        <div className="min-w-0 flex-1">
          <h3 className="truncate text-lg font-semibold">{instance.name}</h3>
          <p className="text-xs text-muted">{t("bedrock.packsCount", { count: instance.packs.length })}</p>
        </div>
        <Button disabled={!installed || play.isPending} onClick={() => play.mutate()} title={installed ? undefined : t("bedrock.notInstalled")}>
          <Play size={16} />
          {t("bedrock.play")}
        </Button>
        <Button variant="ghost" size="sm" className="size-9 px-0 hover:text-danger" aria-label={t("bedrock.delete")} title={t("bedrock.delete")} disabled={destroy.isPending} onClick={() => destroy.mutate()}>
          <Trash2 size={16} />
        </Button>
      </div>

      {instance.packs.length === 0 ? (
        <p className="rounded-lg border border-dashed p-3 text-sm text-muted">{t("bedrock.noPacks")}</p>
      ) : (
        <ul className="space-y-1.5">
          {instance.packs.map((pack) => (
            <li key={pack.folder} className="flex items-center gap-3 rounded-lg border bg-surface/50 px-3 py-2 text-sm">
              <span className="min-w-0 flex-1 truncate font-medium">{pack.name}</span>
              <span className="rounded-full bg-accent/15 px-2 py-0.5 text-[11px] font-medium text-accent">{t(`bedrock.${pack.kind}`)}</span>
              <span className="font-mono text-xs text-muted">{pack.version}</span>
              <Button variant="ghost" size="sm" className="size-7 px-0 hover:text-danger" aria-label={t("bedrock.removePack")} title={t("bedrock.removePack")} disabled={remove.isPending} onClick={() => remove.mutate(pack.folder)}>
                <X size={14} />
              </Button>
            </li>
          ))}
        </ul>
      )}

      <div className="flex flex-wrap gap-2">
        <Button variant="secondary" size="sm" disabled={add.isPending} onClick={() => add.mutate()}>
          <FilePlus2 size={15} />
          {t("bedrock.addPack")}
        </Button>
        <Button variant="ghost" size="sm" onClick={() => void openExternalLink(ADDONS_URL)}>
          <ExternalLink size={14} />
          {t("bedrock.browse")}
        </Button>
      </div>
    </Reveal>
  );
}

export function BedrockPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const instances = useQuery({ queryKey: INSTANCES_KEY, queryFn: listBedrockInstances });
  const installed = useQuery({ queryKey: ["bedrock-installed"], queryFn: bedrockInstalled, staleTime: Infinity });
  const [name, setName] = useState("");

  const create = useMutation({
    mutationFn: () => createBedrockInstance(name),
    onSuccess: async () => {
      setName("");
      await queryClient.invalidateQueries({ queryKey: INSTANCES_KEY });
    },
    onError: (error) => toast.error(errorMessage(t, error)),
  });

  return (
    <div className="space-y-6 pb-6">
      <PageHeader title={t("nav.bedrock")} subtitle={t("bedrock.subtitle")} />

      <p className="rounded-xl border bg-surface/50 p-4 text-sm text-muted">{installed.data === false ? t("bedrock.notInstalled") : t("bedrock.note")}</p>

      <form
        className="panel flex flex-wrap items-end gap-3 p-4"
        onSubmit={(event) => {
          event.preventDefault();
          create.mutate();
        }}
      >
        <label className="min-w-56 flex-1 text-sm">
          <span className="mb-1 block text-muted">{t("bedrock.name")}</span>
          <Input value={name} maxLength={48} onChange={(event) => setName(event.target.value)} />
        </label>
        <Button type="submit" disabled={create.isPending || name.trim() === ""}>
          <Plus size={16} />
          {t("bedrock.create")}
        </Button>
      </form>

      {instances.data?.length === 0 && (
        <div className="grid place-items-center rounded-xl border border-dashed py-14 text-center">
          <Blocks size={36} className="mb-3 text-accent" />
          <p className="font-medium">{t("bedrock.empty")}</p>
          <p className="mt-1 max-w-md text-muted">{t("bedrock.emptyHint")}</p>
        </div>
      )}
      <div className="grid gap-4">
        {instances.data?.map((instance, index) => (
          <InstanceCard key={instance.id} instance={instance} index={index} installed={installed.data === true} />
        ))}
      </div>
    </div>
  );
}
