import { useQuery } from "@tanstack/react-query";
import { getInstance, listInstances } from "@/lib/tauri/instances";

export const INSTANCES_KEY = ["instances"];

export const useInstances = () => useQuery({ queryKey: INSTANCES_KEY, queryFn: listInstances });

export const useInstance = (instanceId: string) =>
  useQuery({ queryKey: ["instance", instanceId], queryFn: () => getInstance(instanceId) });
