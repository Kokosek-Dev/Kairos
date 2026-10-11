import { invoke } from "@tauri-apps/api/core";

export interface TexturePack {
  id: string;
  name: string;
  gameVersion: string;
  packFormat: number;
  rangedFormat: boolean;
  createdAt: string;
}

export interface TextureList {
  textures: string[];
  edited: string[];
}

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
export const installTexturePack = (packId: string, instanceId: string) =>
  invoke<string>("install_texture_pack", { packId, instanceId });
