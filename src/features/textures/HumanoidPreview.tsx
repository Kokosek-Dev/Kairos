import { useEffect, useRef } from "react";
import { IdleAnimation, SkinViewer } from "skinview3d";
import type { PixelDocument } from "@/features/pixel/PixelDocument";

const HUMANOID = /^entity\/(zombie\/(zombie|husk|drowned)|skeleton\/(skeleton|stray|wither_skeleton|bogged)|player\/(wide|slim)\/[a-z_]+|steve|alex)\.png$/;

/** Whether the texture is laid out like a player's skin, so the player model can wear it. */
export function isHumanoid(path: string, doc: PixelDocument): boolean {
  return HUMANOID.test(path) && doc.width === 64 && (doc.height === 64 || doc.height === 32);
}

/** A texture of a mob built like a player, shown on the player model while it is drawn. */
export function HumanoidPreview({ path, doc }: { path: string; doc: PixelDocument }) {
  const host = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const element = canvas.current;
    const container = host.current;
    if (!element || !container) return;
    const viewer = new SkinViewer({ canvas: element, width: container.clientWidth, height: container.clientHeight, zoom: 0.85 });
    viewer.controls.enablePan = false;
    viewer.autoRotate = true;
    viewer.autoRotateSpeed = 0.5;
    viewer.animation = new IdleAnimation();

    let frame = 0;
    const refresh = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        void viewer.loadSkin(doc.canvas, { model: path.includes("/slim/") || path.endsWith("alex.png") ? "slim" : "default" });
      });
    };
    const unsubscribe = doc.onPixels(refresh);
    const observer = new ResizeObserver(() => viewer.setSize(container.clientWidth, container.clientHeight));
    observer.observe(container);
    refresh();

    return () => {
      cancelAnimationFrame(frame);
      unsubscribe();
      observer.disconnect();
      viewer.dispose();
    };
  }, [doc, path]);

  return (
    <div ref={host} className="absolute inset-0 grid place-items-center">
      <canvas ref={canvas} className="block touch-none cursor-grab active:cursor-grabbing" />
    </div>
  );
}
