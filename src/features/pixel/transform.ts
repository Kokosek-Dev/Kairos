import { PixelDocument } from "./PixelDocument";

function copyOf(doc: PixelDocument): HTMLCanvasElement {
  const copy = document.createElement("canvas");
  copy.width = doc.width;
  copy.height = doc.height;
  copy.getContext("2d")?.drawImage(doc.canvas, 0, 0);
  return copy;
}

/** Mirrors the whole picture from left to right, or from top to bottom. */
export function flip(doc: PixelDocument, axis: "horizontal" | "vertical") {
  const copy = copyOf(doc);
  doc.replace((ctx) => {
    ctx.clearRect(0, 0, doc.width, doc.height);
    ctx.save();
    if (axis === "horizontal") ctx.setTransform(-1, 0, 0, 1, doc.width, 0);
    else ctx.setTransform(1, 0, 0, -1, 0, doc.height);
    ctx.drawImage(copy, 0, 0);
    ctx.restore();
  });
}

export function canRotate(doc: PixelDocument): boolean {
  return doc.width === doc.height;
}

/** Turns a square picture a quarter turn clockwise. */
export function rotate(doc: PixelDocument) {
  if (!canRotate(doc)) return;
  const copy = copyOf(doc);
  doc.replace((ctx) => {
    ctx.clearRect(0, 0, doc.width, doc.height);
    ctx.save();
    ctx.setTransform(0, 1, -1, 0, doc.width, 0);
    ctx.drawImage(copy, 0, 0);
    ctx.restore();
  });
}

/** Moves every colour towards white (a positive amount) or black (a negative one), keeping the see-through parts. */
export function shade(doc: PixelDocument, amount: number) {
  doc.replace((ctx) => {
    const image = ctx.getImageData(0, 0, doc.width, doc.height);
    const target = amount > 0 ? 255 : 0;
    const strength = Math.abs(amount);
    for (let index = 0; index < image.data.length; index += 4) {
      if (image.data[index + 3] === 0) continue;
      for (let channel = 0; channel < 3; channel++) {
        image.data[index + channel] = Math.round(image.data[index + channel] + (target - image.data[index + channel]) * strength);
      }
    }
    ctx.putImageData(image, 0, 0);
  });
}

/** Moves the picture by whole pixels, with what leaves one edge coming back in at the other. */
export function shift(doc: PixelDocument, dx: number, dy: number) {
  const copy = copyOf(doc);
  doc.replace((ctx) => {
    ctx.clearRect(0, 0, doc.width, doc.height);
    for (const x of [dx - doc.width, dx, dx + doc.width]) {
      for (const y of [dy - doc.height, dy, dy + doc.height]) ctx.drawImage(copy, x, y);
    }
  });
}

/** The same picture at another size, without blurring the pixels. */
export function resized(doc: PixelDocument, width: number, height: number): PixelDocument {
  const next = new PixelDocument(width, height);
  next.load((ctx) => {
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(doc.canvas, 0, 0, width, height);
  });
  next.markUnsaved();
  return next;
}
