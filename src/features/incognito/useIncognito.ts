import { useActiveAccount } from "@/features/skins/useProfile";
import { usePreferences } from "@/features/settings/preferences";

/** Incognito needs an account that owns the game, since it plays offline under that account. Development builds may skip that. */
export function useCanUseIncognito(): boolean {
  const { active } = useActiveAccount();
  return import.meta.env.DEV || active !== null;
}

export function useIncognito(): boolean {
  const wanted = usePreferences((state) => state.incognito);
  const allowed = useCanUseIncognito();
  return wanted && allowed;
}
