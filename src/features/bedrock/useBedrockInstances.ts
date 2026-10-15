import { useQuery } from "@tanstack/react-query";
import { listBedrockInstances } from "@/lib/tauri/bedrock";

export const BEDROCK_KEY = ["bedrock-instances"];

export function useBedrockInstances() {
  return useQuery({ queryKey: BEDROCK_KEY, queryFn: listBedrockInstances });
}
