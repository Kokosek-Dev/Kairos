import { useQuery } from "@tanstack/react-query";
import { listTexturePacks } from "@/lib/tauri/texturepacks";

export const PACKS_KEY = ["texture-packs"];

export function usePacks() {
  return useQuery({ queryKey: PACKS_KEY, queryFn: listTexturePacks });
}
