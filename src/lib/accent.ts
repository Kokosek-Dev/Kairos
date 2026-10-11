export interface AccentColor {
  key: "blue" | "violet" | "pink" | "orange" | "green" | "teal";
  hue: number;
}

export const ACCENT_COLORS: AccentColor[] = [
  { key: "blue", hue: 252 },
  { key: "violet", hue: 295 },
  { key: "pink", hue: 345 },
  { key: "orange", hue: 55 },
  { key: "green", hue: 150 },
  { key: "teal", hue: 195 },
];

const STORAGE_KEY = "zenith.accent";

function stored(): number | null {
  try {
    const value = Number(localStorage.getItem(STORAGE_KEY));
    return ACCENT_COLORS.some((color) => color.hue === value) ? value : null;
  } catch {
    return null;
  }
}

export function currentAccent(): number {
  return stored() ?? ACCENT_COLORS[0].hue;
}

export function applyAccent(hue: number): void {
  document.documentElement.style.setProperty("--accent-hue", String(hue));
  try {
    localStorage.setItem(STORAGE_KEY, String(hue));
  } catch {
    // The color still applies for this session when storage is unavailable.
  }
}

export function restoreAccent(): void {
  document.documentElement.style.setProperty("--accent-hue", String(currentAccent()));
}
