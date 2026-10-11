import { useEffect, useRef } from "react";
import type { PixelDocument } from "./PixelDocument";
import { applyTool, forEachOnLine, type BrushSettings } from "./tools";

const VIEW_SIZE = 512;
const GRID_MIN_SCALE = 8;

interface PixelCanvasProps {
  doc: PixelDocument;
  brush: BrushSettings;
  onPick: (color: string) => void;
  /** Extra guides drawn above the picture, e.g. the outline of every body part of a skin. */
  guides?: (ctx: CanvasRenderingContext2D, scale: number) => void;
}

function drawGrid(ctx: CanvasRenderingContext2D, doc: PixelDocument, scale: number) {
  ctx.strokeStyle = "rgba(255, 255, 255, 0.12)";
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let x = 1; x < doc.width; x++) {
    ctx.moveTo(x * scale + 0.5, 0);
    ctx.lineTo(x * scale + 0.5, doc.height * scale);
  }
  for (let y = 1; y < doc.height; y++) {
    ctx.moveTo(0, y * scale + 0.5);
    ctx.lineTo(doc.width * scale, y * scale + 0.5);
  }
  ctx.stroke();
}

/** The flat picture, magnified, for precise pixel work. */
export function PixelCanvas({ doc, brush, onPick, guides }: PixelCanvasProps) {
  const view = useRef<HTMLCanvasElement>(null);
  const overlay = useRef<HTMLCanvasElement>(null);
  const live = useRef({ brush, onPick });
  const scale = Math.max(1, Math.floor(VIEW_SIZE / Math.max(doc.width, doc.height)));
  const viewWidth = doc.width * scale;
  const viewHeight = doc.height * scale;

  useEffect(() => {
    live.current = { brush, onPick };
  }, [brush, onPick]);

  useEffect(() => {
    const target = view.current?.getContext("2d");
    const top = overlay.current?.getContext("2d");
    if (!target || !top) return;
    let frame = 0;

    const draw = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        target.imageSmoothingEnabled = false;
        target.clearRect(0, 0, viewWidth, viewHeight);
        target.drawImage(doc.canvas, 0, 0, viewWidth, viewHeight);
        top.clearRect(0, 0, viewWidth, viewHeight);
        if (guides) guides(top, scale);
        else if (scale >= GRID_MIN_SCALE) drawGrid(top, doc, scale);
      });
    };

    const unsubscribe = doc.onPixels(draw);
    draw();
    return () => {
      cancelAnimationFrame(frame);
      unsubscribe();
    };
  }, [doc, guides, scale, viewWidth, viewHeight]);

  useEffect(() => {
    const element = overlay.current;
    if (!element) return;
    let last: { x: number; y: number } | null = null;

    const pixel = (event: PointerEvent) => {
      const bounds = element.getBoundingClientRect();
      return {
        x: Math.floor(((event.clientX - bounds.left) / bounds.width) * doc.width),
        y: Math.floor(((event.clientY - bounds.top) / bounds.height) * doc.height),
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
    <div
      className="checker relative mx-auto w-full max-w-[32rem] overflow-hidden rounded-lg border"
      style={{ aspectRatio: `${doc.width} / ${doc.height}` }}
    >
      <canvas ref={view} width={viewWidth} height={viewHeight} className="absolute inset-0 size-full" />
      <canvas ref={overlay} width={viewWidth} height={viewHeight} className="absolute inset-0 size-full cursor-crosshair touch-none" />
    </div>
  );
}
