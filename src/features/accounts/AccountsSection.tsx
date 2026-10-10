import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Check, ExternalLink, Plus, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Field, Input } from "@/components/ui/input";
import { getAccounts, getAuthConfig, openLoginPage, removeAccount, setActiveAccount, setMsClientId } from "@/lib/tauri/auth";
import { errorMessage } from "@/lib/tauri/errors";
import { AccountAvatar } from "./AccountAvatar";
import { ACCOUNTS_KEY, useSignInStore } from "./signInStore";

const APP_REGISTRATIONS_URL = "https://entra.microsoft.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade";
const CONFIG_KEY = ["auth-config"];

export function useAccounts() {
  return useQuery({ queryKey: ACCOUNTS_KEY, queryFn: getAccounts });
}

function ApplicationId() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const config = useQuery({ queryKey: CONFIG_KEY, queryFn: getAuthConfig });
  const [value, setValue] = useState("");

  useEffect(() => {
    if (config.data) setValue(config.data.clientId ?? "");
  }, [config.data]);

  const save = useMutation({
    mutationFn: () => setMsClientId(value.trim() || null),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: CONFIG_KEY }),
  });

  return (
    <div className="mt-5 border-t pt-5">
      <Field label={t("accounts.appId")} hint={t("accounts.appIdHint")}>
        <div className="flex gap-2">
          <Input
            value={value}
            onChange={(event) => setValue(event.target.value)}
            placeholder="00000000-0000-0000-0000-000000000000"
            spellCheck={false}
          />
          <Button variant="secondary" disabled={save.isPending} onClick={() => save.mutate()}>
            {t("instances.save")}
          </Button>
        </div>
      </Field>
      <div className="mt-2 flex flex-wrap items-center gap-3 text-xs">
        <span className={config.data?.configured ? "text-success" : "text-muted"}>
          {config.data?.configured ? t("accounts.configured") : t("accounts.notConfigured")}
        </span>
        <button
          type="button"
          className="inline-flex items-center gap-1 text-accent hover:underline"
          onClick={() => void openLoginPage(APP_REGISTRATIONS_URL)}
        >
          <ExternalLink size={12} />
          {t("accounts.registerApp")}
        </button>
      </div>
      {save.isError && <p className="mt-2 text-sm text-danger">{errorMessage(t, save.error)}</p>}
    </div>
  );
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
    <section className="rounded-xl border bg-surface p-5">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h2 className="font-semibold">{t("accounts.title")}</h2>
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

      <ApplicationId />
    </section>
  );
}
