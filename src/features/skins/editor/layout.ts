import type { Rect } from "@/features/pixel/PixelDocument";
import type { SkinVariant } from "@/lib/tauri/skins";

export const SKIN_SIZE = 64;

type Size = [width: number, height: number, depth: number];

export type Part = "head" | "body" | "rightLeg" | "leftLeg" | "rightArm" | "leftArm";

interface Box {
  part: Part;
  origin: [number, number];
  size: Size;
  overlay: boolean;
}

interface Faces {
  top: Rect;
  bottom: Rect;
  right: Rect;
  front: Rect;
  left: Rect;
  back: Rect;
}

const SKIN_TONE = "#d9a679";
const HAIR = "#5a3b24";
const SHIRT = "#2f8fd8";
const TROUSERS = "#4a47a3";

function facesOf([u, v]: [number, number], [w, h, d]: Size): Faces {
  return {
    top: { x: u + d, y: v, w, h: d },
    bottom: { x: u + d + w, y: v, w, h: d },
    right: { x: u, y: v + d, w: d, h },
    front: { x: u + d, y: v + d, w, h },
    left: { x: u + d + w, y: v + d, w: d, h },
    back: { x: u + 2 * d + w, y: v + d, w, h },
  };
}

function boxes(variant: SkinVariant): Box[] {
  const arm: Size = [variant === "slim" ? 3 : 4, 12, 4];
  return [
    { part: "head", origin: [0, 0], size: [8, 8, 8], overlay: false },
    { part: "body", origin: [16, 16], size: [8, 12, 4], overlay: false },
    { part: "rightLeg", origin: [0, 16], size: [4, 12, 4], overlay: false },
    { part: "leftLeg", origin: [16, 48], size: [4, 12, 4], overlay: false },
    { part: "rightArm", origin: [40, 16], size: arm, overlay: false },
    { part: "leftArm", origin: [32, 48], size: arm, overlay: false },
    { part: "head", origin: [32, 0], size: [8, 8, 8], overlay: true },
    { part: "body", origin: [16, 32], size: [8, 12, 4], overlay: true },
    { part: "rightLeg", origin: [0, 32], size: [4, 12, 4], overlay: true },
    { part: "leftLeg", origin: [0, 48], size: [4, 12, 4], overlay: true },
    { part: "rightArm", origin: [40, 32], size: arm, overlay: true },
    { part: "leftArm", origin: [48, 48], size: arm, overlay: true },
  ];
}

export function faceRects(variant: SkinVariant): Rect[] {
  return boxes(variant).flatMap(({ origin, size }) => Object.values(facesOf(origin, size)));
}

export interface PartLabel {
  part: Part;
  overlay: boolean;
  rect: Rect;
}

/** The front face of every body part, where its name is shown on the flat texture. */
export function partLabels(variant: SkinVariant): PartLabel[] {
  return boxes(variant).map(({ part, origin, size, overlay }) => ({ part, overlay, rect: facesOf(origin, size).front }));
}

export function paintableMask(variant: SkinVariant): Uint8Array {
  const mask = new Uint8Array(SKIN_SIZE * SKIN_SIZE);
  for (const { x, y, w, h } of faceRects(variant)) {
    for (let row = y; row < y + h; row++) mask.fill(1, row * SKIN_SIZE + x, row * SKIN_SIZE + x + w);
  }
  return mask;
}

export function faceAt(variant: SkinVariant, x: number, y: number): Rect | null {
  return faceRects(variant).find((rect) => x >= rect.x && x < rect.x + rect.w && y >= rect.y && y < rect.y + rect.h) ?? null;
}

function fill(ctx: CanvasRenderingContext2D, rect: Rect, color: string) {
  ctx.fillStyle = color;
  ctx.fillRect(rect.x, rect.y, rect.w, rect.h);
}

/** A neutral starting point that makes every body part easy to recognise. */
export function paintTemplate(ctx: CanvasRenderingContext2D, variant: SkinVariant) {
  ctx.clearRect(0, 0, SKIN_SIZE, SKIN_SIZE);
  const colors = [SKIN_TONE, SHIRT, TROUSERS, TROUSERS, SKIN_TONE, SKIN_TONE];
  boxes(variant)
    .filter((box) => !box.overlay)
    .forEach(({ origin, size }, index) => {
      for (const rect of Object.values(facesOf(origin, size))) fill(ctx, rect, colors[index]);
    });

  const head = facesOf([0, 0], [8, 8, 8]);
  fill(ctx, head.top, HAIR);
  fill(ctx, head.back, HAIR);
  for (const side of [head.front, head.right, head.left]) fill(ctx, { ...side, h: 2 }, HAIR);
  fill(ctx, { x: head.front.x + 1, y: head.front.y + 4, w: 2, h: 1 }, "#ffffff");
  fill(ctx, { x: head.front.x + 5, y: head.front.y + 4, w: 2, h: 1 }, "#ffffff");
  fill(ctx, { x: head.front.x + 2, y: head.front.y + 4, w: 1, h: 1 }, "#2b2b6b");
  fill(ctx, { x: head.front.x + 5, y: head.front.y + 4, w: 1, h: 1 }, "#2b2b6b");
}

function mirror(ctx: CanvasRenderingContext2D, from: Rect, to: Rect) {
  ctx.save();
  ctx.translate(to.x + to.w, to.y);
  ctx.scale(-1, 1);
  ctx.drawImage(ctx.canvas, from.x, from.y, from.w, from.h, 0, 0, to.w, to.h);
  ctx.restore();
}

function mirrorBox(ctx: CanvasRenderingContext2D, from: Faces, to: Faces) {
  mirror(ctx, from.top, to.top);
  mirror(ctx, from.bottom, to.bottom);
  mirror(ctx, from.front, to.front);
  mirror(ctx, from.back, to.back);
  mirror(ctx, from.right, to.left);
  mirror(ctx, from.left, to.right);
}

/** Draws an image onto the canvas, upgrading the legacy 64x32 layout by mirroring the right limbs. */
export function drawSkinImage(ctx: CanvasRenderingContext2D, image: HTMLImageElement) {
  ctx.clearRect(0, 0, SKIN_SIZE, SKIN_SIZE);
  ctx.drawImage(image, 0, 0);
  if (image.naturalHeight === 32) {
    const leg: Size = [4, 12, 4];
    mirrorBox(ctx, facesOf([0, 16], leg), facesOf([16, 48], leg));
    mirrorBox(ctx, facesOf([40, 16], leg), facesOf([32, 48], leg));
  }
}

export function loadImage(source: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error("the skin image could not be read"));
    image.src = source;
  });
}
