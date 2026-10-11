import type { SkinVariant } from "@/lib/tauri/skins";
import { createSkinCanvas, paintableMask, paintTemplate, SKIN_SIZE } from "./layout";

const MAX_HISTORY = 100;

/** The skin being edited, with undo history. Painting notifies listeners without going through React. */
export class SkinDocument {
  readonly canvas = createSkinCanvas();
  readonly ctx = this.canvas.getContext("2d", { willReadFrequently: true }) as CanvasRenderingContext2D;
  private undoStack: ImageData[] = [];
  private redoStack: ImageData[] = [];
  private pending: ImageData | null = null;
  private pixelListeners = new Set<() => void>();
  private historyListeners = new Set<() => void>();
  private currentVariant: SkinVariant = "classic";
  private currentMask = paintableMask("classic");
  private historyRevision = 0;

  constructor() {
    paintTemplate(this.ctx, this.currentVariant);
  }

  get variant(): SkinVariant {
    return this.currentVariant;
  }

  get mask(): Uint8Array {
    return this.currentMask;
  }

  get revision(): number {
    return this.historyRevision;
  }

  get canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  get canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  onPixels = (listener: () => void): (() => void) => {
    this.pixelListeners.add(listener);
    return () => this.pixelListeners.delete(listener);
  };

  onHistory =(listener: () => void): (() => void) => {
    this.historyListeners.add(listener);
    return () => this.historyListeners.delete(listener);
  };

  changed() {
    this.pixelListeners.forEach((listener) => listener());
  }

  private historyChanged() {
    this.historyRevision += 1;
    this.historyListeners.forEach((listener) => listener());
  }

  private snapshot(): ImageData {
    return this.ctx.getImageData(0, 0, SKIN_SIZE, SKIN_SIZE);
  }

  private restore(image: ImageData) {
    this.ctx.putImageData(image, 0, 0);
    this.changed();
  }

  beginStroke() {
    this.pending = this.snapshot();
  }

  endStroke() {
    if (!this.pending) return;
    this.undoStack.push(this.pending);
    if (this.undoStack.length > MAX_HISTORY) this.undoStack.shift();
    this.redoStack = [];
    this.pending = null;
    this.historyChanged();
  }

  undo() {
    const previous = this.undoStack.pop();
    if (!previous) return;
    this.redoStack.push(this.snapshot());
    this.restore(previous);
    this.historyChanged();
  }

  redo() {
    const next = this.redoStack.pop();
    if (!next) return;
    this.undoStack.push(this.snapshot());
    this.restore(next);
    this.historyChanged();
  }

  setVariant(variant: SkinVariant) {
    this.currentVariant = variant;
    this.currentMask = paintableMask(variant);
    this.changed();
    this.historyChanged();
  }

  /** Replaces the whole picture. The change can be undone like any stroke. */
  replace(draw: (ctx: CanvasRenderingContext2D) => void, variant: SkinVariant) {
    this.beginStroke();
    draw(this.ctx);
    this.currentVariant = variant;
    this.currentMask = paintableMask(variant);
    this.endStroke();
    this.changed();
  }

  startFromTemplate(variant: SkinVariant) {
    this.replace((ctx) => paintTemplate(ctx, variant), variant);
  }

  toPngBase64(): string {
    return this.canvas.toDataURL("image/png").split(",")[1];
  }
}
