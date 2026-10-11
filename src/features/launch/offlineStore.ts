import { create } from "zustand";

interface OfflineRequest {
  instanceId: string;
  title: string;
}

interface OfflineState {
  request: OfflineRequest | null;
  ask: (request: OfflineRequest) => void;
  dismiss: () => void;
}

/** Offline play always goes through a confirmation, so it is requested here first. */
export const useOfflineStore = create<OfflineState>((set) => ({
  request: null,
  ask: (request) => set({ request }),
  dismiss: () => set({ request: null }),
}));
