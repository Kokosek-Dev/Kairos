import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { readPackTexture } from "@/lib/tauri/texturepacks";
import { cn } from "@/lib/utils";

export const textureKey = (packId: string, path: string) => ["pack-texture", packId, path];

interface TextureThumbProps {
  packId: string;
  path: string;
  selected: boolean;
  edited: boolean;
  onSelect: () => void;
}

function shortName(path: string): string {
  return (path.split("/").pop() ?? path).replace(/\.png$/, "").replace(/_/g, " ");
}

/** Loads its picture only once it scrolls into view, since a game has thousands of textures. */
export function TextureThumb({ packId, path, selected, edited, onSelect }: TextureThumbProps) {
  const element = useRef<HTMLButtonElement>(null);
  const [visible, setVisible] = useState(false);
  const picture = useQuery({
    queryKey: textureKey(packId, path),
    queryFn: () => readPackTexture(packId, path),
    enabled: visible,
    staleTime: Infinity,
  });

  useEffect(() => {
    const target = element.current;
    if (!target) return;
    const observer = new IntersectionObserver(([entry]) => entry.isIntersecting && setVisible(true), { rootMargin: "120px" });
    observer.observe(target);
    return () => observer.disconnect();
  }, []);

  return (
    <button
      ref={element}
      type="button"
      title={path}
      onClick={onSelect}
      className={cn(
        "group flex flex-col gap-1.5 rounded-lg border bg-surface/40 p-1.5 text-left transition-all hover:-translate-y-0.5 hover:border-accent/60 hover:bg-surface-raised/60",
        selected && "border-accent bg-accent/10 ring-2 ring-accent/50",
      )}
    >
      <span className="checker relative grid aspect-square place-items-center overflow-hidden rounded-md">
        {picture.data && (
          <img src={picture.data} alt="" draggable={false} className="max-h-full max-w-full object-contain transition-transform group-hover:scale-110 [image-rendering:pixelated]" />
        )}
        {edited && <span className="absolute right-1 top-1 size-2 rounded-full bg-success shadow-[0_0_6px_var(--success)]" />}
      </span>
      <span className="truncate text-[10px] capitalize text-muted group-hover:text-foreground">{shortName(path)}</span>
    </button>
  );
}
