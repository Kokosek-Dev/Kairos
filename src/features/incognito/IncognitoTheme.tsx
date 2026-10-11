import { useEffect } from "react";
import { useIncognito } from "./useIncognito";

/** Switches the whole launcher to the dark incognito look while the mode is on. */
export function IncognitoTheme() {
  const incognito = useIncognito();

  useEffect(() => {
    const root = document.documentElement;
    if (incognito) root.dataset.theme = "incognito";
    else delete root.dataset.theme;
    return () => {
      delete root.dataset.theme;
    };
  }, [incognito]);

  return null;
}
