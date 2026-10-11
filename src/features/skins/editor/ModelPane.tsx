import { useEffect, useRef } from "react";
import { IdleAnimation, SkinViewer } from "skinview3d";
import type { SkinDocument } from "./SkinDocument";

interface ModelPaneProps {
  doc: SkinDocument;
  showOverlay: boolean;
}

/** A 3D model that follows the skin being drawn and can be turned around. */
export function ModelPane({ doc, showOverlay }: ModelPaneProps) {
  const host = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const viewer = useRef<SkinViewer | null>(null);
  const overlay = useRef(showOverlay);

  useEffect(() => {
    overlay.current = showOverlay;
    viewer.current?.playerObject.skin.setOuterLayerVisible(showOverlay);
  }, [showOverlay]);

  useEffect(() => {
    const element = canvas.current;
    const container = host.current;
    if (!element || !container) return;

    const created = new SkinViewer({ canvas: element, width: container.clientWidth, height: container.clientHeight, zoom: 0.85 });
    created.autoRotate = true;
    created.autoRotateSpeed = 0.5;
    created.animation = new IdleAnimation();
    created.controls.enablePan = false;
    viewer.current = created;

    let frame = 0;
    const refresh = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        created.loadSkin(doc.canvas, { model: doc.variant === "slim" ? "slim" : "default" });
        created.playerObject.skin.setOuterLayerVisible(overlay.current);
      });
    };

    const unsubscribe = doc.onPixels(refresh);
    const observer = new ResizeObserver(() => created.setSize(container.clientWidth, container.clientHeight));
    observer.observe(container);
    refresh();

    return () => {
      cancelAnimationFrame(frame);
      unsubscribe();
      observer.disconnect();
      created.dispose();
      viewer.current = null;
    };
  }, [doc]);

  return (
    <div ref={host} className="absolute inset-0 grid place-items-center">
      <canvas ref={canvas} className="block cursor-grab active:cursor-grabbing" />
    </div>
  );
}
