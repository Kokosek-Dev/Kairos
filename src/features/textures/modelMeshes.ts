import {
  BoxGeometry,
  BufferGeometry,
  CanvasTexture,
  Color,
  DoubleSide,
  Float32BufferAttribute,
  Group,
  InstancedMesh,
  Matrix4,
  Mesh,
  MeshLambertMaterial,
  NearestFilter,
  SRGBColorSpace,
  Vector3,
} from "three";
import type { BlockModel, ModelElement, ModelFace } from "@/lib/tauri/texturepacks";

const ALPHA_CUTOFF = 0.1;
const BIOME_GREEN = 0x79c05a;
const BLOCK = 16;
const ITEM_THICKNESS = 1.5 / 16;
const MAX_ITEM_PIXELS = 128 * 128;

type Corners = [Vector3, Vector3, Vector3, Vector3];
type Uv = [number, number, number, number];

const AXES: Record<string, Vector3> = { x: new Vector3(1, 0, 0), y: new Vector3(0, 1, 0), z: new Vector3(0, 0, 1) };

/** The four corners of a face as seen from outside the block: top left, top right, bottom right, bottom left. */
function cornersOf(direction: string, [x0, y0, z0]: number[], [x1, y1, z1]: number[]): Corners {
  const corner = (x: number, y: number, z: number) => new Vector3(x, y, z);
  switch (direction) {
    case "up":
      return [corner(x0, y1, z0), corner(x1, y1, z0), corner(x1, y1, z1), corner(x0, y1, z1)];
    case "down":
      return [corner(x0, y0, z1), corner(x1, y0, z1), corner(x1, y0, z0), corner(x0, y0, z0)];
    case "north":
      return [corner(x1, y1, z0), corner(x0, y1, z0), corner(x0, y0, z0), corner(x1, y0, z0)];
    case "south":
      return [corner(x0, y1, z1), corner(x1, y1, z1), corner(x1, y0, z1), corner(x0, y0, z1)];
    case "west":
      return [corner(x0, y1, z0), corner(x0, y1, z1), corner(x0, y0, z1), corner(x0, y0, z0)];
    default:
      return [corner(x1, y1, z1), corner(x1, y1, z0), corner(x1, y0, z0), corner(x1, y0, z1)];
  }
}

/** The part of the texture a face shows when the model does not say, which is the part that lines up with the block's size. */
function defaultUv(direction: string, [x0, y0, z0]: number[], [x1, y1, z1]: number[]): Uv {
  switch (direction) {
    case "up":
      return [x0, z0, x1, z1];
    case "down":
      return [x0, BLOCK - z1, x1, BLOCK - z0];
    case "north":
      return [BLOCK - x1, BLOCK - y1, BLOCK - x0, BLOCK - y0];
    case "south":
      return [x0, BLOCK - y1, x1, BLOCK - y0];
    case "west":
      return [z0, BLOCK - y1, z1, BLOCK - y0];
    default:
      return [BLOCK - z1, BLOCK - y1, BLOCK - z0, BLOCK - y0];
  }
}

function faceGeometry(face: ModelFace, element: ModelElement): BufferGeometry {
  const corners = cornersOf(face.direction, element.from, element.to);
  if (element.rotation) {
    const { origin, axis, angle } = element.rotation;
    const center = new Vector3(...origin);
    const turn = new Matrix4().makeRotationAxis(AXES[axis] ?? AXES.y, (angle * Math.PI) / 180);
    for (const point of corners) point.sub(center).applyMatrix4(turn).add(center);
  }

  const [u0, v0, u1, v1] = face.uv ?? defaultUv(face.direction, element.from, element.to);
  const uvs: [number, number][] = [[u0, v0], [u1, v0], [u1, v1], [u0, v1]];
  const steps = (((face.rotation / 90) % 4) + 4) % 4;

  const positions = corners.flatMap((point) => [(point.x - BLOCK / 2) / BLOCK, (point.y - BLOCK / 2) / BLOCK, (point.z - BLOCK / 2) / BLOCK]);
  const coordinates = corners.flatMap((_, index) => {
    const [u, v] = uvs[(index - steps + 4) % 4];
    return [u / BLOCK, 1 - v / BLOCK];
  });
  const geometry = new BufferGeometry();
  geometry.setAttribute("position", new Float32BufferAttribute(positions, 3));
  geometry.setAttribute("uv", new Float32BufferAttribute(coordinates, 2));
  geometry.setIndex([0, 3, 2, 0, 2, 1]);
  geometry.computeVertexNormals();
  return geometry;
}

