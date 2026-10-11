import { useEffect, useRef } from "react";
import {
  AmbientLight,
  BoxGeometry,
  CanvasTexture,
  DirectionalLight,
  DoubleSide,
  Mesh,
  MeshBasicMaterial,
  MeshLambertMaterial,
  NearestFilter,
  PerspectiveCamera,
  PlaneGeometry,
  Scene,
  SRGBColorSpace,
  WebGLRenderer,
} from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { PixelDocument } from "@/features/pixel/PixelDocument";

const ALPHA_CUTOFF = 0.1;

interface TexturePreview3DProps {
  doc: PixelDocument;
  /** A block is shown as a cube; anything else as a flat sprite. */
  block: boolean;
}

/** The texture being drawn, on a cube or a sprite that turns and follows every stroke. */
export function TexturePreview3D({ doc, block }: TexturePreview3DProps) {
  const host = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = host.current;
    if (!container) return;

    const renderer = new WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(container.clientWidth, container.clientHeight);
    container.appendChild(renderer.domElement);

    const texture = new CanvasTexture(doc.canvas);
    texture.magFilter = NearestFilter;
    texture.minFilter = NearestFilter;
    texture.colorSpace = SRGBColorSpace;
    const frames = doc.height > doc.width && doc.height % doc.width === 0 ? doc.height / doc.width : 1;
    if (frames > 1) {
      texture.repeat.set(1, 1 / frames);
      texture.offset.set(0, 1 - 1 / frames);
    }

    const scene = new Scene();
    const geometry = block ? new BoxGeometry(1, 1, 1) : new PlaneGeometry(1, 1);
    const material = block
      ? new MeshLambertMaterial({ map: texture, transparent: true, alphaTest: ALPHA_CUTOFF })
      : new MeshBasicMaterial({ map: texture, transparent: true, alphaTest: ALPHA_CUTOFF, side: DoubleSide });
    scene.add(new Mesh(geometry, material));
    scene.add(new AmbientLight(0xffffff, 1.5));
    const sun = new DirectionalLight(0xffffff, 1.4);
    sun.position.set(2, 3, 2);
    scene.add(sun);

    const camera = new PerspectiveCamera(40, container.clientWidth / container.clientHeight, 0.1, 20);
    camera.position.set(1.7, 1.3, 2.1);
    camera.lookAt(0, 0, 0);

    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableZoom = false;
    controls.enablePan = false;
    controls.autoRotate = true;
    controls.autoRotateSpeed = 3;
    controls.enableDamping = true;

    const refresh = () => {
      texture.needsUpdate = true;
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
      observer.disconnect();
      unsubscribe();
      renderer.setAnimationLoop(null);
      controls.dispose();
      geometry.dispose();
      material.dispose();
      texture.dispose();
      renderer.dispose();
      renderer.domElement.remove();
    };
  }, [doc, block]);

  return <div ref={host} className="absolute inset-0 cursor-grab active:cursor-grabbing [&>canvas]:size-full" />;
}
