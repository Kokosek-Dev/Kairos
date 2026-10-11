import type { SkinDocument } from "./document";
import { faceAt, SKIN_SIZE, type Rect } from "./layout";

export type Tool = "pencil" | "eraser" | "fill" | "picker";
export type Layer = "base" | "overlay";

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

function stamp(doc: SkinDocument, x: number, y: number, size: number, color: string | null) {
  const offset = Math.floor((size - 1) / 2);
  for (let row = y - offset; row < y - offset + size; row++) {
    for (let column = x - offset; column < x - offset + size; column++) {
      if (column < 0 || row < 0 || column >= SKIN_SIZE || row >= SKIN_SIZE) continue;
      if (!doc.mask[row * SKIN_SIZE + column]) continue;
      if (color === null) {
        doc.ctx.clearRect(column, row, 1, 1);
      } else {
        doc.ctx.fillStyle = color;
        doc.ctx.fillRect(column, row, 1, 1);
      }
    }
  }
}

function floodFill(doc: SkinDocument, face: Rect, startX: number, startY: number, color: string) {
  const image = doc.ctx.getImageData(face.x, face.y, face.w, face.h);
  const { data } = image;
  const at = (x: number, y: number) => (y * face.w + x) * 4;
  const origin = at(startX - face.x, startY - face.y);
  const target = [data[origin], data[origin + 1], data[origin + 2], data[origin + 3]];
  const [r, g, b] = parseHex(color);
  if (target[0] === r && target[1] === g && target[2] === b && target[3] === 255) return;

  const matches = (index: number) => target.every((value, channel) => data[index + channel] === value);
  const stack = [[startX - face.x, startY - face.y]];
  while (stack.length > 0) {
    const [x, y] = stack.pop() as [number, number];
    if (x < 0 || y < 0 || x >= face.w || y >= face.h) continue;
    const index = at(x, y);
    if (!matches(index)) continue;
    data[index] = r;
    data[index + 1] = g;
    data[index + 2] = b;
    data[index + 3] = 255;
    stack.push([x + 1, y], [x - 1, y], [x, y + 1], [x, y - 1]);
  }
  doc.ctx.putImageData(image, face.x, face.y);
}

function pick(doc: SkinDocument, x: number, y: number): string | null {
  const [r, g, b, a] = doc.ctx.getImageData(x, y, 1, 1).data;
  return a === 0 ? null : toHex(r, g, b);
}

/** Applies the active tool to one pixel. Returns the picked color for the picker tool. */
export function applyTool(doc: SkinDocument, brush: BrushSettings, x: number, y: number): string | null {
  if (x < 0 || y < 0 || x >= SKIN_SIZE || y >= SKIN_SIZE) return null;
  switch (brush.tool) {
    case "picker":
      return pick(doc, x, y);
    case "fill": {
      const face = faceAt(doc.variant, x, y);
      if (face) floodFill(doc, face, x, y, brush.color);
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
