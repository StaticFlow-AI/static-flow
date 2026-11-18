import * as T from 'three';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';

export const TAU = Math.PI * 2;
export function group(parent, name, at = [0, 0, 0], data = {}) {
  const node = new T.Group();
  node.name = name;
  node.position.set(...at);
  node.userData = data;
  parent.add(node);
  return node;
}

const finishes = new Map();
function material(kind) {
  if (!finishes.has(kind)) {
    const mat = new T.MeshStandardMaterial({
      vertexColors: true,
      roughness: kind === 'metal' ? 0.43 : 0.82,
      metalness: kind === 'metal' ? 0.38 : 0,
      side: T.DoubleSide
    });
    mat.name = kind;
    if (kind === 'lamp') {
      mat.emissive.setHex(0xffead1);
      mat.emissiveIntensity = 0.85;
    }
    finishes.set(kind, mat);
  }
  return finishes.get(kind);
}

export function mesh(parent, geo, color, at, kind = 'paint') {
  const rgb = new T.Color(color);
  const colors = new Float32Array(geo.attributes.position.count * 3);
  for (let i = 0; i < colors.length; i += 3) rgb.toArray(colors, i);
  geo.setAttribute('color', new T.BufferAttribute(colors, 3));
  if (!geo.attributes.uv)
    geo.setAttribute(
      'uv',
      new T.Float32BufferAttribute(new Float32Array(geo.attributes.position.count * 2), 2)
    );
  const m = new T.Mesh(geo, material(kind));
  m.position.set(...at);
  parent.add(m);
  return m;
}
export function box(p, at, size, color, kind = 'paint', bevel = 0) {
  const [w, h, d] = size;
  const geo = bevel
    ? new RoundedBoxGeometry(w, h, d, 1, Math.min(bevel, w / 3, h / 3, d / 3))
    : new T.BoxGeometry(w, h, d);
  return mesh(p, geo, color, at, kind);
}
export function oval(p, at, radii, color, kind = 'paint', detail = 16) {
  return mesh(
    p,
    new T.SphereGeometry(1, detail, Math.max(8, Math.floor(detail * 0.65))).scale(...radii),
    color,
    at,
    kind
  );
}
export function cyl(p, at, top, bottom, height, color, kind = 'paint', sides = 16) {
  return mesh(p, new T.CylinderGeometry(top, bottom, height, sides), color, at, kind);
}
export function beam(p, a, b, r1, r2, color, kind = 'metal', sides = 10) {
  const start = new T.Vector3(...a),
    delta = new T.Vector3(...b).sub(start);
  const n = cyl(
    p,
    start.addScaledVector(delta, 0.5).toArray(),
    r2,
    r1,
    delta.length(),
    color,
    kind,
    sides
  );
  n.quaternion.setFromUnitVectors(new T.Vector3(0, 1, 0), delta.normalize());
  return n;
}
export function tube(p, points, radius, color, kind = 'paint', steps = 12, sides = 6) {
  const path = new T.CatmullRomCurve3(points.map((point) => new T.Vector3(...point)));
  return mesh(p, new T.TubeGeometry(path, steps, radius, sides, false), color, [0, 0, 0], kind);
}
export function ring(p, at, radius, wire, color, kind = 'metal', segments = 32) {
  return mesh(p, new T.TorusGeometry(radius, wire, 6, segments), color, at, kind);
}
// Cross-sections are [height, half-width, half-depth, lateral offset, depth offset].
export function loft(p, at, sections, color, { sides = 24, pleats = 0, kind = 'paint' } = {}) {
  const v = [],
    idx = [],
    uv = [];
  for (const [level, [y, w, d, x = 0, z = 0]] of sections.entries()) {
    for (let j = 0; j < sides; j++) {
      const a = (j * TAU) / sides,
        fold = 1 + pleats * Math.cos(10 * a);
      v.push(x + w * Math.sin(a) * fold, y, z + d * Math.cos(a) * fold);
      uv.push(j / sides, level / (sections.length - 1));
      if (level) {
        const n = level * sides + j,
          next = level * sides + ((j + 1) % sides);
        idx.push(n - sides, next - sides, n, next - sides, next, n);
      }
    }
  }
  for (let j = 1; j < sides - 1; j++) {
    const last = (sections.length - 1) * sides;
    idx.push(0, j + 1, j, last, last + j, last + j + 1);
  }
  const g = new T.BufferGeometry();
  g.setAttribute('position', new T.Float32BufferAttribute(v, 3));
  g.setAttribute('uv', new T.Float32BufferAttribute(uv, 2));
  g.setIndex(idx);
  g.computeVertexNormals();
  return mesh(p, g, color, at, kind);
}
export function patch(p, points, color, kind = 'paint') {
  const g = new T.BufferGeometry();
  g.setAttribute('position', new T.Float32BufferAttribute(points.flat(), 3));
  const ind = [];
  for (let i = 1; i + 1 < points.length; i++) ind.push(0, i, i + 1);
  g.setIndex(ind);
  g.computeVertexNormals();
  return mesh(p, g, color, [0, 0, 0], kind);
}
// Batch fixed meshes per material in each semantic node. Hinges, character
// joints and interaction targets remain independently addressable in glTF.
export function bake(node) {
  const buckets = new Map();
  for (const child of [...node.children]) {
    if (!child.isMesh) {
      bake(child);
      continue;
    }
    child.updateMatrix();
    const geo = child.geometry.index ? child.geometry.toNonIndexed() : child.geometry.clone();
    geo.applyMatrix4(child.matrix);
    geo.clearGroups();
    const list = buckets.get(child.material) ?? [];
    list.push(geo);
    buckets.set(child.material, list);
    node.remove(child);
    child.geometry.dispose();
  }
  for (const [mat, parts] of buckets) {
    const g = mergeGeometries(parts);
    node.add(new T.Mesh(g, mat));
    for (const part of parts) part.dispose();
  }
}
export function random(seed) {
  return () => {
    seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
    return seed / 4294967296;
  };
}
