import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Check, Plus, Trash2, UserRound } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { getAccounts, removeAccount, setActiveAccount } from "@/lib/tauri/auth";
import { errorMessage } from "@/lib/tauri/errors";
import { AccountAvatar } from "./AccountAvatar";
import { ACCOUNTS_KEY, useSignInStore } from "./signInStore";

export function useAccounts() {
  return useQuery({ queryKey: ACCOUNTS_KEY, queryFn: getAccounts });
}

export function AccountsSection() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const accounts = useAccounts();
  const startSignIn = useSignInStore((state) => state.start);

  const refresh = () => queryClient.invalidateQueries({ queryKey: ACCOUNTS_KEY });
  const activate = useMutation({ mutationFn: setActiveAccount, onSuccess: refresh });
  const remove = useMutation({ mutationFn: removeAccount, onSuccess: refresh });

  return (
    <section className="panel p-5">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="flex items-center gap-2 font-semibold">
            <UserRound size={16} className="text-accent" />
            {t("accounts.title")}
          </h2>
          <p className="mt-1 text-muted">{t("accounts.subtitle")}</p>
        </div>
        <Button size="sm" onClick={() => void startSignIn()}>
          <Plus size={14} />
          {t("accounts.add")}
        </Button>
      </div>

      <ul className="mt-4 space-y-2">
        {accounts.isPending && <li className="skeleton h-14 rounded-lg" />}
        {accounts.data?.accounts.length === 0 && <li className="py-4 text-center text-muted">{t("accounts.empty")}</li>}
        {accounts.data?.accounts.map((account) => {
          const active = account.id === accounts.data.activeId;
          return (
            <li key={account.id} className="flex items-center gap-3 rounded-lg border bg-background/40 px-3 py-2.5">
              <AccountAvatar name={account.username} />
              <span className="min-w-0 flex-1 truncate font-medium">{account.username}</span>
              {active ? (
                <span className="flex items-center gap-1.5 text-xs font-medium text-success">
                  <Check size={14} />
                  {t("accounts.active")}
                </span>
              ) : (
                <Button variant="secondary" size="sm" onClick={() => activate.mutate(account.id)}>
                  {t("accounts.use")}
                </Button>
              )}
              <Button
                variant="ghost"
                size="sm"
                className="size-8 px-0 hover:text-danger"
                aria-label={t("accounts.remove")}
                title={t("accounts.remove")}
                onClick={() => remove.mutate(account.id)}
              >
                <Trash2 size={15} />
              </Button>
            </li>
          );
        })}
      </ul>
      {(activate.isError || remove.isError) && (
        <p className="mt-2 text-sm text-danger">{errorMessage(t, activate.error ?? remove.error)}</p>
      )}
    </section>
  );
}
