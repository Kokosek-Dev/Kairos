import type { LoaderKind } from "@/lib/tauri/instances";

/** Each mod loader gets its own color so instances are recognisable at a glance. */
export const LOADER_HUE: Record<LoaderKind, number | null> = {
  vanilla: null,
  fabric: 75,
  quilt: 310,
  forge: 28,
  neoforge: 50,
};

export function loaderChipStyle(loader: LoaderKind): React.CSSProperties | undefined {
  const hue = LOADER_HUE[loader];
  if (hue === null) return undefined;
  return {
    color: `oklch(var(--chip-text) 0.13 ${hue})`,
    background: `oklch(var(--chip-fill) 0.12 ${hue} / 0.22)`,
    boxShadow: `inset 0 0 0 1px oklch(0.7 0.14 ${hue} / 0.3)`,
  };
}
