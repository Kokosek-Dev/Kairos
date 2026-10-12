import {
  BookOpen,
  Brush,
  Compass,
  Home,
  Library,
  Play,
  Search,
  Settings,
  Shirt,
  SlidersHorizontal,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { InstanceIcon } from "@/features/instances/InstanceIcon";
import { useInstances } from "@/features/instances/useInstances";
import { useGameStore } from "@/features/launch/gameStore";
import { cn } from "@/lib/utils";
import { usePaletteStore } from "./paletteStore";

interface Entry {
  id: string;
  label: string;
  hint: string;
  icon?: LucideIcon;
  leading?: React.ReactNode;
  run: () => void;
}

/** Ctrl+K: jump to any page, open an instance or start playing without touching the mouse. */
export function CommandPalette() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const instances = useInstances();
  const { open, setOpen } = usePaletteStore();
  const [query, setQuery] = useState("");
  const [index, setIndex] = useState(0);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setOpen(!usePaletteStore.getState().open);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [setOpen]);

  const entries = useMemo<Entry[]>(() => {
    const go = (path: string) => () => navigate(path);
    const pages: Entry[] = [
      { id: "home", label: t("nav.home"), hint: t("palette.page"), icon: Home, run: go("/") },
      { id: "library", label: t("nav.library"), hint: t("palette.page"), icon: Library, run: go("/library") },
      { id: "discover", label: t("nav.discover"), hint: t("palette.page"), icon: Compass, run: go("/discover") },
      { id: "journal", label: t("nav.journal"), hint: t("palette.page"), icon: BookOpen, run: go("/journal") },
      { id: "textures", label: t("nav.textures"), hint: t("palette.page"), icon: Shirt, run: go("/textures") },
      { id: "settings", label: t("nav.settings"), hint: t("palette.page"), icon: Settings, run: go("/settings") },
      { id: "editor", label: t("editor.title"), hint: t("palette.tool"), icon: Shirt, run: go("/skins/editor") },
      { id: "journal", label: t("nav.journal"), hint: t("palette.page"), icon: BookOpen, run: go("/journal") },
      { id: "textures", label: t("textures.packs"), hint: t("palette.tool"), icon: Brush, run: go("/library") },
      { id: "storage", label: t("storage.title"), hint: t("palette.page"), icon: SlidersHorizontal, run: go("/settings") },
    ];
    const games = (instances.data ?? []).flatMap<Entry>((instance) => [
      {
        id: `play-${instance.id}`,
        label: t("palette.play", { name: instance.name }),
        hint: instance.gameVersion,
        icon: Play,
        run: () => void useGameStore.getState().play(instance.id, instance.name),
      },
      {
        id: `open-${instance.id}`,
        label: t("palette.open", { name: instance.name }),
        hint: t("palette.instance"),
        leading: <InstanceIcon icon={instance.icon} instanceId={instance.id} className="size-5 rounded" iconSize={11} />,
        run: go(`/instances/${instance.id}`),
      },
    ]);
    return [...pages, ...games];
  }, [t, navigate, instances.data]);

  const matches = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return needle === "" ? entries : entries.filter((entry) => `${entry.label} ${entry.hint}`.toLowerCase().includes(needle));
  }, [entries, query]);

  const choose = (entry: Entry | undefined) => {
    if (!entry) return;
    setOpen(false);
    entry.run();
  };

  const onKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const step = event.key === "ArrowDown" ? 1 : -1;
      setIndex((current) => (matches.length === 0 ? 0 : (current + step + matches.length) % matches.length));
    } else if (event.key === "Enter") {
      choose(matches[index]);
    }
  };

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) {
          setQuery("");
          setIndex(0);
        }
      }}
    >
      <DialogContent className="top-[20%] w-[min(34rem,calc(100vw-2rem))] translate-y-0 gap-0 p-0">
        <DialogTitle className="sr-only">{t("palette.title")}</DialogTitle>
        <label className="flex items-center gap-3 border-b px-4 py-3">
          <Search size={18} className="text-muted" />
          <input
            autoFocus
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setIndex(0);
            }}
            onKeyDown={onKeyDown}
            placeholder={t("palette.placeholder")}
            className="min-w-0 flex-1 bg-transparent text-base outline-none placeholder:text-muted"
          />
        </label>
        <ul className="max-h-80 overflow-y-auto p-2">
          {matches.length === 0 && <li className="px-3 py-6 text-center text-muted">{t("palette.empty")}</li>}
          {matches.map((entry, position) => {
            const Icon = entry.icon;
            return (
              <li key={entry.id}>
                <button
                  type="button"
                  onMouseMove={() => setIndex(position)}
                  onClick={() => choose(entry)}
                  className={cn(
                    "flex w-full items-center gap-3 rounded-lg px-3 py-2.5 text-left transition-colors",
                    position === index ? "bg-accent/15 text-foreground" : "text-muted",
                  )}
                >
                  {Icon ? <Icon size={17} className="text-accent" /> : entry.leading}
                  <span className="min-w-0 flex-1 truncate">{entry.label}</span>
                  <span className="text-xs text-muted">{entry.hint}</span>
                </button>
              </li>
            );
          })}
        </ul>
      </DialogContent>
    </Dialog>
  );
}
