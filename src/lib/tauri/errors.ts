import type { TFunction } from "i18next";

export interface LauncherError {
  code: string;
  message: string;
}

export function isLauncherError(value: unknown): value is LauncherError {
  return typeof value === "object" && value !== null && "code" in value && "message" in value;
}

export function errorMessage(t: TFunction, error: unknown): string {
  if (!isLauncherError(error)) return String(error);
  return t(`errors.${error.code}`, { defaultValue: error.message });
}
