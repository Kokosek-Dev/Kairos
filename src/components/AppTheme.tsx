import { useEffect } from "react";
import { useIncognito } from "@/features/incognito/useIncognito";
import { useResolvedTheme } from "@/features/settings/useResolvedTheme";

/** Puts the chosen look on the page: the black incognito one wins over light, light over the default dark. */
export function AppTheme() {
  const incognito = useIncognito();
  const resolved = useResolvedTheme();
  const theme = incognito ? "incognito" : resolved === "light" ? "light" : null;

  useEffect(() => {
    const root = document.documentElement;
    if (theme) root.dataset.theme = theme;
    else delete root.dataset.theme;
    return () => {
      delete root.dataset.theme;
    };
  }, [theme]);

  return null;
}
