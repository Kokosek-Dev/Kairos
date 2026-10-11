import { PixelDocument, type Rect } from "@/features/pixel/PixelDocument";
import type { SkinVariant } from "@/lib/tauri/skins";
import { faceAt, paintableMask, paintTemplate, SKIN_SIZE } from "./layout";

export class SkinDocument extends PixelDocument {
  private currentVariant: SkinVariant = "classic";
  private mask = paintableMask("classic");

  constructor() {
    super(SKIN_SIZE, SKIN_SIZE);
    paintTemplate(this.ctx, this.currentVariant);
  }

  get variant(): SkinVariant {
    return this.currentVariant;
  }

  override isPaintable(x: number, y: number): boolean {
    return super.isPaintable(x, y) && this.mask[y * SKIN_SIZE + x] === 1;
  }

  override regionAt(x: number, y: number): Rect | null {
    return faceAt(this.currentVariant, x, y);
  }

  private useVariant(variant: SkinVariant) {
    this.currentVariant = variant;
    this.mask = paintableMask(variant);
  }

  setVariant(variant: SkinVariant) {
    this.useVariant(variant);
    this.changed();
    this.historyChanged();
  }

  /** Swaps in another picture and model. The change can be undone like any stroke. */
  replaceSkin(draw: (ctx: CanvasRenderingContext2D) => void, variant: SkinVariant) {
    this.useVariant(variant);
    this.replace(draw);
  }

  startFromTemplate(variant: SkinVariant) {
    this.replaceSkin((ctx) => paintTemplate(ctx, variant), variant);
  }
}
