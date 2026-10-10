import { Channel, invoke } from "@tauri-apps/api/core";

export interface Account {
  id: string;
  username: string;
  addedAt: string;
}

export interface Accounts {
  activeId: string | null;
  accounts: Account[];
}

export interface AuthConfig {
  clientId: string | null;
  configured: boolean;
}

export type SignInStep = "xbox" | "xsts" | "minecraft" | "profile";

export type SignInEvent =
  | { type: "device_code"; userCode: string; verificationUri: string; expiresIn: number }
  | { type: "step"; step: SignInStep };

export const SIGN_IN_STEPS: SignInStep[] = ["xbox", "xsts", "minecraft", "profile"];

export const getAccounts = () => invoke<Accounts>("get_accounts");

export function signIn(onEvent: (event: SignInEvent) => void): Promise<Account> {
  const channel = new Channel<SignInEvent>();
  channel.onmessage = onEvent;
  return invoke<Account>("sign_in", { onEvent: channel });
}

export const cancelSignIn = () => invoke<void>("cancel_sign_in");
export const setActiveAccount = (accountId: string) => invoke<void>("set_active_account", { accountId });
export const removeAccount = (accountId: string) => invoke<void>("remove_account", { accountId });
export const getAuthConfig = () => invoke<AuthConfig>("get_auth_config");
export const setMsClientId = (clientId: string | null) => invoke<void>("set_ms_client_id", { clientId });
export const openLoginPage = (url: string) => invoke<void>("open_login_page", { url });
