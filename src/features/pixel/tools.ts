import type { PixelDocument, Rect } from "./PixelDocument";

export type Tool = "pencil" | "eraser" | "fill" | "picker";

export interface BrushSettings {
  tool: Tool;
  color: string;
  size: number;
}

function toHex(r: number, g: number, b: number): string {
  return `#${[r, g, b].map((value) => value.toString(16).padStart(2, "0")).join("")}`;
}

function parseHex(color: string): [number, number, number] {
  const value = Number.parseInt(color.slice(1), 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
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
  switch (brush.tool) {
    case "picker":
      return pick(doc, x, y);
    case "fill": {
      const region = doc.regionAt(x, y);
      if (region) floodFill(doc, region, x, y, brush.color);
      break;
    }
    case "eraser":
      stamp(doc, x, y, brush.size, null);
      break;
    case "pencil":
      stamp(doc, x, y, brush.size, brush.color);
      break;
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
