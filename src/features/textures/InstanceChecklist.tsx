import { useTranslation } from "react-i18next";
import type { Instance } from "@/lib/tauri/instances";

interface InstanceChecklistProps {
  instances: Instance[];
  gameVersion: string;
  selected: string[];
  onChange: (selected: string[]) => void;
}

/** The instances of one game version, each with a checkbox for using the texture pack in it. */
export function InstanceChecklist({ instances, gameVersion, selected, onChange }: InstanceChecklistProps) {
  const { t } = useTranslation();
  const matching = instances.filter((instance) => instance.gameVersion === gameVersion);
  if (matching.length === 0) return <p className="text-sm text-muted">{t("textures.noMatching", { version: gameVersion })}</p>;

  const toggle = (id: string) => onChange(selected.includes(id) ? selected.filter((item) => item !== id) : [...selected, id]);
  return (
    <ul className="space-y-1.5">
      {matching.map((instance) => (
        <li key={instance.id}>
          <label className="flex cursor-pointer items-center gap-3 rounded-md border bg-surface/60 px-3 py-2 text-sm hover:border-accent/50">
            <input type="checkbox" checked={selected.includes(instance.id)} onChange={() => toggle(instance.id)} />
            <span className="truncate">{instance.name}</span>
          </label>
        </li>
      ))}
    </ul>
  );
}
