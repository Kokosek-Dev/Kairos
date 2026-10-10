import { Upload } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import type { IconInput } from "@/lib/tauri/instances";
import { pickIconFile } from "@/lib/tauri/instances";
import { cn } from "@/lib/utils";
import { PresetTile } from "./InstanceIcon";
import { ICON_PRESETS } from "./presets";

interface IconPickerProps {
  value: IconInput | null;
  onChange: (icon: IconInput) => void;
}

export function IconPicker({ value, onChange }: IconPickerProps) {
  const { t } = useTranslation();

  const chooseFile = async () => {
    const path = await pickIconFile();
    if (path) onChange({ kind: "custom", path });
  };

  return (
    <div className="flex flex-wrap items-center gap-2">
      {ICON_PRESETS.map((preset) => (
        <button
          key={preset.key}
          type="button"
          onClick={() => onChange({ kind: "preset", key: preset.key })}
          aria-label={preset.key}
          className={cn(
            "rounded-lg p-0.5 transition-all hover:scale-105",
            value?.kind === "preset" && value.key === preset.key && "ring-2 ring-accent",
          )}
        >
          <PresetTile presetKey={preset.key} className="size-10" iconSize={20} />
        </button>
      ))}
      <Button type="button" variant="secondary" size="sm" onClick={() => void chooseFile()}>
        <Upload size={14} />
        {t("instances.customIcon")}
      </Button>
    </div>
  );
}
