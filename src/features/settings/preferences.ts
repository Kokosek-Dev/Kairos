import { create } from "zustand";

export type BackgroundStyle = "aurora" | "calm" | "plain" | "image";

interface Preferences {
  autoUpdateMods: boolean;
  mascot: boolean;
  background: BackgroundStyle;
  backgroundImage: string | null;
  backgroundDim: number;
}

const STORAGE_KEY = "kairos.preferences";
const DEFAULTS: Preferences = {
  autoUpdateMods: true,
  mascot: true,
  background: "aurora",
  backgroundImage: null,
  backgroundDim: 0.6,
};

function load(): Preferences {
  try {
    return { ...DEFAULTS, ...JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}") };
  } catch {
    return DEFAULTS;
  }
}

interface PreferencesState extends Preferences {
  update: (changes: Partial<Preferences>) => void;
}

export const usePreferences = create<PreferencesState>((set, get) => ({
  ...load(),
  update: (changes) => {
    set(changes);
    const { autoUpdateMods, mascot, background, backgroundImage, backgroundDim } = get();
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ autoUpdateMods, mascot, background, backgroundImage, backgroundDim }));
    } catch {
      // The choice still applies for this session when storage is full or unavailable.
    }
  },
}));
