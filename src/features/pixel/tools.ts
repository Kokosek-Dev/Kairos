import type { PixelDocument, Rect } from "./PixelDocument";

export type Tool = "pencil" | "eraser" | "fill" | "picker" | "line" | "rect" | "ellipse";
export type Mirror = "none" | "x" | "y" | "both";

export interface BrushSettings {
  tool: Tool;
  color: string;
  size: number;
  /** Everything drawn is repeated across the middle of the picture. */
  mirror: Mirror;
  /** Rectangles and ellipses are filled instead of just an outline. */
  filled: boolean;
}

export const SHAPE_TOOLS: Tool[] = ["line", "rect", "ellipse"];

type Point = [number, number];

function toHex(r: number, g: number, b: number): string {
  return `#${[r, g, b].map((value) => value.toString(16).padStart(2, "0")).join("")}`;
}

function parseHex(color: string): [number, number, number] {
  const value = Number.parseInt(color.slice(1), 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

/** The point itself and where the mirror puts it. */
function withMirror(doc: PixelDocument, mirror: Mirror, x: number, y: number): Point[] {
  const across = doc.width - 1 - x;
  const down = doc.height - 1 - y;
  switch (mirror) {
    case "x":
      return [[x, y], [across, y]];
    case "y":
      return [[x, y], [x, down]];
    case "both":
      return [[x, y], [across, y], [x, down], [across, down]];
    default:
      return [[x, y]];
  }
}

function stamp(doc: PixelDocument, x: number, y: number, size: number, color: string | null) {
  const offset = Math.floor((size - 1) / 2);
  for (let row = y - offset; row < y - offset + size; row++) {
    for (let column = x - offset; column < x - offset + size; column++) {
      if (!doc.isPaintable(column, row)) continue;
      if (color === null) {
        doc.ctx.clearRect(column, row, 1, 1);
      } else {
        doc.ctx.fillStyle = color;
        doc.ctx.fillRect(column, row, 1, 1);
      }
    }
  }
}

function floodFill(doc: PixelDocument, region: Rect, startX: number, startY: number, color: string) {
  const image = doc.ctx.getImageData(region.x, region.y, region.w, region.h);
  const { data } = image;
  const at = (x: number, y: number) => (y * region.w + x) * 4;
  const origin = at(startX - region.x, startY - region.y);
  const target = [data[origin], data[origin + 1], data[origin + 2], data[origin + 3]];
  const [r, g, b] = parseHex(color);
  if (target[0] === r && target[1] === g && target[2] === b && target[3] === 255) return;

  const matches = (index: number) => target.every((value, channel) => data[index + channel] === value);
  const stack = [[startX - region.x, startY - region.y]];
  while (stack.length > 0) {
    const [x, y] = stack.pop() as [number, number];
    if (x < 0 || y < 0 || x >= region.w || y >= region.h) continue;
    const index = at(x, y);
    if (!matches(index)) continue;
    data[index] = r;
    data[index + 1] = g;
    data[index + 2] = b;
    data[index + 3] = 255;
    stack.push([x + 1, y], [x - 1, y], [x, y + 1], [x, y - 1]);
  }
  doc.ctx.putImageData(image, region.x, region.y);
}

function pick(doc: PixelDocument, x: number, y: number): string | null {
  const [r, g, b, a] = doc.ctx.getImageData(x, y, 1, 1).data;
  return a === 0 ? null : toHex(r, g, b);
}

/** Applies the active tool to one pixel. Returns the picked color for the picker tool. */
export function applyTool(doc: PixelDocument, brush: BrushSettings, x: number, y: number): string | null {
  if (x < 0 || y < 0 || x >= doc.width || y >= doc.height) return null;
  if (brush.tool === "picker") return pick(doc, x, y);
  for (const [column, row] of withMirror(doc, brush.mirror, x, y)) {
    switch (brush.tool) {
      case "fill": {
        const region = doc.regionAt(column, row);
        if (region) floodFill(doc, region, column, row, brush.color);
        break;
      }
      case "eraser":
        stamp(doc, column, row, brush.size, null);
        break;
      default:
        stamp(doc, column, row, brush.size, brush.color);
    }
  }
  doc.changed();
  return null;
}

/** Visits every pixel on the straight line between two points, so fast strokes leave no gaps. */
export function forEachOnLine(x0: number, y0: number, x1: number, y1: number, visit: (x: number, y: number) => void) {
  const steps = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0), 1);
  for (let step = 0; step <= steps; step++) {
    visit(Math.round(x0 + ((x1 - x0) * step) / steps), Math.round(y0 + ((y1 - y0) * step) / steps));
  }
}

function forEachInRect(x0: number, y0: number, x1: number, y1: number, filled: boolean, visit: (x: number, y: number) => void) {
  const [left, right] = [Math.min(x0, x1), Math.max(x0, x1)];
  const [top, bottom] = [Math.min(y0, y1), Math.max(y0, y1)];
  for (let y = top; y <= bottom; y++) {
    for (let x = left; x <= right; x++) {
      if (filled || x === left || x === right || y === top || y === bottom) visit(x, y);
    }
  }
}

/** The ellipse inside the box between two corners: a pixel belongs to it when it lies within half a pixel of the curve. */
function forEachInEllipse(x0: number, y0: number, x1: number, y1: number, filled: boolean, visit: (x: number, y: number) => void) {
  const [left, right] = [Math.min(x0, x1), Math.max(x0, x1)];
  const [top, bottom] = [Math.min(y0, y1), Math.max(y0, y1)];
  const radiusX = (right - left + 1) / 2;
  const radiusY = (bottom - top + 1) / 2;
  const inside = (x: number, y: number) => ((x + 0.5 - left - radiusX) / radiusX) ** 2 + ((y + 0.5 - top - radiusY) / radiusY) ** 2 <= 1;
  for (let y = top; y <= bottom; y++) {
    for (let x = left; x <= right; x++) {
      if (!inside(x, y)) continue;
      if (filled || !inside(x - 1, y) || !inside(x + 1, y) || !inside(x, y - 1) || !inside(x, y + 1)) visit(x, y);
    }
  }
}

/** Draws the shape the active tool makes between where the drag started and where it is now. */
export function drawShape(doc: PixelDocument, brush: BrushSettings, from: Point, to: Point) {
  const paint = (x: number, y: number) => {
    for (const [column, row] of withMirror(doc, brush.mirror, x, y)) stamp(doc, column, row, brush.size, brush.color);
  };
  if (brush.tool === "line") forEachOnLine(from[0], from[1], to[0], to[1], paint);
  else if (brush.tool === "rect") forEachInRect(from[0], from[1], to[0], to[1], brush.filled, paint);
  else if (brush.tool === "ellipse") forEachInEllipse(from[0], from[1], to[0], to[1], brush.filled, paint);
  doc.changed();
}
