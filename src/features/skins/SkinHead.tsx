import { useEffect, useRef, useState } from "react";
import { AccountAvatar } from "@/features/accounts/AccountAvatar";
import { cn } from "@/lib/utils";

interface SkinHeadProps {
  url: string | null | undefined;
  name: string;
  size?: number;
  className?: string;
}

const FACE = { x: 8, y: 8, size: 8 };
const HAT = { x: 40, y: 8, size: 8 };

/**
 * Skins converted from the old 64x32 format get an opaque, black second layer. A real hat layer
 * always has at least one transparent pixel, which is the rule the 3D viewer uses too.
 */
function hasHatLayer(image: HTMLImageElement): boolean {
  if (image.naturalHeight < 64) return false;
  const probe = document.createElement("canvas");
  probe.width = HAT.size;
  probe.height = HAT.size;
  const context = probe.getContext("2d");
  if (!context) return false;
  context.drawImage(image, HAT.x, HAT.y, HAT.size, HAT.size, 0, 0, HAT.size, HAT.size);
  const pixels = context.getImageData(0, 0, HAT.size, HAT.size).data;
  return pixels.some((value, index) => index % 4 === 3 && value < 255);
}

/** The player's face, cut out of the skin texture, with the hat layer on top. */
export function SkinHead({ url, name, size = 32, className }: SkinHeadProps) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const [drawn, setDrawn] = useState(false);

  useEffect(() => {
    setDrawn(false);
    if (!url) return;
    const image = new Image();
    image.crossOrigin = "anonymous";
    image.onload = () => {
      const context = canvas.current?.getContext("2d");
      if (!context) return;
      context.imageSmoothingEnabled = false;
      context.clearRect(0, 0, size, size);
      const layers = hasHatLayer(image) ? [FACE, HAT] : [FACE];
      for (const part of layers) {
        context.drawImage(image, part.x, part.y, part.size, part.size, 0, 0, size, size);
      }
      setDrawn(true);
    };
    image.src = url;
  }, [url, size]);

  return (
    <span className="relative inline-block shrink-0" style={{ width: size, height: size }}>
      <canvas
        ref={canvas}
        width={size}
        height={size}
        className={cn("absolute inset-0 rounded-md [image-rendering:pixelated]", className)}
        style={{ width: size, height: size }}
      />
      {!drawn && <AccountAvatar name={name} className={cn("absolute inset-0", className)} />}
    </span>
  );
}
