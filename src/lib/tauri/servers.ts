import { invoke } from "@tauri-apps/api/core";

export interface ServerEntry {
  id: string;
  name: string;
  address: string;
}

export interface ServerStatus {
  motd: string;
  version: string;
  playersOnline: number;
  playersMax: number;
  latencyMs: number | null;
  favicon: string | null;
}

export const listServers = () => invoke<ServerEntry[]>("list_servers");
export const addServer = (name: string, address: string) => invoke<ServerEntry>("add_server", { name, address });
export const removeServer = (id: string) => invoke<void>("remove_server", { id });
export const pingServer = (address: string) => invoke<ServerStatus>("ping_server", { address });
