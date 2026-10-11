import { useEffect, useRef } from "react";
import { IdleAnimation, SkinViewer } from "skinview3d";
import { Raycaster, Vector2 } from "three";
import { applyTool, type BrushSettings } from "@/features/pixel/tools";
import { SKIN_SIZE } from "./layout";
import type { SkinDocument } from "./SkinDocument";

export type Layer = "base" | "overlay";

export interface PaintSettings {
  brush: BrushSettings;
  layer: Layer;
  onPick: (color: string) => void;
}

interface ModelPaneProps {
  doc: SkinDocument;
  showOverlay: boolean;
  /** When set, the model can be painted on directly; otherwise it is a turning preview. */
  paint?: PaintSettings;
}

/** A 3D model that follows the skin being drawn, and optionally lets you paint right on it. */
export function ModelPane({ doc, showOverlay, paint }: ModelPaneProps) {
  const host = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const viewer = useRef<SkinViewer | null>(null);
  const live = useRef({ showOverlay, paint });
  const editable = paint !== undefined;

  useEffect(() => {
    live.current = { showOverlay, paint };
    const outerVisible = showOverlay || paint?.layer === "overlay";
    viewer.current?.playerObject.skin.setOuterLayerVisible(outerVisible);
  }, [showOverlay, paint]);

  useEffect(() => {
    const element = canvas.current;
    const container = host.current;
    if (!element || !container) return;

    const created = new SkinViewer({ canvas: element, width: container.clientWidth, height: container.clientHeight, zoom: 0.85 });
    created.controls.enablePan = false;
    if (!editable) {
      created.autoRotate = true;
      created.autoRotateSpeed = 0.5;
      created.animation = new IdleAnimation();
    }
    viewer.current = created;

    const raycaster = new Raycaster();
    const pointer = new Vector2();
    let frame = 0;
    let stroking = false;

    const refresh = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        created.loadSkin(doc.canvas, { model: doc.variant === "slim" ? "slim" : "default" });
        const { showOverlay: shown, paint: settings } = live.current;
        created.playerObject.skin.setOuterLayerVisible(shown || settings?.layer === "overlay");
      });
    };

    const pixelUnder = (event: PointerEvent): { x: number; y: number } | null => {
      const settings = live.current.paint;
      if (!settings) return null;
      const bounds = element.getBoundingClientRect();
      pointer.set(((event.clientX - bounds.left) / bounds.width) * 2 - 1, -((event.clientY - bounds.top) / bounds.height) * 2 + 1);
      raycaster.setFromCamera(pointer, created.camera);
      const { skin } = created.playerObject;
      const layers = [skin.head, skin.body, skin.rightArm, skin.leftArm, skin.rightLeg, skin.leftLeg].map((part) =>
        settings.layer === "base" ? part.innerLayer : part.outerLayer,
      );
      const uv = raycaster.intersectObjects(layers, true)[0]?.uv;
      return uv ? { x: Math.floor(uv.x * SKIN_SIZE), y: Math.floor((1 - uv.y) * SKIN_SIZE) } : null;
    };

    const paintAt = (event: PointerEvent) => {
      const settings = live.current.paint;
      const point = pixelUnder(event);
      if (!settings || !point) return;
      const picked = applyTool(doc, settings.brush, point.x, point.y);
      if (picked) settings.onPick(picked);
    };

    const onDown = (event: PointerEvent) => {
      if (event.button !== 0 || !pixelUnder(event)) return;
      created.controls.enabled = false;
      element.setPointerCapture(event.pointerId);
      stroking = true;
      if (live.current.paint?.brush.tool !== "picker") doc.beginStroke();
      paintAt(event);
    };

    const onMove = (event: PointerEvent) => {
      if (!editable) return;
      if (stroking) {
        if (live.current.paint?.brush.tool !== "picker") paintAt(event);
      } else {
        element.style.cursor = pixelUnder(event) ? "crosshair" : "grab";
      }
    };

    const onUp = (event: PointerEvent) => {
      if (!stroking) return;
      stroking = false;
      if (live.current.paint?.brush.tool !== "picker") doc.endStroke();
      created.controls.enabled = true;
      element.releasePointerCapture(event.pointerId);
    };

    element.addEventListener("pointerdown", onDown, { capture: true });
    element.addEventListener("pointermove", onMove);
    element.addEventListener("pointerup", onUp);
    element.addEventListener("pointercancel", onUp);
    const unsubscribe = doc.onPixels(refresh);
    const observer = new ResizeObserver(() => created.setSize(container.clientWidth, container.clientHeight));
    observer.observe(container);
    refresh();

    return () => {
      cancelAnimationFrame(frame);
      unsubscribe();
      observer.disconnect();
      element.removeEventListener("pointerdown", onDown, { capture: true });
      element.removeEventListener("pointermove", onMove);
      element.removeEventListener("pointerup", onUp);
      element.removeEventListener("pointercancel", onUp);
      created.dispose();
      viewer.current = null;
    };
  }, [doc, editable]);

  return (
    <div ref={host} className="absolute inset-0 grid place-items-center">
      <canvas ref={canvas} className="block touch-none cursor-grab active:cursor-grabbing" />
    </div>
  );
}
