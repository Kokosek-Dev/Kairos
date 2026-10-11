import { useMutation, useQueryClient } from "@tanstack/react-query";
import { Check, ChevronDown, LogIn, LogOut, Plus, Shirt } from "lucide-react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { SkinHead } from "@/features/skins/SkinHead";
import { useActiveAccount, useProfile } from "@/features/skins/useProfile";
import { removeAccount, setActiveAccount } from "@/lib/tauri/auth";
import { errorMessage } from "@/lib/tauri/errors";
import { ACCOUNTS_KEY, useSignInStore } from "./signInStore";

export function AccountMenu() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const queryClient = useQueryClient();
  const startSignIn = useSignInStore((state) => state.start);
  const { active, accounts, isPending } = useActiveAccount();
  const profile = useProfile();

  const refresh = () => queryClient.invalidateQueries({ queryKey: ACCOUNTS_KEY });
  const onError = (error: unknown) => toast.error(errorMessage(t, error));
  const switchTo = useMutation({ mutationFn: setActiveAccount, onSuccess: refresh, onError });
  const signOut = useMutation({
    mutationFn: removeAccount,
    onSuccess: async () => {
      await refresh();
      toast.success(t("accounts.signedOut"));
    },
    onError,
  });

  if (isPending) return <div className="skeleton mr-2 h-7 w-28 rounded-md" />;

  if (!active) {
    return (
      <button
        type="button"
        onClick={() => void startSignIn()}
        className="mr-2 flex h-7 items-center gap-2 rounded-md bg-accent px-3 text-xs font-semibold text-accent-foreground transition-all hover:bg-accent-strong hover:shadow-glow active:scale-95"
      >
        <LogIn size={14} />
        {t("accounts.signIn")}
      </button>
    );
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          className="mr-2 flex h-8 items-center gap-2 rounded-md px-2 text-xs font-medium transition-colors hover:bg-surface-raised"
        >
          <SkinHead url={profile.data?.skinUrl} name={active.username} size={22} className="size-[22px]" />
          <span className="max-w-32 truncate">{active.username}</span>
          <ChevronDown size={13} className="text-muted" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-60">
        <div className="flex items-center gap-3 px-3 py-3">
          <SkinHead url={profile.data?.skinUrl} name={active.username} size={40} className="size-10" />
          <div className="min-w-0">
            <p className="truncate font-semibold">{active.username}</p>
            <p className="text-xs text-muted">{t("accounts.microsoftAccount")}</p>
          </div>
        </div>
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={() => navigate("/")}>
          <Shirt size={15} />
          {t("accounts.mySkins")}
        </DropdownMenuItem>
        {accounts.length > 1 && <DropdownMenuSeparator />}
        {accounts.length > 1 &&
          accounts.map((account) => (
            <DropdownMenuItem key={account.id} onSelect={() => account.id !== active.id && switchTo.mutate(account.id)}>
              {account.id === active.id ? <Check size={15} className="text-accent" /> : <span className="w-[15px]" />}
              <span className="truncate">{account.username}</span>
            </DropdownMenuItem>
          ))}
        <DropdownMenuSeparator />
        <DropdownMenuItem onSelect={() => void startSignIn()}>
          <Plus size={15} />
          {t("accounts.add")}
        </DropdownMenuItem>
        <DropdownMenuItem danger onSelect={() => signOut.mutate(active.id)}>
          <LogOut size={15} />
          {t("accounts.signOut")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
