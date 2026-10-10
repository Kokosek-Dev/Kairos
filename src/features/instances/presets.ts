import { Box, Flame, Gem, Ghost, Leaf, Pickaxe, Skull, Snowflake, Sword, Zap, type LucideIcon } from "lucide-react";

export interface IconPreset {
  key: string;
  icon: LucideIcon;
  hue: number;
}

export const ICON_PRESETS: IconPreset[] = [
  { key: "cube", icon: Box, hue: 252 },
  { key: "sword", icon: Sword, hue: 15 },
  { key: "pickaxe", icon: Pickaxe, hue: 70 },
  { key: "leaf", icon: Leaf, hue: 150 },
  { key: "flame", icon: Flame, hue: 40 },
  { key: "snowflake", icon: Snowflake, hue: 215 },
  { key: "zap", icon: Zap, hue: 95 },
  { key: "skull", icon: Skull, hue: 300 },
  { key: "ghost", icon: Ghost, hue: 330 },
  { key: "gem", icon: Gem, hue: 190 },
];

export const DEFAULT_PRESET = ICON_PRESETS[0];

export function findPreset(key: string): IconPreset {
  return ICON_PRESETS.find((preset) => preset.key === key) ?? DEFAULT_PRESET;
}
