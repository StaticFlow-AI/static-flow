import { BufferAttribute, BufferGeometry, Float32BufferAttribute, Sphere, Vector3 } from 'three';
import { faceTiles, sw } from './atlas.js';

const faces = [
  [
    [1, 0, 1],
    [1, 0, 0],
    [1, 1, 0],
    [1, 1, 1]
  ],
  [
    [0, 0, 0],
    [0, 0, 1],
    [0, 1, 1],
    [0, 1, 0]
  ],
  [
    [0, 1, 1],
    [1, 1, 1],
    [1, 1, 0],
    [0, 1, 0]
  ],
  [
    [0, 0, 0],
    [1, 0, 0],
    [1, 0, 1],
    [0, 0, 1]
  ],
  [
    [0, 0, 1],
    [1, 0, 1],
    [1, 1, 1],
    [0, 1, 1]
  ],
  [
    [1, 0, 0],
    [0, 0, 0],
    [0, 1, 0],
    [1, 1, 0]
  ]
];
const normals = [
  [1, 0, 0],
  [-1, 0, 0],
  [0, 1, 0],
  [0, -1, 0],
  [0, 0, 1],
  [0, 0, -1]
];
const light = [0.78, 0.73, 1, 0.51, 0.92, 0.82];
const corners = [
  [0, 0],
  [1, 0],
  [1, 1],
  [0, 1]
];
const emissionTiles = new Set([
  sw.lit.id,
  sw.amber.id,
  sw.waveDial.id,
  sw.nixieLeft.id,
  sw.nixieRight.id,
  sw.crtFace.id,
  sw.circuit.id,
  ...faceTiles(33),
  ...faceTiles(54)
]);
export class Batch {
  constructor() {
    this.position = [];
    this.normal = [];
    this.color = [];
    this.uv = [];
    this.irradiance = [];
    this.lighting = 0;
    this.triangles = [];
  }
  face(x, y, z, w, h, d, f, tile, ao, level = 1, lighting = this.lighting, mirror = false) {
    const idx = this.position.length / 3,
      u = ((tile % 16) * 32 + 8.12) / 512,
      v = 1 - ((tile >> 4) * 32 + 23.88) / 512,
      size = 15.76 / 512;
    const shape = faces[f];
    let i = 0;
    for (const [a, b, c] of shape) {
      this.position.push(x + w * a, y + h * b, z + d * c);
      this.normal.push(normals[f][0], normals[f][1], normals[f][2]);
      const l = light[f] * level * (ao ? ao[i] : 1);
      this.color.push(l, l * 0.99, l * 0.975);
      const exposure = (lighting >>> (i * 8)) & 255;
      this.irradiance.push((exposure >> 4) / 9, (exposure & 15) / 9, emissionTiles.has(tile) ? 1 : 0);
      this.uv.push(u + (mirror ? 1 - corners[i][0] : corners[i][0]) * size, v + corners[i][1] * size);
      i++;
    }
    if (ao && ao[0] + ao[2] > ao[1] + ao[3])
      this.triangles.push(idx, idx + 1, idx + 3, idx + 1, idx + 2, idx + 3);
    else this.triangles.push(idx, idx + 1, idx + 2, idx, idx + 2, idx + 3);
  }
  box(x, y, z, w, h, d, color) {
    const tiles = typeof color === 'string' ? sw[color].id : color;
    for (let f = 0; f < 6; f++) this.face(x, y, z, w, h, d, f, Array.isArray(tiles) ? tiles[f] : tiles);
  }
  geometry() {
    const g = new BufferGeometry();
    g.setAttribute('position', new Float32BufferAttribute(this.position, 3));
    g.setAttribute('normal', new Float32BufferAttribute(this.normal, 3));
    g.setAttribute('color', new Float32BufferAttribute(this.color, 3));
    g.setAttribute('uv', new Float32BufferAttribute(this.uv, 2));
    g.setIndex(this.triangles);
    g.computeBoundingSphere();
    return g;
  }
}
// Local prop coordinates are in sixteenths of one block, as are the character skins.
function prop(batch, key, x, y, z) {
  const b = (a, b, c, w, h, d, color) =>
    batch.box(x + a / 16, y + b / 16, z + c / 16, w / 16, h / 16, d / 16, color);
  const face = (a, bottom, w, h, color, zfront = 14) => b(a, bottom, zfront, w, h, 0.22, color);
  switch (key) {
    case 'wave':
      b(0, 1, 1, 16, 11, 13, 'ivory');
      b(1, 0, 3, 2, 1, 8, 'ink');
      b(13, 0, 3, 2, 1, 8, 'ink');
      face(0.7, 2, 10.8, 8.7, 'metal');
      face(1.2, 2.6, 9.6, 7.6, 'ink', 14.3);
      face(2, 3.4, 8.2, 5.9, 'ovenGlass', 14.5);
      for (let i = 0; i < 3; i++) {
        face(2.6 + i * 2.1, 4.2, 2.4, 0.9, 'banana', 14.8);
        face(2.3 + i * 2.1, 4.8, 0.9, 1.1, 'lime', 14.83);
      }
      face(11.2, 3, 1, 7, 'silver', 14.7);
      face(13, 8.4, 2.2, 1.3, 'waveDial');
      face(13.4, 4, 1.5, 1.9, 'metal');
      face(13.6, 4.6, 0.5, 0.9, 'white', 14.5);
      for (let i = 0; i < 5; i++) b(2 + i * 1.2, 12, 4, 0.5, 0.2, 4, 'metal');
      break;
    case 'relay':
      b(1, 0, 2, 14, 3, 12, 'metal');
      b(2, 3, 3, 12, 1, 10, 'green');
      for (const a of [4, 10]) {
        b(a, 4, 6, 2, 5, 3, 'cyan');
        b(a, 9, 6, 2, 1, 3, 'lit');
      }
      b(2, 4, 4, 1, 5, 1, 'silver');
      face(3, 1, 8, 1, 'amber', 14.1);
      break;
    case 'crt':
    case 'monitor': {
      const big = key === 'crt';
      b(2, 0, 3, 12, 1.5, 10, 'ink');
      b(6, 1, 5, 4, 2, 6, 'metal');
      b(2, 3, 1.3, 12, 9, 5, 'metal');
      b(0.4, 2.5, 5, 15.2, 11, 9, 'darkblue');
      b(1.2, 3.4, 13.6, 13.6, 9.4, 0.8, 'ink');
      face(2.1, 5, 11.7, 6.6, big ? 'crtFace' : 'circuit', 14.5);
      face(12, 3.9, 1, 0.6, 'lime', 14.8);
      for (let j = 0; j < 4; j++) b(15.4, 5 + j * 1.35, 6, 0.35, 0.4, 4.8, 'ink');
      if (big) {
        b(13, 3.3, 14.55, 1, 0.8, 0.48, 'silver');
        for (let j = 0; j < 4; j++) face(2 + j * 1.25, 3.35, 0.7, 0.4, 'metal', 14.6);
      }
      if (!big) {
        b(1, 0, 13, 13, 0.6, 3, 'silver');
        for (let i = 0; i < 5; i++) b(2 + i * 2, 0.65, 14, 1.2, 0.2, 0.8, 'ink');
      }
      break;
    }
    case 'phone':
      b(5.4, 0.1, 4, 5, 0.8, 8, 'black');
      b(5.7, 1, 4.5, 4.4, 3.9, 0.8, 'ink');
      b(6.1, 1.5, 5.3, 3.6, 2.7, 0.2, 'cyan');
      for (let row = 0; row < 3; row++)
        for (let col = 0; col < 3; col++) b(6 + col * 1.28, 1, 7 + row * 1.25, 1, 0.15, 0.65, 'silver');
      b(6.4, 3.4, 5.54, 2.9, 0.2, 0.1, 'paper');
      break;
    case 'leap':
      b(3, 1, 4, 2.4, 5, 3, 'ink');
      b(10.5, 1, 4, 2.4, 5, 3, 'ink');
      b(4, 5.7, 5, 8, 1, 1.5, 'silver');
      b(2, 0, 3, 3, 3, 4.5, 'darkblue');
      b(11, 0, 3, 3, 3, 4.5, 'darkblue');
      b(12, 0.2, 8, 0.5, 0.4, 5, 'ink');
      break;
    case 'upa':
      b(4, 0.3, 4, 8, 4.5, 7.5, 'silver');
      b(5.4, 4.8, 5, 5.2, 1.1, 5.3, 'ivory');
      b(4, 4.8, 6, 1.3, 1.7, 2, 'silver');
      face(5.4, 2.8, 1, 1, 'ink', 11.5);
      face(9.3, 2.8, 1, 1, 'ink', 11.5);
      face(7.3, 2, 1.5, 0.65, 'ivory', 11.7);
      break;
    case 'sofa':
      b(0, 8, 1, 16, 7.4, 3.7, 'green');
      b(0.3, 15.2, 1.4, 15.4, 0.55, 3, 'sage');
      b(0.5, 0, 1, 15, 2, 12, 'deepwood');
      b(0, 2, 1, 16, 7, 4, 'green');
      b(0.5, 3, 4.5, 15, 4, 9, 'sage');
      b(0, 6.8, 4.5, 1, 0.4, 9, 'green');
      b(0.6, 2, 13.4, 14.8, 3, 1, 'green');
      b(1, 6.85, 12.8, 14, 0.35, 0.55, 'sage');
      b(1.2, 7.1, 5, 13.6, 0.22, 0.45, 'seam');
      for (const x of [4.4, 11.6]) b(x, 11.9, 4.67, 0.65, 0.7, 0.25, 'seam');
      break;
    case 'panda':
      b(3, 0, 3, 10, 8, 5, 'white');
      for (const i of [2, 11]) {
        b(i, 7, 4, 3, 2.2, 3, 'black');
        b(i + 1, 3, 7.9, 2.6, 2.5, 0.4, 'black');
      }
      b(7, 2, 8.1, 2, 1, 0.4, 'ink');
      break;
    case 'table':
      b(0, 13, 0, 16, 3, 16, 'wood');
      b(0.5, 13.2, 15.8, 15, 0.45, 0.28, 'deepwood');
      b(0.4, 13, 0.4, 15.2, 0.55, 15.2, 'deepwood');
      for (const a of [1, 13]) for (const c of [1, 13]) b(a, 0, c, 2, 13, 2, 'deepwood');
      break;
    case 'books':
      for (let j = 0; j < 3; j++) {
        b(1 + j, 1.8 * j, 4 + j * 0.5, 11, 1.7, 7, ['red', 'darkblue', 'green'][j]);
        b(2 + j, 0.3 + j * 1.8, 10.8 + j * 0.5, 9, 1, 0.3, 'paper');
      }
      break;
    case 'curtain':
      for (let i = 0; i < 8; i++) b(2, 0, i * 2, 1 + (i % 2) * 0.5, 16, 2.1, i % 2 ? 'rose' : 'pink');
      break;
    case 'nixie':
      b(0.5, 0, 3, 15, 2.5, 10, 'deepwood');
      b(1, 2.5, 3.5, 14, 1, 9, 'metal');
      for (let i = 0; i < 5; i++) {
        let a = 1.5 + i * 2.7;
        b(a, 3.5, 6, 2.2, 4.5, 2.5, 'tube');
        b(a + 0.3, 4.3, 8.5, 1.4, 2.8, 0.3, 'amber');
        b(a + 0.5, 5.2, 8.8, 0.7, 0.65, 0.15, 'deepwood');
      }
      face(1, 3.8, 7, 3.5, 'nixieLeft', 9.1);
      face(8, 3.8, 7, 3.5, 'nixieRight', 9.1);
      break;
    case 'flower':
    case 'sapling':
    case 'fern':
      b(7, 0, 7, 1.2, 7, 1.2, 'moss');
      b(4, 2, 7, 3, 2, 1.2, 'green');
      b(8, 3, 7, 3, 2, 1.3, 'moss');
      if (key === 'flower') {
        b(5, 6, 6, 5, 2, 3, 'paper');
        b(6.5, 7, 6, 2, 2, 2.5, 'amber');
      } else {
        b(5, 5, 5, 6, 3, 5, 'green');
        b(6, 7, 6, 4, 2.4, 3, 'moss');
      }
      break;
    case 'torch':
      b(7, 0, 7, 2, 8, 2, 'wood');
      b(7, 7, 7, 2, 2, 2, 'orange');
      b(7.25, 9, 7.25, 1.5, 2.2, 1.5, 'lit');
      break;
    case 'awning':
      b(0, 12, 0, 16, 1.4, 1.4, 'metal');
      for (let strip = 0; strip < 4; strip++) {
        const cloth = strip % 2 ? 'paper' : 'teal';
        for (let rung = 0; rung < 4; rung++)
          b(strip * 4, 11.7 - rung * 1.25, rung * 4, 4, 1.2, 4.25, cloth);
        b(strip * 4, 5.3, 14.9, 4, 3, 1.1, cloth);
      }
      b(0.7, 5, 0.2, 0.7, 0.7, 14, 'metal');
      break;
    case 'rail':
      b(1, 0, 1, 1.45, 16, 1.3, 'metal');
      b(1, 0, 13.8, 1.45, 16, 1.3, 'metal');
      b(0.85, 14.3, 0, 1.8, 1.5, 16, 'silver');
      b(1, 6.5, 0, 1.15, 0.9, 16, 'metal');
      break;
    case 'sign':
      b(0, 2, 0, 16, 12, 1, 'deepwood');
      b(0.4, 2.5, 1, 15.2, 11, 0.4, 'cream');
      // Three painted letters on the old Brown Tube Workshop sign.
      ['111100100100111', '110101110101101', '111010010010010'].forEach((glyph, i) => {
        [...glyph].forEach((pixel, j) => {
          if (pixel === '1')
            face(1.5 + i * 4.5 + (j % 3), 9.7 - Math.floor(j / 3) * 1.25, 1, 1.25, 'auburn', 1.55);
        });
      });
      break;
    case 'rug':
      b(0, 0.02, 0, 16, 0.13, 16, 'red');
      b(1, 0.16, 1, 14, 0.1, 14, 'cream');
      b(2, 0.27, 2, 12, 0.1, 12, 'rose');
      break;
    case 'fan':
      b(3, 0, 4, 10, 1.6, 8, 'metal');
      b(7, 1.5, 7, 2, 6, 2, 'silver');
      b(3, 7, 6, 10, 6.4, 3, 'metal');
      b(5, 5.4, 6, 6, 9.5, 3, 'metal');
      b(4, 6.6, 9, 8, 6.4, 0.5, 'sage');
      b(5, 8, 9.5, 3, 2, 0.35, 'teal');
      b(8, 9, 9.5, 2, 3, 0.35, 'white');
      b(8, 7.5, 9.5, 2.7, 1.7, 0.35, 'teal');
      for (const v of [4.5, 6.5, 8.5, 10.5]) b(v, 7, 10, 0.28, 5.7, 0.4, 'silver');
      b(7.3, 9.1, 10.3, 1.5, 1.6, 0.3, 'ivory');
      break;
    case 'pinboard':
      b(-3, 1, 0.1, 22, 12, 1, 'deepwood');
      b(-2.5, 1.6, 1, 21, 10.8, 0.45, 'latte');
      for (const [a, v, w, h] of [
        [-1.5, 4.1, 5.2, 6.8],
        [4.7, 3, 5.4, 5.9],
        [11.1, 4.3, 5.1, 6.4]
      ]) {
        b(a, v, 1.48, w, h, 0.18, 'paper');
        b(a + 1, v + h - 0.8, 1.72, 1, 0.65, 0.3, 'red');
        for (let j = 0; j < 3; j++) b(a + 0.7, v + 1.2 + j * 1.05, 1.72, 2 + (j % 2), 0.26, 0.3, 'metal');
      }
      b(4.5, 3.6, 1.8, 4.5, 5.8, 0.14, 'photograph');
      b(2, 7, 1.86, 9, 0.32, 0.16, 'red');
      b(8, 6.4, 1.88, 6, 0.32, 0.16, 'auburn');
      break;
    case 'tube_light':
      for (const a of [3, 13]) b(a, 13.1, 6, 0.45, 2.9, 0.55, 'metal');
      b(0.6, 12.2, 4, 14.8, 1.2, 4.7, 'silver');
      b(1.5, 11.5, 4.6, 13, 0.8, 1.1, 'lit');
      b(1.5, 11.5, 7, 13, 0.8, 1.1, 'paper');
      break;
    case 'radio':
      b(1, 0, 4, 14, 7, 7, 'deepwood');
      face(1.6, 0.7, 12.8, 5.5, 'latte', 11);
      for (let i = 0; i < 4; i++) face(2.3, 1.4 + i * 0.9, 6, 0.32, 'metal', 11.3);
      face(9, 3, 4.6, 2, 'screen', 11.3);
      face(10, 3.6, 0.45, 1, 'red', 11.5);
      b(12, 6.8, 5, 0.4, 5.2, 0.4, 'silver');
      break;
    case 'cup':
      b(5, 0, 5, 5, 4.7, 5, 'paper');
      b(5.6, 4.6, 5.6, 3.8, 0.25, 3.8, 'deepwood');
      b(10, 1, 6, 2, 2.8, 1, 'cream');
      break;
    case 'watch':
      b(4, 5, 0.05, 8, 8, 1, 'deepwood');
      b(4.8, 5.8, 1, 6.4, 6.4, 0.3, 'paper');
      b(7.8, 8.4, 1.3, 0.5, 2.5, 0.2, 'ink');
      b(8, 8.2, 1.3, 2, 0.5, 0.2, 'ink');
      break;
  }
}
// Rust emits two words per visible face: a local block/face/AO descriptor and four light samples.
const variedGrain = new Set([
  'plank',
  'plaster',
  'grass',
  'sand',
  'stone',
  'dirt',
  'road',
  'leaves',
  'pine',
  'snow'
]);
function windowTrim(batch, x, y, z, face) {
  const bar = 1 / 32,
    lip = 0.018;
  if (face === 4 || face === 5) {
    const at = z + (face === 4 ? 1 - lip : -lip);
    batch.box(x, y, at, 1, bar, 2 * lip, 'metal');
    batch.box(x, y + bar, at, bar, 1 - bar, 2 * lip, 'silver');
  } else if (face < 2) {
    const at = x + (face === 0 ? 1 - lip : -lip);
    batch.box(at, y, z, 2 * lip, bar, 1, 'metal');
    batch.box(at, y + bar, z, 2 * lip, 1 - bar, bar, 'silver');
  }
}
export function chunkBuffers(words, items) {
  const batch = new Batch();
  const tiles = items.map((_, id) => faceTiles(id));
  const ao = [1, 1, 1, 1];
  const solid = [],
    cutout = [];
  let low = Infinity,
    high = -Infinity;
  for (let i = 0; i < words.length; i += 2) {
    const header = words[i],
      lighting = words[i + 1];
    const x = header & 15,
      z = (header >>> 4) & 15,
      y = (header >>> 8) & 63,
      f = (header >>> 14) & 7,
      id = (header >>> 17) & 127;
    low = Math.min(low, y);
    high = Math.max(high, y + 1);
    batch.triangles = items[id].form === 'cutout' ? cutout : solid;
    if (f === 6) {
      batch.lighting = lighting;
      prop(batch, items[id].key, x, y, z);
    } else {
      const form = items[id].form;
      for (let j = 0; j < 4; j++) ao[j] = 1 - 0.105 * ((header >>> (24 + j * 2)) & 3);
      const mirror = variedGrain.has(items[id].key) && ((x * 7 + z * 11 + y * 3) & 3) === 1;
      batch.face(
        x,
        y,
        z,
        1,
        form === 'slab' ? 0.5 : form === 'liquid' ? 0.875 : 1,
        1,
        f,
        tiles[id][f],
        ao,
        1,
        lighting,
        mirror
      );
      if (items[id].key === 'glass') {
        batch.triangles = solid;
        batch.lighting = lighting;
        windowTrim(batch, x, y, z, f);
      }
    }
  }
  const index = solid.concat(cutout);
  const pack = (a) => Uint8Array.from(a, (n) => Math.round(Math.min(1, n) * 255));
  return {
    position: new Float32Array(batch.position),
    normal: Int8Array.from(batch.normal, (n) => n * 127),
    color: pack(batch.color),
    uv: Uint16Array.from(batch.uv, (n) => Math.round(n * 65535)),
    irradiance: pack(batch.irradiance),
    index: batch.position.length / 3 > 65535 ? new Uint32Array(index) : new Uint16Array(index),
    solidCount: solid.length,
    low: Number.isFinite(low) ? low : 0,
    high: Number.isFinite(high) ? high : 0
  };
}
export function uploadedGeometry(data) {
  const geometry = new BufferGeometry();
  for (const [name, size, normalized] of [
    ['position', 3, false],
    ['normal', 3, true],
    ['color', 3, true],
    ['uv', 2, true],
    ['irradiance', 3, true]
  ]) {
    geometry.setAttribute(name, new BufferAttribute(data[name], size, normalized));
  }
  geometry.setIndex(new BufferAttribute(data.index, 1));
  if (data.solidCount) geometry.addGroup(0, data.solidCount, 0);
  if (data.index.length > data.solidCount)
    geometry.addGroup(data.solidCount, data.index.length - data.solidCount, 1);
  const center = (data.low + data.high) * 0.5;
  // Sign panels and awnings can overhang a voxel. Bound them without rescanning the upload.
  geometry.boundingSphere = new Sphere(
    new Vector3(8, center, 8),
    Math.hypot(9, 9, center - data.low + 1)
  );
  return geometry;
}

