const MAX_HISTORY = 100;

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** A picture being edited, with undo history. Painting notifies listeners without going through React. */
export class PixelDocument {
  readonly canvas = document.createElement("canvas");
  readonly ctx: CanvasRenderingContext2D;
  private undoStack: ImageData[] = [];
  private redoStack: ImageData[] = [];
  private pending: ImageData | null = null;
  private pixelListeners = new Set<() => void>();
  private historyListeners = new Set<() => void>();
  private historyRevision = 0;
  private unsaved = false;

  constructor(
    readonly width: number,
    readonly height: number,
  ) {
    this.canvas.width = width;
    this.canvas.height = height;
    this.ctx = this.canvas.getContext("2d", { willReadFrequently: true }) as CanvasRenderingContext2D;
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

  get dirty(): boolean {
    return this.unsaved;
  }

  /** Whether the pixel belongs to a part of the picture that may be painted. */
  isPaintable(x: number, y: number): boolean {
    return x >= 0 && y >= 0 && x < this.width && y < this.height;
  }

  /** The region a fill stays inside when started at this pixel. */
  regionAt(x: number, y: number): Rect | null {
    return this.isPaintable(x, y) ? { x: 0, y: 0, w: this.width, h: this.height } : null;
  }

  onPixels = (listener: () => void): (() => void) => {
    this.pixelListeners.add(listener);
    return () => this.pixelListeners.delete(listener);
  };

  onHistory = (listener: () => void): (() => void) => {
    this.historyListeners.add(listener);
    return () => this.historyListeners.delete(listener);
  };

  changed() {
    this.pixelListeners.forEach((listener) => listener());
  }

  protected historyChanged() {
    this.historyRevision += 1;
    this.historyListeners.forEach((listener) => listener());
  }

  private snapshot(): ImageData {
    return this.ctx.getImageData(0, 0, this.width, this.height);
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
    this.unsaved = true;
    this.historyChanged();
  }

  undo() {
    const previous = this.undoStack.pop();
    if (!previous) return;
    this.redoStack.push(this.snapshot());
    this.unsaved = true;
    this.restore(previous);
    this.historyChanged();
  }

  redo() {
    const next = this.redoStack.pop();
    if (!next) return;
    this.undoStack.push(this.snapshot());
    this.unsaved = true;
    this.restore(next);
    this.historyChanged();
  }

  /** Replaces the whole picture. The change can be undone like any stroke. */
  replace(draw: (ctx: CanvasRenderingContext2D) => void) {
    this.beginStroke();
    draw(this.ctx);
    this.endStroke();
    this.changed();
  }

  /** Starts over with a picture that is not an edit yet, e.g. a freshly opened file. */
  load(draw: (ctx: CanvasRenderingContext2D) => void) {
    this.undoStack = [];
    this.redoStack = [];
    this.pending = null;
    this.ctx.clearRect(0, 0, this.width, this.height);
    draw(this.ctx);
    this.unsaved = false;
    this.changed();
    this.historyChanged();
  }

  markSaved() {
    this.unsaved = false;
    this.historyChanged();
  }

  toPngBase64(): string {
    return this.canvas.toDataURL("image/png").split(",")[1];
  }
}
