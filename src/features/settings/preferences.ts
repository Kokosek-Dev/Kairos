import { create } from "zustand";

export type ThemeChoice = "dark" | "light" | "system";
export type BackgroundStyle = "aurora" | "calm" | "plain" | "image";

interface Preferences {
  theme: ThemeChoice;
  autoUpdateMods: boolean;
  mascot: boolean;
  incognito: boolean;
  discordPresence: boolean;
  newsOpen: boolean;
  background: BackgroundStyle;
  backgroundImage: string | null;
  backgroundDim: number;
}

const STORAGE_KEY = "kairos.preferences";
const DEFAULTS: Preferences = {
  theme: "dark",
  autoUpdateMods: true,
  mascot: true,
  incognito: false,
  discordPresence: true,
  newsOpen: window.innerWidth >= 1280,
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
    const { theme, autoUpdateMods, mascot, incognito, discordPresence, newsOpen, background, backgroundImage, backgroundDim } = get();
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify({ theme, autoUpdateMods, mascot, incognito, discordPresence, newsOpen, background, backgroundImage, backgroundDim }));
    } catch {
      // The choice still applies for this session when storage is full or unavailable.
    }
  },
}));
