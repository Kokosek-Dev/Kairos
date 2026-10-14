import { create } from "zustand";
import type { FeedbackKind } from "@/lib/tauri/feedback";

interface FeedbackState {
  open: boolean;
  kind: FeedbackKind;
  /** The instance the message is about, so its mods are the ones listed in the details. */
  instanceId: string | null;
  show: (kind?: FeedbackKind, instanceId?: string | null) => void;
  close: () => void;
}

export const useFeedbackStore = create<FeedbackState>((set) => ({
  open: false,
  kind: "bug",
  instanceId: null,
  show: (kind = "bug", instanceId = null) => set({ open: true, kind, instanceId }),
  close: () => set({ open: false }),
}));
