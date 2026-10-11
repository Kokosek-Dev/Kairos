import { useEffect, useRef } from "react";
import type { SkinDocument } from "./document";
import { faceRects, SKIN_SIZE } from "./layout";
import { applyTool, forEachOnLine, type BrushSettings } from "./tools";

const SCALE = 8;
const VIEW = SKIN_SIZE * SCALE;

interface FlatPaneProps {
  doc: SkinDocument;
  brush: BrushSettings;
  onPick: (color: string) => void;
}

function drawGuides(ctx: CanvasRenderingContext2D, doc: SkinDocument) {
  ctx.clearRect(0, 0, VIEW, VIEW);
  ctx.fillStyle = "rgba(0, 0, 0, 0.55)";
  ctx.fillRect(0, 0, VIEW, VIEW);
  const rects = faceRects(doc.variant);
  for (const { x, y, w, h } of rects) ctx.clearRect(x * SCALE, y * SCALE, w * SCALE, h * SCALE);
  ctx.strokeStyle = "rgba(255, 255, 255, 0.35)";
  ctx.lineWidth = 1;
  for (const { x, y, w, h } of rects) ctx.strokeRect(x * SCALE + 0.5, y * SCALE + 0.5, w * SCALE - 1, h * SCALE - 1);
}

/** The flat skin texture, magnified, for precise pixel work. */
export function FlatPane({ doc, brush, onPick }: FlatPaneProps) {
  const view = useRef<HTMLCanvasElement>(null);
  const guides = useRef<HTMLCanvasElement>(null);
  const live = useRef({ brush, onPick });

  useEffect(() => {
    live.current = { brush, onPick };
  }, [brush, onPick]);

  useEffect(() => {
    const target = view.current?.getContext("2d");
    const overlay = guides.current?.getContext("2d");
    if (!target || !overlay) return;
    let frame = 0;

    const draw = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        target.imageSmoothingEnabled = false;
        target.clearRect(0, 0, VIEW, VIEW);
        target.drawImage(doc.canvas, 0, 0, VIEW, VIEW);
        drawGuides(overlay, doc);
      });
    };

    const unsubscribe = doc.onPixels(draw);
    draw();
    return () => {
      cancelAnimationFrame(frame);
      unsubscribe();
    };
  }, [doc]);

  useEffect(() => {
    const element = guides.current;
    if (!element) return;
    let last: { x: number; y: number } | null = null;

    const pixel = (event: PointerEvent) => {
      const bounds = element.getBoundingClientRect();
      return {
        x: Math.floor(((event.clientX - bounds.left) / bounds.width) * SKIN_SIZE),
        y: Math.floor(((event.clientY - bounds.top) / bounds.height) * SKIN_SIZE),
      };
    };

    const paint = (x: number, y: number) => {
      const picked = applyTool(doc, live.current.brush, x, y);
      if (picked) live.current.onPick(picked);
    };

    const onDown = (event: PointerEvent) => {
      if (event.button !== 0) return;
      element.setPointerCapture(event.pointerId);
      if (live.current.brush.tool !== "picker") doc.beginStroke();
      last = pixel(event);
      paint(last.x, last.y);
    };

    const onMove = (event: PointerEvent) => {
      if (!last || live.current.brush.tool === "picker") return;
      const next = pixel(event);
      forEachOnLine(last.x, last.y, next.x, next.y, paint);
      last = next;
    };

    const onUp = (event: PointerEvent) => {
      if (!last) return;
      last = null;
      if (live.current.brush.tool !== "picker") doc.endStroke();
      element.releasePointerCapture(event.pointerId);
    };

    element.addEventListener("pointerdown", onDown);
    element.addEventListener("pointermove", onMove);
    element.addEventListener("pointerup", onUp);
    element.addEventListener("pointercancel", onUp);
    return () => {
      element.removeEventListener("pointerdown", onDown);
      element.removeEventListener("pointermove", onMove);
      element.removeEventListener("pointerup", onUp);
      element.removeEventListener("pointercancel", onUp);
    };
  }, [doc]);

  return (
    <div className="checker relative aspect-square w-full max-w-[32rem] overflow-hidden rounded-lg border">
      <canvas ref={view} width={VIEW} height={VIEW} className="absolute inset-0 size-full" />
      <canvas ref={guides} width={VIEW} height={VIEW} className="absolute inset-0 size-full cursor-crosshair touch-none" />
    </div>
  );
}
