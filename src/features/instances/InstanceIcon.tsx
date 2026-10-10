import { useQuery } from "@tanstack/react-query";
import { ImageIcon } from "lucide-react";
import { getInstanceIcon, type InstanceIcon as IconData } from "@/lib/tauri/instances";
import { cn } from "@/lib/utils";
import { findPreset } from "./presets";

interface InstanceIconProps {
  icon: IconData;
  instanceId: string;
  className?: string;
  iconSize?: number;
}

export function PresetTile({ presetKey, className, iconSize = 28 }: { presetKey: string; className?: string; iconSize?: number }) {
  const { icon: Icon, hue } = findPreset(presetKey);
  return (
    <div
      className={cn("grid shrink-0 place-items-center rounded-lg text-white", className)}
      style={{ background: `linear-gradient(135deg, oklch(0.66 0.17 ${hue}), oklch(0.42 0.16 ${hue + 30}))` }}
    >
      <Icon size={iconSize} />
    </div>
  );
}

export function PlaceholderTile({ className, iconSize = 28 }: { className?: string; iconSize?: number }) {
  return (
    <div className={cn("grid shrink-0 place-items-center rounded-lg bg-surface-raised text-muted", className)}>
      <ImageIcon size={iconSize} />
    </div>
  );
}

export function InstanceIcon({ icon, instanceId, className, iconSize }: InstanceIconProps) {
  const custom = icon.kind === "custom";
  const { data } = useQuery({
    queryKey: ["instance-icon", instanceId, custom ? icon.file : null],
    queryFn: () => getInstanceIcon(instanceId),
    enabled: custom,
    staleTime: Infinity,
  });

  if (icon.kind === "preset") return <PresetTile presetKey={icon.key} className={className} iconSize={iconSize} />;
  if (!data) return <PlaceholderTile className={cn("skeleton", className)} iconSize={iconSize} />;
  return <img src={data} alt="" className={cn("shrink-0 rounded-lg object-cover", className)} draggable={false} />;
}