export function heldGeometry(item, id) {
  const batch = new Batch(),
    key = item.key;
  if (item.form === 'prop') {
    prop(batch, key, 0, 0, 0);
    return batch.geometry().translate(-0.5, 0, -0.5).scale(0.23, 0.23, 0.23);
  }
  if (item.form !== 'item') {
    batch.box(-0.09, 0, -0.09, 0.18, 0.18, 0.18, faceTiles(id));
    return batch.geometry();
  }
  const b = (x, y, z, w, h, d, color) =>
    batch.box(x * 0.024, y * 0.024, z * 0.024, w * 0.024, h * 0.024, d * 0.024, color);
  if (key.endsWith('_pick') || key === 'stick') {
    for (let i = 0; i < 8; i++) b(-3 + i * 0.7, i * 0.7, 0, 1.15, 1.15, 1, 'wood');
    if (key.endsWith('_pick')) {
      const color = key === 'wood_pick' ? 'wood' : key === 'stone_pick' ? 'grey' : 'orange';
      for (const [x, y, w, h] of [
        [-2, 7, 4, 1.25],
        [1.5, 6, 2.5, 2],
        [3.5, 4.5, 1.55, 2.4],
        [4.5, 2.6, 1.25, 2.5]
      ])
        b(x, y, -0.28, w, h, 1.55, color);
    }
  } else {
    const color =
      { apple: 'red', fuel: 'ink', ore: 'orange', ingot: 'orange', shard: 'cyan' }[key] || 'cream';
    b(-2.7, 1, -1.6, 5.4, 3, 3.2, color);
    b(-2, 0, -1.2, 4, 5, 2.4, color);
    if (key === 'apple') {
      b(0, 5, 0, 0.8, 1.6, 0.75, 'deepwood');
      b(0.5, 5.5, 0, 1.6, 0.65, 0.7, 'moss');
    }
    if (key === 'shard') b(-1.3, 4, -0.8, 2.2, 2.3, 1.6, 'cyan');
  }
  return batch.geometry();
}
