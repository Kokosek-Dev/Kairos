import { useEffect, useRef } from "react";
import type { PixelDocument } from "./PixelDocument";
import { applyTool, drawShape, forEachOnLine, SHAPE_TOOLS, type BrushSettings } from "./tools";

const DEFAULT_VIEW_SIZE = 512;
const GRID_MIN_SCALE = 8;
const MAX_HEIGHT = 576;

interface PixelCanvasProps {
  doc: PixelDocument;
  brush: BrushSettings;
  onPick: (color: string) => void;
  /** Extra guides drawn above the picture, e.g. the outline of every body part of a skin. */
  guides?: (ctx: CanvasRenderingContext2D, scale: number) => void;
  /** Pixel size of the longest side of the drawing area. */
  viewSize?: number;
  /** The tallest the drawing area may get, in pixels. */
  maxHeight?: number;
  /** The pixel under the pointer, or null when it leaves the picture. */
  onHover?: (pixel: { x: number; y: number } | null) => void;
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
export function PixelCanvas({ doc, brush, onPick, guides, viewSize = DEFAULT_VIEW_SIZE, maxHeight = MAX_HEIGHT, onHover }: PixelCanvasProps) {
  const view = useRef<HTMLCanvasElement>(null);
  const overlay = useRef<HTMLCanvasElement>(null);
  const live = useRef({ brush, onPick, onHover });
  const scale = Math.max(1, Math.floor(viewSize / Math.max(doc.width, doc.height)));
  const viewWidth = doc.width * scale;
  const viewHeight = doc.height * scale;

  useEffect(() => {
    live.current = { brush, onPick, onHover };
  }, [brush, onPick, onHover]);

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
    let origin: { x: number; y: number } | null = null;

    const pixel = (event: PointerEvent) => {
      const bounds = element.getBoundingClientRect();
      return {
        x: Math.floor(((event.clientX - bounds.left) / bounds.width) * doc.width),
        y: Math.floor(((event.clientY - bounds.top) / bounds.height) * doc.height),
      };
    };

    const clamped = (event: PointerEvent) => {
      const { x, y } = pixel(event);
      return { x: Math.min(doc.width - 1, Math.max(0, x)), y: Math.min(doc.height - 1, Math.max(0, y)) };
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
      if (SHAPE_TOOLS.includes(live.current.brush.tool)) {
        origin = clamped(event);
        drawShape(doc, live.current.brush, [origin.x, origin.y], [origin.x, origin.y]);
        return;
      }
      paint(last.x, last.y);
    };

    const onMove = (event: PointerEvent) => {
      const hovered = pixel(event);
      const inside = hovered.x >= 0 && hovered.y >= 0 && hovered.x < doc.width && hovered.y < doc.height;
      live.current.onHover?.(inside ? hovered : null);
      if (!last || live.current.brush.tool === "picker") return;
      if (origin) {
        const end = clamped(event);
        doc.revertStroke();
        drawShape(doc, live.current.brush, [origin.x, origin.y], [end.x, end.y]);
        return;
      }
      const next = pixel(event);
      forEachOnLine(last.x, last.y, next.x, next.y, paint);
      last = next;
    };

    const onUp = (event: PointerEvent) => {
      if (!last) return;
      last = null;
      origin = null;
      if (live.current.brush.tool !== "picker") doc.endStroke();
      element.releasePointerCapture(event.pointerId);
    };

    element.addEventListener("pointerdown", onDown);
    element.addEventListener("pointermove", onMove);
    element.addEventListener("pointerup", onUp);
    element.addEventListener("pointercancel", onUp);
    const onLeave = () => live.current.onHover?.(null);
    element.addEventListener("pointerleave", onLeave);
    return () => {
      element.removeEventListener("pointerleave", onLeave);
      element.removeEventListener("pointerdown", onDown);
      element.removeEventListener("pointermove", onMove);
      element.removeEventListener("pointerup", onUp);
      element.removeEventListener("pointercancel", onUp);
    };
  }, [doc]);

  return (
    <div
      className="checker relative mx-auto w-full overflow-hidden rounded-lg border"
      style={{ aspectRatio: `${doc.width} / ${doc.height}`, maxWidth: `${Math.min(viewSize, (maxHeight * doc.width) / doc.height)}px` }}
    >
      <canvas ref={view} width={viewWidth} height={viewHeight} className="absolute inset-0 size-full" />
      <canvas ref={overlay} width={viewWidth} height={viewHeight} className="absolute inset-0 size-full cursor-crosshair touch-none" />
    </div>
  );
}
