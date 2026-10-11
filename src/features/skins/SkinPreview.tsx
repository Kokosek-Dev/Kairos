import { useEffect, useRef } from "react";
import type { SkinVariant } from "@/lib/tauri/skins";

interface SkinPreviewProps {
  src: string;
  variant: SkinVariant;
  scale?: number;
}

interface Part {
  /** Source rectangle in the texture. */
  sx: number;
  sy: number;
  w: number;
  h: number;
  /** Position on the 16x32 canvas. */
  dx: number;
  dy: number;
  /** Overlay layer position in the texture, when the skin has one for this part. */
  ox: number;
  oy: number;
  mirror?: boolean;
}

/** Front view of a skin, drawn pixel by pixel; only a flat preview is needed for the library. */
export function SkinPreview({ src, variant, scale = 5 }: SkinPreviewProps) {
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const image = new Image();
    image.onload = () => {
      const context = canvas.current?.getContext("2d");
      if (!context) return;
      const arm = variant === "slim" ? 3 : 4;
      const modern = image.height >= 64;
      const parts: Part[] = [
        { sx: 8, sy: 8, w: 8, h: 8, dx: 4, dy: 0, ox: 40, oy: 8 },
        { sx: 20, sy: 20, w: 8, h: 12, dx: 4, dy: 8, ox: 20, oy: 36 },
        { sx: 44, sy: 20, w: arm, h: 12, dx: 4 - arm, dy: 8, ox: 44, oy: 36 },
        modern
          ? { sx: 36, sy: 52, w: arm, h: 12, dx: 12, dy: 8, ox: 52, oy: 52 }
          : { sx: 44, sy: 20, w: arm, h: 12, dx: 12, dy: 8, ox: 0, oy: 0, mirror: true },
        { sx: 4, sy: 20, w: 4, h: 12, dx: 4, dy: 20, ox: 4, oy: 36 },
        modern
          ? { sx: 20, sy: 52, w: 4, h: 12, dx: 8, dy: 20, ox: 4, oy: 52 }
          : { sx: 4, sy: 20, w: 4, h: 12, dx: 8, dy: 20, ox: 0, oy: 0, mirror: true },
      ];
      context.imageSmoothingEnabled = false;
      context.clearRect(0, 0, 16, 32);
      for (const part of parts) {
        const layers = modern ? [[part.sx, part.sy], [part.ox, part.oy]] : [[part.sx, part.sy]];
        for (const [x, y] of layers) {
          if (part.mirror) {
            context.save();
            context.translate(part.dx + part.w, part.dy);
            context.scale(-1, 1);
            context.drawImage(image, x, y, part.w, part.h, 0, 0, part.w, part.h);
            context.restore();
          } else {
            context.drawImage(image, x, y, part.w, part.h, part.dx, part.dy, part.w, part.h);
          }
        }
      }
    };
    image.src = src;
  }, [src, variant]);

  return (
    <canvas
      ref={canvas}
      width={16}
      height={32}
      className="[image-rendering:pixelated]"
      style={{ width: 16 * scale, height: 32 * scale }}
    />
  );
}
