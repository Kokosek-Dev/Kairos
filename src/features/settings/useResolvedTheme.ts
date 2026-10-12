import { useEffect, useState } from "react";
import { usePreferences } from "./preferences";

const LIGHT_QUERY = "(prefers-color-scheme: light)";

/** The look to show: the player's choice, or the system's when they chose to follow it. */
export function useResolvedTheme(): "dark" | "light" {
  const theme = usePreferences((state) => state.theme);
  const [systemLight, setSystemLight] = useState(() => window.matchMedia(LIGHT_QUERY).matches);

  useEffect(() => {
    const media = window.matchMedia(LIGHT_QUERY);
    const onChange = () => setSystemLight(media.matches);
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);

  if (theme === "system") return systemLight ? "light" : "dark";
  return theme;
}
