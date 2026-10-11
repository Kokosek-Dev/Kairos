import { useQuery } from "@tanstack/react-query";
import { getTexturePacksEverywhere } from "@/lib/tauri/texturepacks";

export const EVERYWHERE_KEY = ["texture-packs-everywhere"];

/** Whether texture packs are used in every instance of their game version automatically. */
export function useEverywhere(): boolean {
  const everywhere = useQuery({ queryKey: EVERYWHERE_KEY, queryFn: getTexturePacksEverywhere });
  return everywhere.data ?? true;
}
