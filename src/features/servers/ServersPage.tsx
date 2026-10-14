import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Copy, ExternalLink, Plus, RefreshCw, Server, Trash2, Users } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { PageHeader } from "@/components/shell/PageHeader";
import { Reveal } from "@/components/Reveal";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Segmented } from "@/components/ui/segmented";
import { errorMessage } from "@/lib/tauri/errors";
import { addServer, bedrockInstalled, listServers, openBedrock, pingServer, removeServer, type Edition, type ServerEntry } from "@/lib/tauri/servers";
import { cn } from "@/lib/utils";

const SERVERS_KEY = ["servers"];
const EDITIONS: Edition[] = ["java", "bedrock"];
const REFRESH_MS = 60_000;
const GOOD_PING = 80;
const OK_PING = 200;

function pingColor(latency: number): string {
  if (latency < GOOD_PING) return "text-success";
  return latency < OK_PING ? "text-warning" : "text-danger";
}

function ServerCard({ server, index }: { server: ServerEntry; index: number }) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const status = useQuery({
    queryKey: ["server-status", server.edition, server.address],
    queryFn: () => pingServer(server.address, server.edition),
    staleTime: REFRESH_MS / 2,
    refetchInterval: REFRESH_MS,
    retry: false,
  });
  const remove = useMutation({
    mutationFn: () => removeServer(server.id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: SERVERS_KEY }),
  });
  const installed = useQuery({ queryKey: ["bedrock-installed"], queryFn: bedrockInstalled, staleTime: Infinity, enabled: server.edition === "bedrock" });
  const open = useMutation({ mutationFn: openBedrock, onError: (error) => toast.error(errorMessage(t, error)) });
  const data = status.data;
  const offline = status.isError;
  const bedrock = server.edition === "bedrock";

  const copy = async () => {
    await navigator.clipboard.writeText(server.address);
    toast.success(t("servers.copied"));
  };

  return (
    <Reveal index={index} className="panel flex gap-4 p-4">
      {data?.favicon ? (
        <img src={data.favicon} alt="" className="size-16 shrink-0 rounded-lg [image-rendering:pixelated]" draggable={false} />
      ) : (
        <div className="grid size-16 shrink-0 place-items-center rounded-lg bg-surface-raised text-muted">
          <Server size={26} />
        </div>
      )}
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <h3 className="truncate font-semibold">{server.name}</h3>
          {bedrock && <span className="rounded-full bg-accent/15 px-2 py-0.5 text-[11px] font-medium text-accent">{t("servers.bedrock")}</span>}
          {status.isPending && <span className="skeleton h-4 w-12 rounded-full" />}
          {offline && <span className="rounded-full bg-danger/15 px-2 py-0.5 text-[11px] font-medium text-danger">{t("servers.offline")}</span>}
          {data && (
            <span className="rounded-full bg-success/15 px-2 py-0.5 text-[11px] font-medium text-success">{t("servers.online")}</span>
          )}
        </div>
        <p className="truncate text-xs text-muted">{server.address}</p>
        {data && (
          <>
            <p className="mt-1.5 line-clamp-2 text-sm">{data.motd || t("servers.noDescription")}</p>
            <div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted">
              <span className="flex items-center gap-1.5">
                <Users size={13} />
                {data.playersOnline} / {data.playersMax}
              </span>
              {data.latencyMs !== null && <span className={cn("font-medium", pingColor(data.latencyMs))}>{t("servers.ping", { ms: data.latencyMs })}</span>}
              {data.version && <span className="truncate">{data.version}</span>}
            </div>
          </>
        )}
        {offline && <p className="mt-1.5 text-sm text-muted">{t("servers.unreachable")}</p>}
        {bedrock && (
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <Button size="sm" variant="secondary" disabled={installed.data !== true || open.isPending} onClick={() => open.mutate()}>
              <ExternalLink size={14} />
              {t("servers.openBedrock")}
            </Button>
            <p className="basis-full text-xs text-muted">{installed.data === false ? t("servers.bedrockMissing") : t("servers.bedrockHint")}</p>
          </div>
        )}
      </div>
      <div className="flex shrink-0 flex-col gap-1">
        <Button variant="ghost" size="sm" className="size-8 px-0" aria-label={t("servers.copy")} title={t("servers.copy")} onClick={() => void copy()}>
          <Copy size={15} />
        </Button>
        <Button variant="ghost" size="sm" className="size-8 px-0" aria-label={t("servers.refresh")} title={t("servers.refresh")} onClick={() => void status.refetch()}>
          <RefreshCw size={15} className={cn(status.isFetching && "animate-spin")} />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          className="size-8 px-0 hover:text-danger"
          aria-label={t("servers.remove")}
          title={t("servers.remove")}
          disabled={remove.isPending}
          onClick={() => remove.mutate()}
        >
          <Trash2 size={15} />
        </Button>
      </div>
    </Reveal>
  );
}

export function ServersPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const servers = useQuery({ queryKey: SERVERS_KEY, queryFn: listServers });
  const [name, setName] = useState("");
  const [address, setAddress] = useState("");
  const [edition, setEdition] = useState<Edition>("java");

  const add = useMutation({
    mutationFn: () => addServer(name, address, edition),
    onSuccess: async () => {
      setName("");
      setAddress("");
      await queryClient.invalidateQueries({ queryKey: SERVERS_KEY });
    },
    onError: (error) => toast.error(errorMessage(t, error)),
  });

  return (
    <div className="space-y-6 pb-6">
      <PageHeader title={t("nav.servers")} subtitle={t("servers.subtitle")} />

      <form
        className="panel flex flex-wrap items-end gap-3 p-4"
        onSubmit={(event) => {
          event.preventDefault();
          add.mutate();
        }}
      >
        <div className="text-sm">
          <span className="mb-1 block text-muted">{t("servers.edition")}</span>
          <Segmented value={edition} onChange={setEdition} options={EDITIONS.map((value) => ({ value, label: t(`servers.${value}`) }))} />
        </div>
        <label className="min-w-40 flex-1 text-sm">
          <span className="mb-1 block text-muted">{t("servers.name")}</span>
          <Input value={name} maxLength={48} onChange={(event) => setName(event.target.value)} />
        </label>
        <label className="min-w-56 flex-[2] text-sm">
          <span className="mb-1 block text-muted">{t("servers.address")}</span>
          <Input value={address} placeholder={edition === "java" ? "mc.example.com" : "play.example.net"} maxLength={253} onChange={(event) => setAddress(event.target.value)} />
        </label>
        <Button type="submit" disabled={add.isPending || name.trim() === "" || address.trim() === ""}>
          <Plus size={16} />
          {t("servers.add")}
        </Button>
      </form>

      {servers.data?.length === 0 && (
        <div className="grid place-items-center rounded-xl border border-dashed py-14 text-center">
          <Server size={36} className="mb-3 text-accent" />
          <p className="font-medium">{t("servers.empty")}</p>
          <p className="mt-1 max-w-sm text-muted">{t("servers.emptyHint")}</p>
        </div>
      )}
      <div className="grid grid-cols-[repeat(auto-fill,minmax(22rem,1fr))] gap-4">
        {servers.data?.map((server, index) => (
          <ServerCard key={server.id} server={server} index={index} />
        ))}
      </div>
    </div>
  );
}
