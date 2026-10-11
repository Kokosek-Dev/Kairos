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

  const label = path.replace(/\.png$/, "").split("/").slice(1).join("/") || path;
  return (
    <button
      ref={element}
      type="button"
      title={path}
      onClick={onSelect}
      className={cn(
        "checker relative grid aspect-square place-items-center rounded-lg border p-1.5 transition-all hover:border-accent/60",
        selected && "border-accent ring-2 ring-accent/50",
      )}
    >
      {picture.data && <img src={picture.data} alt={label} draggable={false} className="max-h-full max-w-full object-contain [image-rendering:pixelated]" />}
      {edited && <span className="absolute right-1 top-1 size-2 rounded-full bg-success" />}
    </button>
  );
}
