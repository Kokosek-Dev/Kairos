import { useEffect, useRef } from "react";
import type { PixelDocument } from "@/features/pixel/PixelDocument";

const TILES = 3;

/** The texture repeated side by side the way a wall of the block looks, so seams between the blocks show. */
export function TilePreview({ doc }: { doc: PixelDocument }) {
  const view = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const target = view.current?.getContext("2d");
    if (!target) return;
    const draw = () => {
      target.imageSmoothingEnabled = false;
      target.clearRect(0, 0, doc.width * TILES, doc.height * TILES);
      for (let row = 0; row < TILES; row++) {
        for (let column = 0; column < TILES; column++) target.drawImage(doc.canvas, column * doc.width, row * doc.height);
      }
    };
    draw();
    return doc.onPixels(draw);
  }, [doc]);

  return (
    <div className="checker absolute inset-0 grid place-items-center p-3 pt-12">
      <canvas ref={view} width={doc.width * TILES} height={doc.height * TILES} className="size-full object-contain [image-rendering:pixelated]" />
    </div>
  );
}
