import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ArrowLeft, Boxes, Copy, FileDown, FolderOpen, Save, Trash2 } from "lucide-react";
import { useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Link, useNavigate, useParams } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Field, Input, Textarea } from "@/components/ui/input";
import { isBusy, useGameStore } from "@/features/launch/gameStore";
import { joinArgs, splitArgs } from "@/lib/args";
import { formatMemory } from "@/lib/format";
import { exportMrpack, pickMrpackDestination } from "@/lib/tauri/modrinth";
import { errorMessage } from "@/lib/tauri/errors";
import {
  duplicateInstance,
  getMemoryAdvice,
  openInstanceFolder,
  pickJavaFile,
  setInstanceIcon,
  updateInstance,
  type IconInput,
  type Instance,
  type LoaderKind,
} from "@/lib/tauri/instances";
import { DeleteInstanceDialog } from "./DeleteInstanceDialog";
import { PlayButton } from "./PlayButton";
import { IconPicker } from "./IconPicker";
import { InstanceIcon } from "./InstanceIcon";
import { LoaderPicker } from "./LoaderPicker";
import { INSTANCES_KEY, useInstance } from "./useInstances";
import { VersionPicker } from "./VersionPicker";

const MIN_MEMORY_MB = 512;
const FALLBACK_MAX_MB = 16384;
const MEMORY_STEP_MB = 256;

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="space-y-4 panel p-5">
      <h2 className="font-semibold">{title}</h2>
      {children}
    </section>
  );
}

export function InstanceSettingsPage() {
  const { id = "" } = useParams();
  const { t } = useTranslation();
  const instance = useInstance(id);

  if (instance.isPending) return <div className="skeleton h-96 max-w-3xl rounded-xl" />;
  if (instance.isError) {
    return (
      <div className="max-w-3xl space-y-3">
        <p className="text-danger">{errorMessage(t, instance.error)}</p>
        <Button asChild variant="secondary">
          <Link to="/library">{t("instances.back")}</Link>
        </Button>
      </div>
    );
  }
  return <SettingsForm key={instance.data.id} instance={instance.data} />;
}

