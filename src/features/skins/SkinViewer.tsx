import { IdleAnimation, SkinViewer as Viewer } from "skinview3d";
import { useEffect, useRef } from "react";
import type { SkinVariant } from "@/lib/tauri/skins";

interface SkinViewerProps {
  skinUrl: string | null;
  variant: SkinVariant;
  capeUrl: string | null;
  width: number;
  height: number;
}

/** A slowly turning 3D model of the player that can also be dragged around. */
export function SkinViewer({ skinUrl, variant, capeUrl, width, height }: SkinViewerProps) {
  const canvas = useRef<HTMLCanvasElement>(null);
  const viewer = useRef<Viewer | null>(null);

  useEffect(() => {
    if (!canvas.current) return;
    const created = new Viewer({ canvas: canvas.current, width, height });
    created.autoRotate = true;
    created.autoRotateSpeed = 0.6;
    created.animation = new IdleAnimation();
    created.controls.enableZoom = false;
    created.controls.enablePan = false;
    viewer.current = created;
    return () => {
      created.dispose();
      viewer.current = null;
    };
  }, [width, height]);

  useEffect(() => {
    const current = viewer.current;
    if (!current) return;
    if (skinUrl) {
      void current.loadSkin(skinUrl, { model: variant === "slim" ? "slim" : "default" }).catch(() => undefined);
    } else {
      current.resetSkin();
    }
  }, [skinUrl, variant, width, height]);

  useEffect(() => {
    const current = viewer.current;
    if (!current) return;
    if (capeUrl) {
      void current.loadCape(capeUrl, { backEquipment: "cape" }).catch(() => undefined);
    } else {
      current.resetCape();
    }
  }, [capeUrl, width, height]);

  return <canvas ref={canvas} className="cursor-grab active:cursor-grabbing" />;
}
