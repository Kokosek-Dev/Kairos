import { useEffect } from "react";
import { usePreferences } from "@/features/settings/preferences";
import { findUpdate, offerUpdate } from "./updater";

const FIRST_CHECK_DELAY_MS = 8000;

/** Quietly looks for a newer Kairos shortly after the start and offers it. */
export function UpdateWatcher() {
  useEffect(() => {
    const timer = setTimeout(() => {
      if (usePreferences.getState().incognito) return;
      void findUpdate().then((update) => update && offerUpdate(update));
    }, FIRST_CHECK_DELAY_MS);
    return () => clearTimeout(timer);
  }, []);
  return null;
}
