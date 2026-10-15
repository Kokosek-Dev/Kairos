import { invoke } from "@tauri-apps/api/core";

export interface TexturePack {
  id: string;
  name: string;
  gameVersion: string;
  packFormat: number;
  rangedFormat: boolean;
  instances: string[];
  createdAt: string;
}

export interface TextureList {
  textures: string[];
  edited: string[];
}

export interface ModelFace {
  direction: string;
  texture: string;
  uv: [number, number, number, number] | null;
  rotation: number;
  tinted: boolean;
}

export interface ModelElement {
  from: [number, number, number];
  to: [number, number, number];
  rotation: { origin: [number, number, number]; axis: string; angle: number } | null;
  faces: ModelFace[];
}

export interface BlockModel {
  elements: ModelElement[];
}

export const getTexturePacksEverywhere = () => invoke<boolean>("get_texture_packs_everywhere");
export const setTexturePacksEverywhere = (enabled: boolean) => invoke<void>("set_texture_packs_everywhere", { enabled });
export const listTextureVersions = () => invoke<string[]>("list_texture_versions");
export const createTexturePack = (name: string, gameVersion: string) =>
  invoke<TexturePack>("create_texture_pack", { name, gameVersion });
export const listTexturePacks = () => invoke<TexturePack[]>("list_texture_packs");
export const deleteTexturePack = (packId: string) => invoke<void>("delete_texture_pack", { packId });
export const listPackTextures = (packId: string) => invoke<TextureList>("list_pack_textures", { packId });
export const readPackTexture = (packId: string, path: string) => invoke<string>("read_pack_texture", { packId, path });
export const writePackTexture = (packId: string, path: string, pngBase64: string) =>
  invoke<void>("write_pack_texture", { packId, path, pngBase64 });
export const resetPackTexture = (packId: string, path: string) => invoke<void>("reset_pack_texture", { packId, path });
export const exportTexturePack = (packId: string) => invoke<string | null>("export_texture_pack", { packId });
export const setTexturePackInstances = (packId: string, instanceIds: string[]) =>
  invoke<TexturePack>("use_texture_pack_in_instances", { packId, instanceIds });
export const getTextureModel = (packId: string, path: string) => invoke<BlockModel | null>("get_texture_model", { packId, path });
