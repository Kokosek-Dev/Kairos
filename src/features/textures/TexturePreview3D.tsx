import { useEffect, useRef } from "react";
import { AmbientLight, Box3, DirectionalLight, PerspectiveCamera, Scene, Vector3, WebGLRenderer, type CanvasTexture } from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { PixelDocument } from "@/features/pixel/PixelDocument";
import { loadImage } from "@/features/skins/editor/layout";
import { readPackTexture, type BlockModel } from "@/lib/tauri/texturepacks";
import { buildItem, buildModel, canvasTexture, cubeOf, showFirstFrame, squareOf, type Disposables } from "./modelMeshes";

const VIEW_DIRECTION = new Vector3(0.75, 0.55, 1).normalize();
const FIT = 3.5;

interface TexturePreview3DProps {
  packId: string;
  path: string;
  doc: PixelDocument;
  /** The shape the game draws a block texture on; `undefined` while it is still being looked up. */
  model: BlockModel | null | undefined;
}

/** The texture being drawn, on the shape the game uses for it, turning and following every stroke. */
export function TexturePreview3D({ packId, path, doc, model }: TexturePreview3DProps) {
  const host = useRef<HTMLDivElement>(null);
  const isBlock = path.startsWith("block/");
  const waiting = isBlock && model === undefined;

  useEffect(() => {
    const container = host.current;
    if (!container || waiting) return;
    let disposed = false;

    const renderer = new WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(container.clientWidth, container.clientHeight);
    container.appendChild(renderer.domElement);

    const textures: CanvasTexture[] = [];
    const own: CanvasTexture[] = [];
    const textureFor = (name: string): CanvasTexture => {
      if (`${name}.png` === path) {
        const texture = canvasTexture(doc.canvas);
        own.push(texture);
        textures.push(texture);
        return texture;
      }
      const picture = document.createElement("canvas");
      picture.width = 16;
      picture.height = 16;
      const texture = canvasTexture(picture);
      textures.push(texture);
      readPackTexture(packId, `${name}.png`)
        .then(loadImage)
        .then((image) => {
          if (disposed) return;
          picture.width = image.naturalWidth;
          picture.height = image.naturalHeight;
          picture.getContext("2d")?.drawImage(image, 0, 0);
          showFirstFrame(texture, picture.width, picture.height);
          texture.needsUpdate = true;
        })
        .catch(() => undefined);
      return texture;
    };

    const stem = path.replace(/\.png$/, "");
    const item = path.startsWith("item/") ? buildItem(doc.canvas) : null;
    const built = item ?? buildModel(isBlock ? (model ?? cubeOf(stem)) : squareOf(stem), textureFor);
    const disposables: Disposables = built.disposables;

    const scene = new Scene();
    scene.add(built.group);
    scene.add(new AmbientLight(0xffffff, 1.5));
    const sun = new DirectionalLight(0xffffff, 1.4);
    sun.position.set(2, 3, 2);
    scene.add(sun);

    const bounds = new Box3().setFromObject(built.group);
    const center = bounds.getCenter(new Vector3());
    const reach = Math.max(...bounds.getSize(new Vector3()).toArray(), 0.5) * FIT;
    const camera = new PerspectiveCamera(40, container.clientWidth / container.clientHeight, 0.05, 50);
    camera.position.copy(center).addScaledVector(VIEW_DIRECTION, reach);
    camera.lookAt(center);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.target.copy(center);
    controls.enablePan = false;
    controls.autoRotate = true;
    controls.autoRotateSpeed = 3;
    controls.enableDamping = true;

    const refresh = () => {
      own.forEach((texture) => {
        texture.needsUpdate = true;
      });
      if (item) item.update();
    };
    const unsubscribe = doc.onPixels(refresh);
    renderer.setAnimationLoop(() => {
      controls.update();
      renderer.render(scene, camera);
    });
    const observer = new ResizeObserver(() => {
      renderer.setSize(container.clientWidth, container.clientHeight);
      camera.aspect = container.clientWidth / container.clientHeight;
      camera.updateProjectionMatrix();
    });
    observer.observe(container);

    return () => {
      disposed = true;
      observer.disconnect();
      unsubscribe();
      renderer.setAnimationLoop(null);
      controls.dispose();
      disposables.dispose();
      textures.forEach((texture) => texture.dispose());
      renderer.dispose();
      renderer.domElement.remove();
    };
  }, [doc, packId, path, model, isBlock, waiting]);

  return <div ref={host} className="absolute inset-0 cursor-grab active:cursor-grabbing [&>canvas]:size-full" />;
}
