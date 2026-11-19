import * as T from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { HDRLoader } from 'three/addons/loaders/HDRLoader.js';
import { Batch, surface, canvasTexture } from './model.js';
import cargo from '../src/pages/freight/deck.json';

export { cargo };
const colors = [0xa54c35, 0x486c7d, 0xc9b58c, 0x507064, 0xcca04e];
const markings = ['NORTHLINE', 'HAVN', 'MORROW', 'PELAGIC', 'MERIDIAN'];
const v = (x, y, z) => [x, y, z];

async function imageTexture(url, signal, data = false) {
  const response = await fetch(url, { signal });
  if (!response.ok) throw Error('Missing ' + url.pathname);
  const blob = await response.blob();
  // Keep the decoded image available for WebGL context restoration.
  const img = new Image(),
    src = URL.createObjectURL(blob);
  try {
    img.src = src;
    await img.decode();
  } finally {
    URL.revokeObjectURL(src);
  }
  const tex = new T.Texture(img);
  tex.colorSpace = data ? T.NoColorSpace : T.SRGBColorSpace;
  tex.needsUpdate = true;
  return tex;
}
async function payload(url, signal) {
  const response = await fetch(url, { signal });
  if (!response.ok) throw Error('Missing ' + url.pathname);
  return response.arrayBuffer();
}
export async function shipAssets(base, signal) {
  const names = ['sky', 'deck-color', 'deck-normal', 'paint-color', 'paint-normal', 'sea-normal'];
  const loader = new GLTFLoader(),
    model = async (name) =>
      (await loader.parseAsync(await payload(new URL(name + '.glb', base), signal), base.href)).scene;
  const [data, radiance, crew, crate, fender] = await Promise.all([
    Promise.all(names.map((n) => imageTexture(new URL(n + '.webp', base), signal, n.endsWith('normal')))),
    payload(new URL('daylight.hdr', base), signal),
    model('crew'),
    model('crate'),
    model('fender')
  ]);
  const assets = Object.fromEntries(names.map((n, i) => [n, data[i]]));
  assets.sky.mapping = T.EquirectangularReflectionMapping;
  for (const key of names.slice(1)) {
    const t = assets[key];
    t.wrapS = t.wrapT = T.RepeatWrapping;
    t.anisotropy = 4;
  }
  const parsed = new HDRLoader().parse(radiance);
  const hdr = new T.DataTexture(parsed.data, parsed.width, parsed.height, T.RGBAFormat, parsed.type);
  hdr.mapping = T.EquirectangularReflectionMapping;
  hdr.colorSpace = T.LinearSRGBColorSpace;
  hdr.minFilter = hdr.magFilter = T.LinearFilter;
  hdr.flipY = true;
  hdr.needsUpdate = true;
  return { ...assets, hdr, crew, crate, fender };
}
function normalizedProp(root) {
  root.updateMatrixWorld(true);
  let part;
  root.traverse((n) => {
    if (n.isMesh) part = n;
  });
  const geometry = part.geometry.clone().applyMatrix4(part.matrixWorld);
  geometry.computeBoundingBox();
  const size = geometry.boundingBox.getSize(new T.Vector3()),
    center = geometry.boundingBox.getCenter(new T.Vector3());
  geometry.translate(-center.x, -center.y, -center.z);
  geometry.scale(1 / size.x, 1 / size.y, 1 / size.z);
  return { geometry, material: part.material };
}
function decals(batch) {
  const atlas = canvasTexture(2048, 1024, (c) => {
    c.clearRect(0, 0, 2048, 1024);
    c.textBaseline = 'middle';
    function row(i, label, small) {
      const y = i * 128;
      c.fillStyle = '#efe8cf';
      c.font = '900 80px Arial';
      c.fillText(label, 16, y + 58);
      c.font = 'bold 16px monospace';
      c.fillText(small, 20, y + 110);
      for (let p = 0; p < 8; p++) c.fillRect(785 + p * 13, y + 22, 5, 13 + (p % 3) * 7);
    }
    markings.forEach((name, i) => row(i, name, 'INTERMODAL  /  GLOBAL FREIGHT'));
    row(5, 'HNL 047', 'IMO 8917420   •   NORTHLINE MARINE');
    row(6, 'FORE  /  02', 'NO SMOKING   •   CREW ONLY');
    row(7, 'AFT  /  01', 'KEEP CLEAR   •   WATCH YOUR STEP');
    for (let i = 0; i < 8; i++) {
      const y = i * 128;
      c.fillStyle = '#eee8d9';
      c.font = 'bold 29px monospace';
      c.fillText('NLLU  ' + (583291 + i * 1238), 1040, y + 26);
      c.font = 'bold 17px monospace';
      [
        'MAX. GROSS     30 480 KG',
        'TARE           2 260 KG',
        'NET            28 220 KG',
        'CU. CAP.       33.2 CU.M.'
      ].forEach((s, j) => c.fillText(s, 1040, y + 52 + j * 19));
      c.fillStyle = '#f0b83d';
      c.fillRect(1664, y + 14, 320, 83);
      c.fillStyle = '#262c2b';
      c.font = '900 32px sans-serif';
      c.fillText('CAUTION', 1680, y + 43);
      c.font = 'bold 16px monospace';
      c.fillText('HIGH CUBE  /  9 FT 6 IN', 1680, y + 75);
    }
  });
  const mat = new T.MeshBasicMaterial({
    map: atlas,
    transparent: true,
    depthWrite: false,
    polygonOffset: true,
    polygonOffsetFactor: -2
  });
  const plane = new T.PlaneGeometry(1, 1);
  return (x, y, z, w, h, ry, row, part = 0) => {
    const g = plane.clone(),
      uv = g.attributes.uv;
    const start = [0, 0.505, 0.81][part],
      end = [0.49, 0.8, 0.985][part];
    for (let i = 0; i < uv.count; i++)
      uv.setXY(i, start + uv.getX(i) * (end - start), 1 - (row + 1 - uv.getY(i)) / 8);
    batch.piece(g, mat, x, y, z, w, h, 1, [0, ry, 0]);
    g.dispose();
  };
}
function ribPanel(length, height) {
  // An actual trapezoidal steel profile makes highlights and dents read at grazing angles.
  const positions = [],
    normals = [],
    uvs = [];
  const count = Math.round(length / 0.3),
    period = length / count;
  for (let i = 0; i < count; i++) {
    const xs = [0, 0.16, 0.32, 0.75, 1],
      zs = [0, 0, 0.052, 0.052, 0];
    for (let j = 0; j < 4; j++) {
      const a = -length / 2 + (i + xs[j]) * period,
        b = -length / 2 + (i + xs[j + 1]) * period;
      const za = zs[j],
        zb = zs[j + 1],
        n = new T.Vector3(za - zb, 0, b - a).normalize();
      for (const [u, y, z] of [
        [a, 0, za],
        [b, 0, zb],
        [b, height, zb],
        [a, 0, za],
        [b, height, zb],
        [a, height, za]
      ]) {
        positions.push(u, y, z);
        normals.push(n.x, 0, n.z);
        uvs.push((u + length / 2) / 3, y / 3);
      }
    }
  }
  const g = new T.BufferGeometry();
  g.setAttribute('position', new T.Float32BufferAttribute(positions, 3));
  g.setAttribute('normal', new T.Float32BufferAttribute(normals, 3));
  g.setAttribute('uv', new T.Float32BufferAttribute(uvs, 2));
  return g;
}
function boxCargo(b, label, o, i, m) {
  const { x, z, w, d, h, paint } = o,
    y = o.y || 0,
    alongX = w > d,
    L = Math.max(w, d),
    W = Math.min(w, d);
  const body = m.cont[paint],
    rim = m.rim[paint];
  const transform = (lx, ly, lz) => (alongX ? v(x + lx, y + ly, z + lz) : v(x + lz, y + ly, z - lx));
  const rot = alongX ? 0 : Math.PI / 2;
  const box = (mat, lx, ly, lz, sx, sy, sz) => b.box(mat, ...transform(lx, ly, lz), sx, sy, sz, [0, rot, 0]);
  box(body, 0, h / 2, 0, L, h, W);
  const rib = ribPanel(L - 0.2, h - 0.23);
  for (const sign of [-1, 1]) {
    b.piece(rib, body, ...transform(0, 0.1, sign * (W / 2 + 0.002)), 1, 1, 1, [
      0,
      rot + (sign < 0 ? Math.PI : 0),
      0
    ]);
    box(rim, 0, 0.065, (sign * W) / 2, L, 0.13, 0.125);
    box(rim, 0, h - 0.06, (sign * W) / 2, L, 0.12, 0.13);
    for (const s of [-1, 1]) box(rim, s * (L / 2 - 0.065), h / 2, sign * (W / 2 + 0.02), 0.15, h, 0.125);
    label(
      ...transform(sign * L * 0.15, h * 0.62, sign * (W / 2 + 0.06)),
      Math.min(3.4, L * 0.48),
      0.7,
      rot + (sign < 0 ? Math.PI : 0),
      paint
    );
  }
  rib.dispose();
  for (let k = 0.24; k < L - 0.1; k += 0.48) box(rim, k - L / 2, h + 0.014, 0, 0.045, 0.027, W - 0.15);
  for (const sign of [-1, 1]) {
    const face = sign * (L / 2 + 0.016);
    box(rim, face, h * 0.5, 0, 0.042, h - 0.22, 0.045);
    for (const line of [-0.39, -0.13, 0.13, 0.39]) {
      const offset = W * line;
      b.beam(
        m.zinc,
        transform(face + sign * 0.036, 0.12, offset),
        transform(face + sign * 0.036, h - 0.12, offset),
        0.019,
        6
      );
      for (const t of [0.27, 0.75]) box(m.hinge, face + sign * 0.04, h * t, offset, 0.067, 0.09, 0.11);
      box(m.zinc, face + sign * 0.065, h * 0.31, offset + 0.085, 0.035, 0.033, 0.23);
    }
    label(
      ...transform(face + sign * 0.057, h * 0.69, sign * 0.49),
      0.79,
      0.38,
      rot + (sign * Math.PI) / 2,
      i % 8,
      1
    );
    label(
      ...transform(face + sign * 0.061, h * 0.45, -sign * 0.42),
      0.42,
      0.19,
      rot + (sign * Math.PI) / 2,
      0,
      2
    );
  }
}
function ramp(b, o, m) {
  const { x, z, w, h, d } = o,
    sign = o.paint === 1 ? 1 : -1;
  if (o.kind === 'ramp') {
    const wedge = new T.Shape();
    wedge.moveTo(-d / 2, 0);
    wedge.lineTo(d / 2, 0);
    wedge.lineTo((sign * d) / 2, h);
    wedge.closePath();
    const side = new T.ExtrudeGeometry(wedge, { depth: w - 0.09, bevelEnabled: false });
    b.piece(side, m.grate, x + (w - 0.09) / 2, 0, z, 1, 1, 1, [0, -Math.PI / 2, 0]);
    side.dispose();
    const slant = Math.atan2(h, d);
    b.box(m.grate, x, h * 0.5 - 0.07, z, w, 0.14, Math.hypot(d, h), [-sign * slant, 0, 0]);
    for (let i = 1; i < 18; i++) {
      const t = i / 18;
      b.box(m.zinc, x, h * (sign > 0 ? t : 1 - t) + 0.017, z + (t - 0.5) * d, w - 0.1, 0.034, 0.06);
    }
    for (const s of [-1, 1])
      b.beam(
        m.yellow,
        v(x + s * w * 0.5, sign > 0 ? 0.09 : h, z - d / 2),
        v(x + s * w * 0.5, sign > 0 ? h : 0.09, z + d / 2),
        0.065
      );
  } else {
    b.box(m.grate, x, h * 0.5, z, w - 0.11, h - 0.12, d - 0.1);
    b.box(m.grate, x, h - 0.08, z, w, 0.16, d);
    for (const a of [-1, 1])
      for (const s of [-1, 1])
        b.box(m.hinge, x + s * (w / 2 - 0.11), h * 0.5, z + a * (d / 2 - 0.12), 0.14, h, 0.14);
    b.box(m.yellow, x, h - 0.18, z + d / 2, w, 0.21, 0.09);
  }
}
function ocean(scene, a) {
  const mat = new T.ShaderMaterial({
    uniforms: { waterTime: { value: 0 }, waves: { value: a['sea-normal'] }, panorama: { value: a.sky } },
    vertexShader: `varying vec3 pos;
      void main(){vec4 p=modelMatrix*vec4(position,1.);pos=p.xyz;gl_Position=projectionMatrix*viewMatrix*p;}`,
    fragmentShader: `uniform float waterTime;uniform sampler2D waves,panorama;varying vec3 pos;
      vec3 skyAt(vec3 r, float blur) {
        float angle=atan(r.z,r.x)+.96;
        vec2 uv=vec2(fract(angle*.15915494+.5),asin(clamp(r.y,-.99,.99))*.31830989+.5);
        return textureLod(panorama,uv,blur).rgb;
      }
      void main(){
        vec2 p=pos.xz;float t=waterTime;float dist=distance(cameraPosition,pos);
        vec2 a=texture2D(waves,p*.037+vec2(t*.004,t*.0022)).rg*2.-1.;
        vec2 b=texture2D(waves,vec2(-p.y,p.x)*.107+vec2(t*.003,-t*.005)).rg*2.-1.;
        vec2 d=a*.51+b*.24*(1.-smoothstep(35.,160.,dist));
        vec3 n=normalize(vec3(d.x,1.,d.y)), v=normalize(cameraPosition-pos),r=reflect(-v,n);
        float fres=.055+.80*pow(1.-max(dot(n,v),0.),4.);
        vec3 sky=skyAt(vec3(r.x,abs(r.y)*.8+.02,r.z),3.);
        vec3 water=mix(vec3(.021,.085,.097)*(1.+d.y*.45),sky,fres);
        vec3 halfWay=normalize(v+normalize(vec3(-31.,52.,23.)));
        water+=vec3(.94,.82,.61)*.26*pow(max(0.,dot(n,halfWay)),110.);
        float shore=1.-exp(-dist*.0038);
        vec3 haze=skyAt(vec3(-v.x,.018,-v.z),5.);
        gl_FragColor=vec4(mix(water,haze,shore*.84),1.);
        #include <tonemapping_fragment>
        #include <colorspace_fragment>
      }`
  });
  const mesh = new T.Mesh(new T.PlaneGeometry(1850, 1850), mat);
  mesh.rotation.x = -Math.PI / 2;
  mesh.position.y = -4.4;
  scene.add(mesh);
  return mat.uniforms.waterTime;
}
function coastline(harbor, m) {
  // A continuous silhouette meets the water. Two staggered ridges avoid a repeating horizon.
  for (let side = 0; side < 2; side++) {
    const p = [],
      uv = [],
      c = [],
      segments = 110;
    const point = (i, band) => {
      const x = -450 + i * 8.2,
        ridge =
          4.6 * Math.sin(i * 0.17 + side) + 2.9 * Math.sin(i * 0.38 + side * 3) + 1.3 * Math.sin(i * 0.83);
      const z = -192 - side * 92 - band * 10 + 6 * Math.sin(i * 0.2);
      return [x, band === 0 ? -5.3 : band === 1 ? -3 + Math.max(0, ridge * 0.35) : 2 + side * 4 + ridge, z];
    };
    for (let i = 0; i < segments; i++)
      for (let band = 0; band < 2; band++)
        for (const [j, k] of [
          [i, band],
          [i + 1, band],
          [i + 1, band + 1],
          [i, band],
          [i + 1, band + 1],
          [i, band + 1]
        ]) {
          p.push(...point(j, k));
          uv.push(j * 0.2, k);
          const hue = 0.07 * Math.sin(j * 0.7) + 0.03 * Math.cos(j);
          c.push(0.24 + hue, 0.31 + hue, 0.29 + hue);
        }
    const g = new T.BufferGeometry();
    g.setAttribute('position', new T.Float32BufferAttribute(p, 3));
    g.setAttribute('uv', new T.Float32BufferAttribute(uv, 2));
    g.setAttribute('color', new T.Float32BufferAttribute(c, 3));
    g.computeVertexNormals();
    const land = surface(0xb5c2b9, 0.97, 0.0, { side: T.DoubleSide, vertexColors: true });
    harbor.piece(g, land, 0, 0, 0);
    g.dispose();
  }
  const concrete = surface(0x65716f, 0.88, 0.1),
    wall = surface(0x514f47, 0.9, 0.12),
    crane = surface(0xa66544, 0.74, 0.36),
    boom = surface(0x817e6c, 0.7, 0.28);
  harbor.box(concrete, -149, -2.6, -15, 63, 6.0, 236);
  for (let z = -130; z < 101; z += 3.1) {
    harbor.box(wall, -117, -2.0, z, 0.36, 5.3, 2.8);
    if (Math.round(z) % 3 === 0) harbor.box(m.rubber, -116.65, -1.2, z, 0.67, 2.7, 0.75);
  }
  for (let i = 0; i < 38; i++) {
    const z = -118 + (i % 13) * 15,
      x = -130 - Math.floor(i / 13) * 7;
    const h = 3 + (i % 4) * 1.4;
    harbor.box(m.cont[i % 5], x, h * 0.5 + 0.5, z, 4.5, h, 10);
    harbor.box(m.rim[i % 5], x, h + 0.55, z, 4.6, 0.2, 10.1);
  }
  for (let i = 0; i < 4; i++) {
    const x = -125 - i * 0.6,
      z = -95 + i * 49;
    for (const dz of [-5, 5]) {
      for (const dx of [-5, 7]) {
        harbor.beam(crane, v(x + dx, 1, z + dz), v(x + dx * 0.57, 25, z + dz), 0.57, 6);
        for (let y = 4; y < 23; y += 4)
          harbor.beam(boom, v(x + dx * 0.7, y, z + dz), v(x - dx * 0.5, y + 4, z + dz), 0.12, 5);
      }
      harbor.beam(crane, v(x - 5, 24, z + dz), v(x + 28, 24, z + dz), 0.44, 6);
      harbor.beam(crane, v(x - 5, 26, z + dz), v(x + 28, 26, z + dz), 0.29, 6);
      for (let dx = -5; dx < 27; dx += 3)
        harbor.beam(boom, v(x + dx, 24, z + dz), v(x + dx + 2.9, 26, z + dz), 0.13, 5);
      harbor.beam(crane, v(x, 34, z + dz), v(x + 19, 26, z + dz), 0.19);
      harbor.beam(crane, v(x - 4, 25, z + dz), v(x, 34, z + dz), 0.29);
      harbor.beam(m.hinge, v(x + 20, 24, z + dz), v(x + 20, 13, z + dz), 0.045);
    }
    harbor.box(boom, x - 4, 25, z, 4.6, 2.7, 11);
    harbor.box(m.red, x + 20, 13, z, 2, 0.65, 10.3);
  }
  // Loaded coastal freighter riding at anchor on the starboard horizon.
  const x = 94,
    z = -72,
    shape = new T.Shape();
  [
    [-5, 24],
    [-5, -19],
    [-3, -26],
    [0, -30],
    [3, -26],
    [5, -19],
    [5, 24],
    [4, 26],
    [-4, 26]
  ].forEach(([u, v], i) => (i ? shape.lineTo(u, -v) : shape.moveTo(u, -v)));
  shape.closePath();
  const hull = new T.ExtrudeGeometry(shape, { depth: 3.9, bevelEnabled: false });
  harbor.piece(hull, m.navy, x, -4.5, z, 1, 1, 1, [-Math.PI / 2, 0, 0]);
  hull.dispose();
  for (let i = 0; i < 15; i++) {
    const px = x + ((i % 3) - 1) * 2.52,
      pz = z - 14 + Math.floor(i / 3) * 6.6;
    harbor.box(m.cont[(i * 7) % 5], px, 0.72, pz, 2.35, 2.65, 6.1);
    if (i % 4) harbor.box(m.cont[(i + 2) % 5], px, 3.31, pz, 2.35, 2.5, 6.1);
  }
  harbor.box(m.white, x, 2.2, z + 19, 9, 5.4, 4.5);
  harbor.box(m.glass, x, 4.25, z + 16.7, 8.4, 0.75, 0.16);
  harbor.box(m.red, x - 2.1, 5.9, z + 20.1, 1.5, 2.7, 1.5);
}
export function buildShip(scene, a) {
  const b = new Batch(),
    ink = new Batch(),
    harbor = new Batch();
  const label = decals(ink);
  const mat = (color, rough, metal, extra) => surface(color, rough, metal, extra);
  const painted = (c) =>
    mat(c, 0.67, 0.36, {
      map: a['paint-color'],
      normalMap: a['paint-normal'],
      normalScale: new T.Vector2(0.22, 0.22)
    });
  const m = {
    cont: colors.map(painted),
    rim: colors.map((c) => painted(new T.Color(c).multiplyScalar(0.77))),
    deck: mat(0xcbd1c8, 0.79, 0.36, {
      map: a['deck-color'],
      normalMap: a['deck-normal'],
      normalScale: new T.Vector2(0.59, 0.59)
    }),
    zinc: mat(0x9ca6a1, 0.39, 0.68),
    hinge: painted(0x414b4d),
    grate: painted(0x606b65),
    yellow: painted(0xd4a342),
    white: painted(0xe3ded1),
    navy: painted(0x313e45),
    red: painted(0x984738),
    line: mat(0xd8b25a, 0.88, 0.04),
    stripe: mat(0xd6d2bf, 0.87, 0.04),
    rubber: mat(0x252b2c, 0.9, 0.02),
    glass: mat(0x334f5a, 0.19, 0.64)
  };
  for (const v of [...m.cont, ...m.rim, m.hinge, m.grate, m.yellow, m.white, m.navy, m.red])
    v.userData.tile = 2.9;
  m.deck.userData.tile = 4.7;

  const plate = new T.Shape();
  [
    [-15.8, 29],
    [-15.7, -28],
    [-13.0, -36],
    [-8.4, -41],
    [0, -44],
    [8.4, -41],
    [13, -36],
    [15.7, -28],
    [15.8, 29],
    [12.9, 33],
    [-12.9, 33]
  ].forEach(([x, z], i) => (i ? plate.lineTo(x, -z) : plate.moveTo(x, -z)));
  plate.closePath();
  const hull = new T.ExtrudeGeometry(plate, {
    depth: 4.45,
    bevelEnabled: true,
    bevelSegments: 1,
    bevelSize: 0.2,
    bevelThickness: 0.15
  });
  b.piece(hull, m.navy, 0, -4.61, 0, 1, 1, 1, [-Math.PI / 2, 0, 0]);
  hull.dispose();
  b.box(m.deck, 0, -0.1, 0, 30.4, 0.2, 60.8);
  for (let z = -28; z <= 28; z += 4.0) {
    b.box(m.hinge, 0, 0.006, z, 29.9, 0.011, 0.03);
    for (const x of [-12.8, -3.8, 4.1, 13]) b.box(m.line, x, 0.014, z + 1.15, 0.083, 0.014, 1.8);
  }
  for (const x of [-14.2, 14.2]) b.box(m.line, x, 0.015, 0, 0.18, 0.02, 57);
  for (const z of [-24, 24])
    for (let x = -13; x < 14; x += 1.2) b.box(m.line, x, 0.017, z, 0.38, 0.016, 0.85, [0, 0.52, 0]);
  // Welded hatch seams, drainage slots and flush lashing sockets.
  for (let z = -25; z <= 26; z += 5.0)
    for (const x of [-12.9, -0.4, 12.9]) {
      b.box(m.hinge, x, 0.016, z, 0.35, 0.026, 0.17);
      for (const s of [-1, 1]) b.box(m.zinc, x + s * 0.105, 0.033, z, 0.045, 0.016, 0.1);
    }
  const contact = canvasTexture(96, 96, (c, w, h) => {
    const grad = c.createRadialGradient(w / 2, h / 2, 8, w / 2, h / 2, 48);
    grad.addColorStop(0, 'rgba(12,14,12,.78)');
    grad.addColorStop(0.68, 'rgba(15,18,15,.46)');
    grad.addColorStop(1, 'rgba(12,14,12,0)');
    c.fillStyle = grad;
    c.fillRect(0, 0, w, h);
  });
  const shade = new T.MeshBasicMaterial({
    map: contact,
    transparent: true,
    depthWrite: false,
    polygonOffset: true,
    polygonOffsetFactor: -1
  });
  const shadowPlane = new T.PlaneGeometry(1, 1);
  const crate = normalizedProp(a.crate),
    fender = normalizedProp(a.fender);
  cargo.forEach((o, i) => {
    if (!o.y) ink.piece(shadowPlane, shade, o.x, 0.039, o.z, o.w + 1.4, o.d + 1.7, 1, [-Math.PI / 2, 0, 0]);
    if (o.kind === 'container') boxCargo(b, label, o, i, m);
    else if (o.kind === 'crate') {
      const levels = o.h > 1.28 ? 2 : 1,
        h = (o.h - 0.1) / levels;
      for (let level = 0; level < levels; level++)
        b.piece(
          crate.geometry,
          crate.material,
          o.x,
          (o.y || 0) + 0.1 + h * (level + 0.5),
          o.z,
          o.w - 0.02,
          h - 0.012,
          o.d
        );
      for (const side of [-1, 1])
        b.box(m.rim[2], o.x + side * o.w * 0.36, (o.y || 0) + 0.05, o.z, 0.14, 0.1, o.d);
    } else ramp(b, o, m);
  });
  shadowPlane.dispose();
  crate.geometry.dispose();
  for (const side of [-1, 1])
    for (const z of [-21, -7, 8, 23]) {
      b.piece(fender.geometry, fender.material, side * 15.92, -1.36, z, 1.46, 0.46, 1.46, [
        0,
        0,
        Math.PI / 2
      ]);
      for (const dz of [-0.3, 0.3])
        b.beam(m.hinge, v(side * 15.86, -0.9, z + dz), v(side * 15.5, 0.17, z + dz * 1.5), 0.028, 6);
    }
  fender.geometry.dispose();
  function rail(x1, z1, x2, z2) {
    const len = Math.hypot(x2 - x1, z2 - z1),
      n = Math.ceil(len / 2);
    for (let i = 0; i <= n; i++) {
      const t = i / n;
      b.cyl(m.white, x1 + (x2 - x1) * t, 0.55, z1 + (z2 - z1) * t, 0.045, 1.1, 6);
    }
    for (const y of [0.39, 0.72, 1.08])
      b.beam(y > 1 ? m.yellow : m.white, v(x1, y, z1), v(x2, y, z2), y > 1 ? 0.052 : 0.026, 8);
    b.beam(m.red, v(x1, 0.12, z1), v(x2, 0.12, z2), 0.12);
  }
  for (const s of [-1, 1]) {
    rail(s * 14.88, -29, s * 14.88, 28.95);
    rail(-14.88, s * 28.95, 14.88, s * 28.95);
  }
  for (const x of [-13.5, 13.5])
    for (const z of [-26, 24]) {
      b.box(m.grate, x, 0.12, z, 1.05, 0.24, 1.6);
      for (const dz of [-0.42, 0.42]) {
        b.cyl(m.navy, x, 0.5, z + dz, 0.23, 0.64);
        b.cyl(m.zinc, x, 0.83, z + dz, 0.29, 0.11);
      }
    }
  // Bridge, wheelhouse glazing, walkways, masts and ship exhaust.
  b.box(m.white, 0, 2.12, -33.7, 17.6, 4.28, 6.8);
  b.box(m.red, 0, 0.43, -33.7, 17.85, 0.68, 6.95);
  b.box(m.white, 0, 5.0, -34.2, 20.6, 1.75, 5.6);
  b.box(m.navy, 0, 5.55, -31.37, 19.8, 1.17, 0.12);
  for (let i = -7; i <= 7; i++) b.box(m.glass, i * 1.29, 5.55, -31.25, 1.17, 0.89, 0.11, [0.1, 0, 0]);
  for (const s of [-1, 1]) {
    b.box(m.glass, s * 10.32, 5.45, -34.1, 0.085, 0.9, 4.9);
    b.box(m.white, s * 9.65, 4.33, -32.85, 3.8, 0.19, 5.0);
    for (let z = -35; z < -30; z += 1.2) b.cyl(m.white, s * 11.3, 4.83, z, 0.048, 1.0);
    b.beam(m.white, v(s * 11.3, 5.34, -35.3), v(s * 11.3, 5.34, -30.4), 0.05);
    for (let i = 0; i < 4; i++) b.box(m.glass, s * (3.7 + i * 1.23), 2.58, -30.22, 0.75, 0.8, 0.07);
  }
  b.box(m.white, 0, 6.07, -34.2, 21.1, 0.28, 5.9);
  for (const s of [-1, 1]) {
    b.box(m.navy, s * 2.4, 1.47, -30.24, 1.18, 2.49, 0.09);
    b.box(m.glass, s * 2.4, 2.04, -30.15, 0.7, 0.58, 0.06);
  }
  label(0, 3.55, -30.22, 3.05, 0.52, 0, 6);
  b.cyl(m.white, 0, 10.0, -35, 0.15, 8.2);
  b.cyl(m.navy, 0, 13.88, -35, 0.069, 2.8);
  for (const y of [8.2, 10.8, 12.4]) {
    b.beam(m.white, v(-2.1, y, -35), v(2.1, y, -35), 0.085);
    for (const x of [-1.75, 1.75]) b.beam(m.zinc, v(x, y, -35), v(0, y + 2.1, -35), 0.032);
  }
  b.box(m.navy, 0, 9.1, -34.9, 2.85, 0.18, 0.34);
  b.box(m.red, -5.6, 7.65, -37.2, 2.42, 4.15, 2.4, [0.1, 0, 0]);
  b.box(m.navy, -5.6, 9.52, -37.42, 2.48, 0.88, 2.58, [0.1, 0, 0]);
  for (const x of [-1.4, 1.4]) b.beam(m.hinge, v(x, 0, 31.3), v(0.45 * x, 8.7, 31.3), 0.12);
  b.beam(m.yellow, v(-1.2, 8.1, 31.3), v(1.2, 8.1, 31.3), 0.095);
  for (const x of [-14.8, 14.8])
    for (const z of [-15, 14]) {
      const tor = new T.TorusGeometry(0.39, 0.091, 7, 20);
      b.piece(tor, m.red, x, 0.95, z, 1, 1, 1, [0, Math.PI / 2, 0]);
      tor.dispose();
      b.box(m.white, x, 0.95 + 0.39, z, 0.15, 0.14, 0.2);
      b.box(m.white, x, 0.95 - 0.39, z, 0.15, 0.14, 0.2);
    }
  // Two shipboard cargo booms frame the playable deck; their cables remain outside the lanes.
  for (const [x, z, dir] of [
    [-15.4, -19, -1],
    [15.5, 19, 1]
  ]) {
    b.cyl(m.navy, x, 1.15, z, 1.03, 2.3);
    b.cyl(m.yellow, x, 5.2, z, 0.55, 7.8);
    b.cyl(m.hinge, x, 8.5, z, 0.7, 0.48);
    const end = v(x + dir * 4.2, 13.0, z - 7);
    b.beam(m.yellow, v(x, 7.0, z), end, 0.27);
    b.beam(m.zinc, v(x, 10.4, z), end, 0.045);
    b.beam(m.yellow, v(x, 8, z), v(x, 10.5, z), 0.17);
    b.beam(m.rubber, end, v(end[0], 5.5, end[2]), 0.032);
    b.box(m.yellow, end[0], 5.4, end[2], 0.49, 0.52, 0.37);
    b.box(m.white, x - dir * 0.54, 7.48, z, 0.99, 1.48, 1.25);
    b.box(m.glass, x - dir * 1.05, 7.65, z, 0.06, 0.69, 1.0);
  }
  b.make(scene);
  ink.make(scene, false);
  coastline(harbor, m);
  harbor.make(scene, false);
  return ocean(scene, a);
}
