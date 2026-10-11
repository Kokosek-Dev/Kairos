import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useAccounts } from "@/features/accounts/AccountsSection";
import { usePreferences } from "@/features/settings/preferences";
import { getProfile, type Profile } from "@/lib/tauri/skins";

export const SKINS_KEY = ["saved-skins"];

export function useActiveAccount() {
  const accounts = useAccounts();
  const active = accounts.data?.accounts.find((account) => account.id === accounts.data?.activeId) ?? null;
  return { active, isPending: accounts.isPending, accounts: accounts.data?.accounts ?? [], activeId: accounts.data?.activeId ?? null };
}

export const profileKey = (accountId: string | null) => ["profile", accountId];

export function useProfile() {
  const { active } = useActiveAccount();
  const incognito = usePreferences((state) => state.incognito);
  return useQuery({
    queryKey: profileKey(active?.id ?? null),
    queryFn: getProfile,
    enabled: active !== null && !incognito,
    staleTime: 5 * 60_000,
    retry: false,
  });
}

export function useSetProfile() {
  const queryClient = useQueryClient();
  const { active } = useActiveAccount();
  return (profile: Profile) => queryClient.setQueryData(profileKey(active?.id ?? null), profile);
}
