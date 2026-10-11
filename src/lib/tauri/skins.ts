import { invoke } from "@tauri-apps/api/core";

export type SkinVariant = "classic" | "slim";

export interface Cape {
  id: string;
  alias: string;
  url: string;
  active: boolean;
}

export interface Profile {
  id: string;
  name: string;
  skinUrl: string | null;
  variant: SkinVariant;
  capes: Cape[];
}

export interface SavedSkin {
  id: string;
  name: string;
  variant: SkinVariant;
  addedAt: string;
  dataUrl: string;
}

export const getProfile = () => invoke<Profile>("get_profile");
export const listSkins = () => invoke<SavedSkin[]>("list_skins");
export const addSkinFromFile = (path: string, name: string | null, variant: SkinVariant) =>
  invoke<SavedSkin>("add_skin_from_file", { path, name, variant });
export const saveSkinPng = (name: string, variant: SkinVariant, pngBase64: string) =>
  invoke<SavedSkin>("save_skin_png", { name, variant, pngBase64 });
export const addSkinFromPlayer = (name: string) => invoke<SavedSkin>("add_skin_from_player", { name });
export const saveCurrentSkin = (name: string) => invoke<SavedSkin>("save_current_skin", { name });
export const removeSkin = (skinId: string) => invoke<void>("remove_skin", { skinId });
export const applySkin = (skinId: string) => invoke<Profile>("apply_skin", { skinId });
export const resetSkin = () => invoke<Profile>("reset_skin");
export const setCape = (capeId: string | null) => invoke<Profile>("set_cape", { capeId });
export const pickSkinFile = () => invoke<string | null>("pick_skin_file");
