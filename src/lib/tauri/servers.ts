import { invoke } from "@tauri-apps/api/core";

export type Edition = "java" | "bedrock";

export interface ServerEntry {
  id: string;
  name: string;
  address: string;
  edition: Edition;
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
export const addServer = (name: string, address: string, edition: Edition) => invoke<ServerEntry>("add_server", { name, address, edition });
export const removeServer = (id: string) => invoke<void>("remove_server", { id });
export const pingServer = (address: string, edition: Edition) => invoke<ServerStatus>("ping_server", { address, edition });
export const bedrockInstalled = () => invoke<boolean>("bedrock_installed");
export const openBedrock = () => invoke<void>("open_bedrock");