function SettingsForm({ instance }: { instance: Instance }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const { instanceId, phase } = useGameStore();

  const [name, setName] = useState(instance.name);
  const [gameVersion, setGameVersion] = useState(instance.gameVersion);
  const [loader, setLoader] = useState<LoaderKind>(instance.loader);
  const [loaderVersion, setLoaderVersion] = useState(instance.loaderVersion);
  const [memoryMb, setMemoryMb] = useState(instance.memoryMb);
  const [jvmText, setJvmText] = useState(joinArgs(instance.jvmArgs));
  const [javaPath, setJavaPath] = useState(instance.javaPath);
  const [deleting, setDeleting] = useState(false);

  const advice = useQuery({
    queryKey: ["memory-advice", loader !== "vanilla"],
    queryFn: () => getMemoryAdvice(loader !== "vanilla"),
    staleTime: Infinity,
  });
  const sliderMax = advice.data?.maxMb ?? FALLBACK_MAX_MB;

  const thisRunning = instanceId === instance.id && isBusy(phase);
  const dirty =
    name !== instance.name ||
    gameVersion !== instance.gameVersion ||
    loader !== instance.loader ||
    loaderVersion !== instance.loaderVersion ||
    memoryMb !== instance.memoryMb ||
    jvmText !== joinArgs(instance.jvmArgs) ||
    javaPath !== instance.javaPath;

  const refresh = (updated: Instance) => {
    queryClient.setQueryData(["instance", instance.id], updated);
    void queryClient.invalidateQueries({ queryKey: INSTANCES_KEY });
  };

  const save = useMutation({
    mutationFn: () =>
      updateInstance(instance.id, {
        name,
        gameVersion,
        loader,
        loaderVersion: loader === "vanilla" ? null : loaderVersion,
        memoryMb,
        jvmArgs: splitArgs(jvmText),
        javaPath,
      }),
    onSuccess: refresh,
  });

  const changeIcon = useMutation({
    mutationFn: (icon: IconInput) => setInstanceIcon(instance.id, icon),
    onSuccess: (updated) => {
      queryClient.removeQueries({ queryKey: ["instance-icon", instance.id] });
      refresh(updated);
    },
  });

  const duplicate = useMutation({
    mutationFn: () => duplicateInstance(instance.id),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: INSTANCES_KEY });
      navigate("/library");
    },
  });

  const exportPack = useMutation({
    mutationFn: async () => {
      const destination = await pickMrpackDestination(`${instance.name}.mrpack`);
      if (destination) await exportMrpack(instance.id, destination, instance.name, "1.0.0");
    },
  });

  const reset = () => {
    setName(instance.name);
    setGameVersion(instance.gameVersion);
    setLoader(instance.loader);
    setLoaderVersion(instance.loaderVersion);
    setMemoryMb(instance.memoryMb);
    setJvmText(joinArgs(instance.jvmArgs));
    setJavaPath(instance.javaPath);
    save.reset();
  };

  const browseJava = async () => {
    const path = await pickJavaFile();
    if (path) setJavaPath(path);
  };

  const currentIcon: IconInput | null = instance.icon.kind === "preset" ? instance.icon : null;
  const error = save.error ?? changeIcon.error ?? duplicate.error ?? exportPack.error;

  return (
    <div className="max-w-6xl space-y-5 pb-24">
      <Link to="/library" className="inline-flex items-center gap-2 text-muted transition-colors hover:text-foreground">
        <ArrowLeft size={16} />
        {t("instances.back")}
      </Link>

      <header className="flex items-center gap-4">
        <InstanceIcon icon={instance.icon} instanceId={instance.id} className="size-16" iconSize={30} />
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-2xl font-bold tracking-tight">{instance.name}</h1>
          <p className="text-muted">
            {instance.gameVersion} · {t(`instances.loader.${instance.loader}`)}
            {instance.loaderVersion ? ` ${instance.loaderVersion}` : ""}
          </p>
        </div>
        <Button variant="secondary" size="lg" asChild>
          <Link to={`/instances/${instance.id}/content`}>
            <Boxes size={18} />
            {t("content.open")}
          </Link>
        </Button>
        <PlayButton instance={instance} />
      </header>

      <div className="grid items-start gap-5 xl:grid-cols-2">
      <Section title={t("instances.general")}>
        <Field label={t("instances.name")}>
          <Input value={name} maxLength={64} onChange={(event) => setName(event.target.value)} />
        </Field>
        <Field label={t("instances.icon")}>
          <IconPicker value={currentIcon} onChange={(icon) => changeIcon.mutate(icon)} />
        </Field>
        <Field label={t("instances.gameVersion")} hint={t("instances.versionWarning")}>
          <VersionPicker value={gameVersion} onChange={setGameVersion} />
        </Field>
        <Field label={t("instances.loaderLabel")} hint={t("instances.loaderWarning")}>
          <LoaderPicker
            gameVersion={gameVersion}
            kind={loader}
            version={loaderVersion}
            onChange={(kind, version) => {
              setLoader(kind);
              setLoaderVersion(version);
            }}
          />
        </Field>
      </Section>

      <Section title={t("instances.memory")}>
        <div className="flex items-center gap-4">
          <input
            type="range"
            min={MIN_MEMORY_MB}
            max={sliderMax}
            step={MEMORY_STEP_MB}
            value={Math.min(memoryMb, sliderMax)}
            onChange={(event) => setMemoryMb(Number(event.target.value))}
            className="h-2 flex-1 cursor-pointer accent-accent"
          />
          <span className="w-16 text-right font-semibold">{formatMemory(memoryMb)}</span>
        </div>
        {advice.data && (
          <div className="flex flex-wrap items-center gap-3 text-sm">
            <p className={memoryMb > advice.data.maxMb ? "text-danger" : "text-muted"}>
              {memoryMb > advice.data.maxMb
                ? t("instances.memoryTooMuch", { max: formatMemory(advice.data.maxMb) })
                : t("instances.memoryAdvice", { total: formatMemory(advice.data.totalMb), recommended: formatMemory(advice.data.recommendedMb) })}
            </p>
            {memoryMb !== advice.data.recommendedMb && (
              <Button variant="secondary" size="sm" onClick={() => setMemoryMb(advice.data.recommendedMb)}>
                {t("instances.memoryUse")}
              </Button>
            )}
          </div>
        )}
        <Field label={t("instances.memoryMb")}>
          <Input
            type="number"
            min={MIN_MEMORY_MB}
            step={MEMORY_STEP_MB}
            value={memoryMb}
            onChange={(event) => setMemoryMb(Number(event.target.value))}
            className="w-40"
          />
        </Field>
      </Section>

      <Section title={t("instances.advanced")}>
        <Field label={t("instances.jvmArgs")} hint={t("instances.jvmArgsHint")}>
          <Textarea value={jvmText} onChange={(event) => setJvmText(event.target.value)} spellCheck={false} />
        </Field>
        <Field label={t("instances.javaPath")} hint={t("instances.javaPathHint")}>
          <div className="flex gap-2">
            <Input
              value={javaPath ?? ""}
              placeholder={t("instances.javaAuto")}
              onChange={(event) => setJavaPath(event.target.value || null)}
            />
            <Button variant="secondary" onClick={() => void browseJava()}>
              {t("instances.browse")}
            </Button>
            {javaPath !== null && (
              <Button variant="ghost" onClick={() => setJavaPath(null)}>
                {t("instances.auto")}
              </Button>
            )}
          </div>
        </Field>
      </Section>

      <Section title={t("instances.manage")}>
        <div className="flex flex-wrap gap-2">
          <Button variant="secondary" onClick={() => void openInstanceFolder(instance.id)}>
            <FolderOpen size={16} />
            {t("instances.openFolder")}
          </Button>
          <Button variant="secondary" disabled={exportPack.isPending} onClick={() => exportPack.mutate()}>
            <FileDown size={16} />
            {t("content.exportPack")}
          </Button>
          <Button variant="secondary" disabled={duplicate.isPending} onClick={() => duplicate.mutate()}>
            <Copy size={16} />
            {t("instances.duplicate")}
          </Button>
          <Button
            variant="secondary"
            className="text-danger hover:bg-danger/15"
            disabled={thisRunning}
            onClick={() => setDeleting(true)}
          >
            <Trash2 size={16} />
            {t("instances.delete")}
          </Button>
        </div>
      </Section>
      </div>

      {error && <p className="text-sm text-danger">{errorMessage(t, error)}</p>}

      {dirty && (
        <div className="glass fixed bottom-4 left-1/2 z-30 flex -translate-x-1/2 items-center gap-3 rounded-xl px-4 py-3 shadow-glow">
          <span className="text-sm text-muted">{t("instances.unsaved")}</span>
          <Button variant="ghost" size="sm" onClick={reset}>
            {t("instances.discard")}
          </Button>
          <Button
            size="sm"
            disabled={save.isPending || !name.trim() || (loader !== "vanilla" && !loaderVersion)}
            onClick={() => save.mutate()}
          >
            <Save size={14} />
            {t("instances.save")}
          </Button>
        </div>
      )}

      <DeleteInstanceDialog
        instance={deleting ? instance : null}
        onClose={() => setDeleting(false)}
        onDeleted={() => navigate("/library")}
      />
    </div>
  );
}
