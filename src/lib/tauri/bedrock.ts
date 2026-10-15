import { invoke } from "@tauri-apps/api/core";

export type PackKind = "resources" | "behavior";

export interface BedrockPack {
  folder: string;
  name: string;
  kind: PackKind;
  version: string;
}

export interface BedrockInstance {
  id: string;
  name: string;
  createdAt: string;
  packs: BedrockPack[];
}

export const listBedrockInstances = () => invoke<BedrockInstance[]>("list_bedrock_instances");
export const createBedrockInstance = (name: string) => invoke<BedrockInstance>("create_bedrock_instance", { name });
export const deleteBedrockInstance = (id: string) => invoke<void>("delete_bedrock_instance", { id });
export const addBedrockPack = (id: string) => invoke<BedrockInstance | null>("add_bedrock_pack", { id });
export const removeBedrockPack = (id: string, folder: string) => invoke<BedrockInstance>("remove_bedrock_pack", { id, folder });
export const playBedrockInstance = (id: string) => invoke<void>("play_bedrock_instance", { id });
