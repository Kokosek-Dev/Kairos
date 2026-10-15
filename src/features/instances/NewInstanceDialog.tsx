import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Brush } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Field, Input } from "@/components/ui/input";
import { Segmented } from "@/components/ui/segmented";
import { CreatePackDialog } from "@/features/textures/CreatePackDialog";
import { BEDROCK_KEY } from "@/features/bedrock/useBedrockInstances";
import { createBedrockInstance } from "@/lib/tauri/bedrock";
import { errorMessage } from "@/lib/tauri/errors";
import { createInstance, type IconInput, type LoaderKind } from "@/lib/tauri/instances";
import { PlaceholderTile, PresetTile } from "./InstanceIcon";
import { IconPicker } from "./IconPicker";
import { LoaderPicker } from "./LoaderPicker";
import { DEFAULT_PRESET } from "./presets";
import { useVersionManifest, VersionPicker } from "./VersionPicker";

interface NewInstanceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

type Edition = "java" | "bedrock";

const DEFAULT_ICON: IconInput = { kind: "preset", key: DEFAULT_PRESET.key };

function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function NewInstanceDialog({ open, onOpenChange }: NewInstanceDialogProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const navigate = useNavigate();
  const manifest = useVersionManifest();
  const [edition, setEdition] = useState<Edition>("java");
  const [bedrockName, setBedrockName] = useState("");
  const [name, setName] = useState("");
  const [nameEdited, setNameEdited] = useState(false);
  const [gameVersion, setGameVersion] = useState("");
  const [icon, setIcon] = useState<IconInput>(DEFAULT_ICON);
  const [loader, setLoader] = useState<LoaderKind>("vanilla");
  const [loaderVersion, setLoaderVersion] = useState<string | null>(null);
  const [texturesOpen, setTexturesOpen] = useState(false);

  const latestRelease = manifest.data?.latest.release;
  useEffect(() => {
    if (open && !gameVersion && latestRelease) setGameVersion(latestRelease);
  }, [open, gameVersion, latestRelease]);

  useEffect(() => {
    if (!nameEdited && gameVersion) setName(t("instances.defaultName", { version: gameVersion }));
  }, [nameEdited, gameVersion, t]);

  const reset = () => {
    setName("");
    setNameEdited(false);
    setGameVersion("");
    setIcon(DEFAULT_ICON);
    setLoader("vanilla");
    setLoaderVersion(null);
    setEdition("java");
    setBedrockName("");
    create.reset();
    createBedrock.reset();
  };

  const create = useMutation({
    mutationFn: () => createInstance({ name, gameVersion, icon, loader, loaderVersion }),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["instances"] });
      onOpenChange(false);
      reset();
    },
  });

  const createBedrock = useMutation({
    mutationFn: () => createBedrockInstance(bedrockName),
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: BEDROCK_KEY });
      onOpenChange(false);
      reset();
      navigate("/library");
    },
  });

  const bedrock = edition === "bedrock";
  const failure = bedrock ? createBedrock.error : create.error;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogTitle>{t("instances.newTitle")}</DialogTitle>
        <DialogDescription>{bedrock ? t("bedrock.newSubtitle") : t("instances.newSubtitle")}</DialogDescription>

        <div className="mt-4">
          <Segmented
            value={edition}
            onChange={setEdition}
            options={[
              { value: "java", label: t("servers.java") },
              { value: "bedrock", label: t("servers.bedrock") },
            ]}
          />
        </div>

        {bedrock ? (
          <div className="mt-5 flex-1 space-y-4">
            <Field label={t("instances.name")}>
              <Input value={bedrockName} maxLength={48} autoFocus onChange={(event) => setBedrockName(event.target.value)} />
            </Field>
            <p className="rounded-lg border bg-surface/50 p-3 text-sm text-muted">{t("bedrock.note")}</p>
          </div>
        ) : (
        <div className="mt-5 flex-1 space-y-5 overflow-y-auto pr-1">
          <div className="flex items-end gap-4">
            {icon.kind === "preset" ? (
              <PresetTile presetKey={icon.key} className="size-16" />
            ) : (
              <PlaceholderTile className="size-16" />
            )}
            <div className="min-w-0 flex-1">
              <Field label={t("instances.name")}>
                <Input
                  value={name}
                  maxLength={64}
                  onChange={(event) => {
                    setName(event.target.value);
                    setNameEdited(true);
                  }}
                />
              </Field>
            </div>
          </div>

          <Field label={t("instances.icon")} hint={icon.kind === "custom" ? fileName(icon.path) : undefined}>
            <IconPicker value={icon} onChange={setIcon} />
          </Field>

          <Field label={t("instances.gameVersion")}>
            <VersionPicker value={gameVersion} onChange={setGameVersion} />
          </Field>

          <Field label={t("instances.loaderLabel")}>
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
        </div>
        )}

        {failure && <p className="mt-3 text-sm text-danger">{errorMessage(t, failure)}</p>}

        <div className="mt-5 flex items-center justify-between gap-2">
          {bedrock ? (
            <span />
          ) : (
            <Button variant="ghost" onClick={() => setTexturesOpen(true)}>
              <Brush size={16} />
              {t("textures.createOwn")}
              <span className="rounded-full bg-accent/15 px-2 py-0.5 text-[11px] font-semibold text-accent">{t("textures.beta")}</span>
            </Button>
          )}
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t("common.cancel")}
          </Button>
          <Button
            disabled={bedrock ? !bedrockName.trim() || createBedrock.isPending : !name.trim() || !gameVersion || (loader !== "vanilla" && !loaderVersion) || create.isPending}
            onClick={() => (bedrock ? createBedrock.mutate() : create.mutate())}
          >
            {t("instances.create")}
          </Button>
          </div>
        </div>
      </DialogContent>
      <CreatePackDialog open={texturesOpen} onOpenChange={setTexturesOpen} />
    </Dialog>
  );
}
