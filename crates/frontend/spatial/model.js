import * as T from 'three';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';

const cube = new T.BoxGeometry(1, 1, 1);
const tubes = new Map();
const matrix = new T.Matrix4(),
  at = new T.Vector3(),
  size = new T.Vector3(),
  q = new T.Quaternion(),
  euler = new T.Euler();
const up = new T.Vector3(0, 1, 0);
function tube(n) {
  if (!tubes.has(n)) tubes.set(n, new T.CylinderGeometry(1, 1, 1, n));
  return tubes.get(n);
}
// Merge rigid parts by material once. Moving joints keep their own small batch.
export class Batch {
  constructor() {
    this.parts = new Map();
  }
  piece(shape, m, x, y, z, w = 1, h = 1, d = 1, rot, meters = false) {
    const g = shape.index ? shape.toNonIndexed() : shape.clone();
    if (meters && m.userData.tile) {
      const uv = g.attributes.uv,
        n = g.attributes.normal,
        tile = m.userData.tile;
      for (let i = 0; i < uv.count; i++) {
        const side = Math.abs(n.getX(i)) > 0.6,
          top = Math.abs(n.getY(i)) > 0.6;
        uv.setXY(i, (uv.getX(i) * (side ? d : w)) / tile, (uv.getY(i) * (top ? d : h)) / tile);
      }
    }
    if (Array.isArray(rot)) q.setFromEuler(euler.set(...rot));
    else if (rot) q.copy(rot);
    else q.identity();
    g.applyMatrix4(matrix.compose(at.set(x, y, z), q, size.set(w, h, d)));
    if (!this.parts.has(m)) this.parts.set(m, []);
    this.parts.get(m).push(g);
  }
  box(m, x, y, z, w, h, d, rot) {
    this.piece(cube, m, x, y, z, w, h, d, rot, true);
  }
  bevel(m, x, y, z, w, h, d, r = 0.025, rot) {
    const g = new RoundedBoxGeometry(w, h, d, 1, Math.min(r, w / 3, h / 3, d / 3));
    this.piece(g, m, x, y, z, 1, 1, 1, rot);
    g.dispose();
  }
  cyl(m, x, y, z, r, h, n = 12, rot) {
    this.piece(tube(n), m, x, y, z, r, h, r, rot);
  }
  beam(m, a, b, r, n = 8) {
    const v = new T.Vector3(b[0] - a[0], b[1] - a[1], b[2] - a[2]);
    const len = v.length();
    if (!len) return;
    const orientation = new T.Quaternion().setFromUnitVectors(up, v.divideScalar(len));
    this.piece(tube(n), m, (a[0] + b[0]) / 2, (a[1] + b[1]) / 2, (a[2] + b[2]) / 2, r, len, r, orientation);
  }
  make(parent, shadow = true) {
    const group = new T.Group();
    for (const [mat, parts] of this.parts) {
      const g = mergeGeometries(parts);
      g.computeBoundingSphere();
      const mesh = new T.Mesh(g, mat);
      mesh.castShadow = shadow;
      mesh.receiveShadow = shadow;
      group.add(mesh);
      for (const p of parts) p.dispose();
    }
    this.parts.clear();
    parent?.add(group);
    return group;
  }
}
export function surface(color, roughness = 0.72, metalness = 0.16, params = {}) {
  return new T.MeshStandardMaterial({ color, roughness, metalness, ...params });
}
export function canvasTexture(w, h, paint) {
  const canvas = document.createElement('canvas');
  canvas.width = w;
  canvas.height = h;
  paint(canvas.getContext('2d'), w, h);
  const map = new T.CanvasTexture(canvas);
  map.colorSpace = T.SRGBColorSpace;
  map.anisotropy = 4;
  return map;
}
export function disposeScene(root) {
  const geo = new Set(),
    mats = new Set(),
    maps = new Set();
  root.traverse((node) => {
    if (node.isInstancedMesh) node.dispose();
    if (node.geometry) geo.add(node.geometry);
    if (node.material)
      for (const m of [].concat(node.material)) {
        mats.add(m);
        for (const value of Object.values(m)) if (value?.isTexture) maps.add(value);
      }
  });
  for (const g of geo) g.dispose();
  for (const m of mats) m.dispose();
  for (const t of maps) t.dispose();
}
