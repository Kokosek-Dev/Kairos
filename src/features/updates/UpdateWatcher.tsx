import { useEffect } from "react";
import { findUpdate, offerUpdate } from "./updater";

const FIRST_CHECK_DELAY_MS = 8000;

/** Quietly looks for a newer Zenith shortly after the start and offers it. */
export function UpdateWatcher() {
  useEffect(() => {
    const timer = setTimeout(() => {
      void findUpdate().then((update) => update && offerUpdate(update));
    }, FIRST_CHECK_DELAY_MS);
    return () => clearTimeout(timer);
  }, []);
  return null;
}
