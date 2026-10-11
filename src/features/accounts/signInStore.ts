import { toast } from "sonner";
import { create } from "zustand";
import i18n from "@/i18n";
import { isLauncherError } from "@/lib/tauri/errors";
import { queryClient } from "@/lib/queryClient";
import { cancelSignIn, signIn, type SignInEvent, type SignInStep } from "@/lib/tauri/auth";

export const ACCOUNTS_KEY = ["accounts"];
const CLOSE_DELAY_MS = 1200;

export type SignInPhase =
  | { kind: "idle" }
  | { kind: "starting" }
  | { kind: "browser"; url: string }
  | { kind: "working"; step: SignInStep }
  | { kind: "done"; username: string }
  | { kind: "failed"; error: unknown };

interface SignInState {
  open: boolean;
  phase: SignInPhase;
  start: () => Promise<void>;
  close: () => void;
}

export const useSignInStore = create<SignInState>((set, get) => {
  const handle = (event: SignInEvent) => {
    if (event.type === "browser") {
      set({ phase: { kind: "browser", url: event.url } });
    } else {
      set({ phase: { kind: "working", step: event.step } });
    }
  };

  return {
    open: false,
    phase: { kind: "idle" },

    start: async () => {
      set({ open: true, phase: { kind: "starting" } });
      try {
        const account = await signIn(handle);
        set({ phase: { kind: "done", username: account.username } });
        await queryClient.invalidateQueries({ queryKey: ACCOUNTS_KEY });
        toast.success(i18n.t("signIn.done", { name: account.username }));
        setTimeout(() => get().phase.kind === "done" && get().close(), CLOSE_DELAY_MS);
      } catch (error) {
        if (isLauncherError(error) && error.code === "cancelled") {
          set({ open: false, phase: { kind: "idle" } });
        } else {
          set({ phase: { kind: "failed", error } });
        }
      }
    },

    close: () => {
      const { phase } = get();
      if (phase.kind === "starting" || phase.kind === "browser" || phase.kind === "working") {
        void cancelSignIn();
      }
      set({ open: false, phase: { kind: "idle" } });
    },
  };
});
