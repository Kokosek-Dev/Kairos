import { useEffect, useRef } from "react";
import { SkinViewer } from "skinview3d";
import { Raycaster, Vector2 } from "three";
import type { SkinDocument } from "./document";
import { SKIN_SIZE } from "./layout";
import { applyTool, type BrushSettings, type Layer } from "./tools";

interface ModelPaneProps {
  doc: SkinDocument;
  brush: BrushSettings;
  layer: Layer;
  showOverlay: boolean;
  onPick: (color: string) => void;
}

/** A 3D model of the skin that can be turned around and painted on directly. */
export function ModelPane({ doc, brush, layer, showOverlay, onPick }: ModelPaneProps) {
  const host = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const viewer = useRef<SkinViewer | null>(null);
  const live = useRef({ brush, layer, showOverlay, onPick });

  useEffect(() => {
    live.current = { brush, layer, showOverlay, onPick };
    const current = viewer.current;
    if (current) current.playerObject.skin.setOuterLayerVisible(showOverlay || layer === "overlay");
  }, [brush, layer, showOverlay, onPick]);

  useEffect(() => {
    const element = canvas.current;
    const container = host.current;
    if (!element || !container) return;

    const created = new SkinViewer({ canvas: element, width: container.clientWidth, height: container.clientHeight, zoom: 0.85 });
    created.controls.enablePan = false;
    viewer.current = created;

    const raycaster = new Raycaster();
    const pointer = new Vector2();
    let frame = 0;
    let stroking = false;

    const refresh = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        created.loadSkin(doc.canvas, { model: doc.variant === "slim" ? "slim" : "default" });
        const { layer: activeLayer, showOverlay: overlayShown } = live.current;
        created.playerObject.skin.setOuterLayerVisible(overlayShown || activeLayer === "overlay");
      });
    };

    const pixelUnder = (event: PointerEvent): { x: number; y: number } | null => {
      const bounds = element.getBoundingClientRect();
      pointer.set(((event.clientX - bounds.left) / bounds.width) * 2 - 1, -((event.clientY - bounds.top) / bounds.height) * 2 + 1);
      raycaster.setFromCamera(pointer, created.camera);
      const { skin } = created.playerObject;
      const layers = [skin.head, skin.body, skin.rightArm, skin.leftArm, skin.rightLeg, skin.leftLeg].map((part) =>
        live.current.layer === "base" ? part.innerLayer : part.outerLayer,
      );
      const uv = raycaster.intersectObjects(layers, true)[0]?.uv;
      return uv ? { x: Math.floor(uv.x * SKIN_SIZE), y: Math.floor((1 - uv.y) * SKIN_SIZE) } : null;
    };

    const paint = (event: PointerEvent) => {
      const point = pixelUnder(event);
      if (!point) return;
      const picked = applyTool(doc, live.current.brush, point.x, point.y);
      if (picked) live.current.onPick(picked);
    };

    const onDown = (event: PointerEvent) => {
      if (event.button !== 0 || !pixelUnder(event)) return;
      created.controls.enabled = false;
      element.setPointerCapture(event.pointerId);
      stroking = true;
      if (live.current.brush.tool !== "picker") doc.beginStroke();
      paint(event);
    };

    const onMove = (event: PointerEvent) => {
      if (stroking) {
        if (live.current.brush.tool !== "picker") paint(event);
      } else {
        element.style.cursor = pixelUnder(event) ? "crosshair" : "grab";
      }
    };

    const onUp = (event: PointerEvent) => {
      if (!stroking) return;
      stroking = false;
      if (live.current.brush.tool !== "picker") doc.endStroke();
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
  }, [doc]);

  return (
    <div ref={host} className="absolute inset-0 grid place-items-center">
      <canvas ref={canvas} className="block touch-none" />
    </div>
  );
}
