import { invoke } from "@tauri-apps/api/core";

export interface DataLocation {
  path: string;
  defaultPath: string;
  isDefault: boolean;
  sizeBytes: number;
}

export const getDataLocation = () => invoke<DataLocation>("get_data_location");
export const pickDataFolder = () => invoke<string | null>("pick_data_folder");
export const moveDataLocation = (folder: string) => invoke<void>("move_data_location", { folder });
export const restartApp = () => invoke<void>("restart_app");