export interface Disposables {
  dispose: () => void;
}

/** Builds the shape of a model out of one flat face per side, each wearing the texture the model names for it. */
export function buildModel(model: BlockModel, textureFor: (name: string) => CanvasTexture): { group: Group; disposables: Disposables } {
  const group = new Group();
  const geometries: BufferGeometry[] = [];
  const materials: MeshLambertMaterial[] = [];
  for (const element of model.elements) {
    for (const face of element.faces) {
      const geometry = faceGeometry(face, element);
      const material = new MeshLambertMaterial({ map: textureFor(face.texture), transparent: true, alphaTest: ALPHA_CUTOFF, side: DoubleSide });
      if (face.tinted) material.color = new Color(BIOME_GREEN);
      group.add(new Mesh(geometry, material));
      geometries.push(geometry);
      materials.push(material);
    }
  }
  return {
    group,
    disposables: {
      dispose: () => {
        geometries.forEach((geometry) => geometry.dispose());
        materials.forEach((material) => material.dispose());
      },
    },
  };
}

/** A whole block that wears one texture on every side, for blocks the game has no model of. */
export function cubeOf(texture: string): BlockModel {
  const directions = ["down", "up", "north", "south", "west", "east"];
  return {
    elements: [
      { from: [0, 0, 0], to: [16, 16, 16], rotation: null, faces: directions.map((direction) => ({ direction, texture, uv: null, rotation: 0, tinted: false })) },
    ],
  };
}

/** A single upright square showing the whole picture, for pictures that are not drawn on a shape. */
export function squareOf(texture: string): BlockModel {
  const faces = ["north", "south"].map((direction) => ({ direction, texture, uv: null, rotation: 0, tinted: false }));
  return { elements: [{ from: [0, 0, 8], to: [16, 16, 8], rotation: null, faces }] };
}

/** How many square pictures are stacked in a texture, as in an animated one. */
export function frameCount(width: number, height: number): number {
  return height > width && height % width === 0 ? height / width : 1;
}

export function canvasTexture(source: HTMLCanvasElement): CanvasTexture {
  const texture = new CanvasTexture(source);
  texture.magFilter = NearestFilter;
  texture.minFilter = NearestFilter;
  texture.colorSpace = SRGBColorSpace;
  showFirstFrame(texture, source.width, source.height);
  return texture;
}

/** Of an animated texture, only the top picture is shown. */
export function showFirstFrame(texture: CanvasTexture, width: number, height: number) {
  const frames = frameCount(width, height);
  texture.repeat.set(1, 1 / frames);
  texture.offset.set(0, 1 - 1 / frames);
}

/** An item the way the game draws it held: the flat picture given thickness, one little block per visible pixel. */
export function buildItem(source: HTMLCanvasElement): { group: Group; update: () => void; disposables: Disposables } | null {
  const size = source.width;
  const rows = frameCount(source.width, source.height) > 1 ? source.width : source.height;
  const context = source.getContext("2d");
  if (!context || size === 0 || size * rows > MAX_ITEM_PIXELS) return null;

  const geometry = new BoxGeometry(1 / size, 1 / size, ITEM_THICKNESS);
  const material = new MeshLambertMaterial({ transparent: true, alphaTest: ALPHA_CUTOFF });
  const mesh = new InstancedMesh(geometry, material, size * rows);
  const place = new Matrix4();
  const color = new Color();

  const update = () => {
    const { data } = context.getImageData(0, 0, size, rows);
    let count = 0;
    for (let y = 0; y < rows; y++) {
      for (let x = 0; x < size; x++) {
        const at = (y * size + x) * 4;
        if (data[at + 3] < ALPHA_CUTOFF * 255) continue;
        place.makeTranslation((x + 0.5) / size - 0.5, 0.5 - (y + 0.5) / size, 0);
        mesh.setMatrixAt(count, place);
        mesh.setColorAt(count, color.setRGB(data[at] / 255, data[at + 1] / 255, data[at + 2] / 255, SRGBColorSpace));
        count += 1;
      }
    }
    mesh.count = count;
    mesh.instanceMatrix.needsUpdate = true;
    if (mesh.instanceColor) mesh.instanceColor.needsUpdate = true;
  };
  update();

  const group = new Group();
  group.add(mesh);
  return {
    group,
    update,
    disposables: {
      dispose: () => {
        mesh.dispose();
        geometry.dispose();
        material.dispose();
      },
    },
  };
}
