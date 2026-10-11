import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { Brush } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Field, Input } from "@/components/ui/input";
import { CreatePackDialog } from "@/features/textures/CreatePackDialog";
import { errorMessage } from "@/lib/tauri/errors";
import { createInstance, type IconInput, type LoaderKind } from "@/lib/tauri/instances";
import { installContent } from "@/lib/tauri/modrinth";
import { PlaceholderTile, PresetTile } from "./InstanceIcon";
import { IconPicker } from "./IconPicker";
import { LoaderPicker } from "./LoaderPicker";
import { DEFAULT_PRESET } from "./presets";
import { useVersionManifest, VersionPicker } from "./VersionPicker";

interface NewInstanceDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

const DEFAULT_ICON: IconInput = { kind: "preset", key: DEFAULT_PRESET.key };

/** Catalogue id of the Simple Voice Chat mod, which works on every mod loader. */
const VOICE_CHAT_PROJECT = "9eGKb6K1";

function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function NewInstanceDialog({ open, onOpenChange }: NewInstanceDialogProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const manifest = useVersionManifest();
  const [name, setName] = useState("");
  const [nameEdited, setNameEdited] = useState(false);
  const [gameVersion, setGameVersion] = useState("");
  const [icon, setIcon] = useState<IconInput>(DEFAULT_ICON);
  const [loader, setLoader] = useState<LoaderKind>("vanilla");
  const [loaderVersion, setLoaderVersion] = useState<string | null>(null);
  const [texturesOpen, setTexturesOpen] = useState(false);
  const [voiceChat, setVoiceChat] = useState(false);

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
    setVoiceChat(false);
    create.reset();
  };

  const create = useMutation({
    mutationFn: () => createInstance({ name, gameVersion, icon, loader, loaderVersion }),
    onSuccess: async (instance) => {
      if (voiceChat && loader !== "vanilla") {
        try {
          await installContent(instance.id, VOICE_CHAT_PROJECT, null, () => undefined);
        } catch (error) {
          toast.error(t("instances.voiceChatFailed", { reason: errorMessage(t, error) }));
        }
      }
      await queryClient.invalidateQueries({ queryKey: ["instances"] });
      onOpenChange(false);
      reset();
    },
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogTitle>{t("instances.newTitle")}</DialogTitle>
        <DialogDescription>{t("instances.newSubtitle")}</DialogDescription>

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
          {loader !== "vanilla" && (
            <label className="flex items-start gap-3 rounded-lg border bg-surface/60 p-3 text-sm">
              <input type="checkbox" className="mt-1" checked={voiceChat} onChange={(event) => setVoiceChat(event.target.checked)} />
              <span>
                <span className="block font-medium">{t("instances.voiceChat")}</span>
                <span className="text-muted">{t("instances.voiceChatHint")}</span>
              </span>
            </label>
          )}
        </div>

        {create.isError && <p className="mt-3 text-sm text-danger">{errorMessage(t, create.error)}</p>}

        <div className="mt-5 flex items-center justify-between gap-2">
          <Button variant="ghost" onClick={() => setTexturesOpen(true)}>
            <Brush size={16} />
            {t("textures.createOwn")}
            <span className="rounded-full bg-accent/15 px-2 py-0.5 text-[11px] font-semibold text-accent">{t("textures.beta")}</span>
          </Button>
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => onOpenChange(false)}>
            {t("common.cancel")}
          </Button>
          <Button
            disabled={!name.trim() || !gameVersion || (loader !== "vanilla" && !loaderVersion) || create.isPending}
            onClick={() => create.mutate()}
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
