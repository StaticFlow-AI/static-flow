import { CanvasTexture, SRGBColorSpace, LinearFilter, LinearMipmapLinearFilter } from 'three';
import { seeded } from './stage.js';

const BASE = {
  air: '#ffffff',
  grass: '#639846',
  dirt: '#866046',
  stone: '#858680',
  bedrock: '#494b52',
  log: '#82603e',
  leaves: '#548339',
  plank: '#c59561',
  cobble: '#838681',
  sand: '#d8c491',
  coal_ore: '#828480',
  copper_ore: '#848783',
  crystal_ore: '#67677b',
  brick: '#a5624b',
  plaster: '#dfd7bc',
  glass: '#b5d7d8',
  steel: '#7b8b8b',
  tile: '#c6bfb0',
  shelf: '#8f6948',
  bench: '#ad794b',
  furnace: '#6f7471',
  relay: '#767b79',
  wave: '#ddd9c5',
  phone: '#425a56',
  crt: '#343d3d',
  leap: '#424e56',
  upa: '#c3c9bf',
  sofa: '#557467',
  table: '#9c693e',
  cabinet: '#aa8e66',
  monitor: '#515b55',
  flower: '#f3df99',
  torch: '#eeb35b',
  lamp: '#fff2b8',
  water: '#599aa8',
  slab: '#8a9392',
  books: '#a15951',
  curtain: '#c98286',
  nixie: '#e9a457',
  panda: '#e5e2ce',
  pine: '#3d6b4f',
  snow: '#dfeae4',
  stick: '#906140',
  wood_pick: '#a4794b',
  stone_pick: '#989c99',
  copper_pick: '#d9885e',
  fuel: '#41474a',
  ore: '#bf7956',
  ingot: '#d18461',
  shard: '#58bec0',
  road: '#787b79',
  stripe: '#dbd4bd',
  sign: '#bf8d61',
  sapling: '#70a24c',
  beacon: '#9ee3cd',
  rug: '#b48365',
  fern: '#689b45',
  watch: '#e1dcbd',
  apple: '#ce634e',
  roof: '#7b8784',
  fan: '#899c91',
  pinboard: '#a48a66',
  tube_light: '#e4dfc6',
  radio: '#677876',
  cup: '#dbb997',
  awning: '#84a297',
  rail: '#83918a'
};
// The small swatches share the same atlas and draw call as the terrain and characters.
export const sw = Object.fromEntries(
  Object.entries({
    ink: '#37383c',
    metal: '#687575',
    silver: '#bac5bf',
    ivory: '#eee6cf',
    paper: '#f4ebd1',
    latte: '#bda988',
    hide: '#976941',
    wood: '#b47e4d',
    deepwood: '#77523c',
    green: '#597664',
    seam: '#394e46',
    sage: '#7d9780',
    amber: '#ffba65',
    lit: '#ffeea7',
    lime: '#aac741',
    cyan: '#69c6bf',
    blue: '#648897',
    darkblue: '#3d5766',
    red: '#b35f50',
    rose: '#cf8397',
    white: '#eee8dc',
    black: '#39363c',
    skin: '#e0b195',
    skinlit: '#edc2a0',
    hair: '#54403c',
    chestnut: '#8f5349',
    auburn: '#ab5f51',
    gold: '#ccad70',
    violet: '#746079',
    navy: '#4a5368',
    teal: '#6f9692',
    pink: '#de9ca7',
    cream: '#d1b68a',
    khaki: '#998a68',
    denim: '#576781',
    moss: '#778765',
    darkskin: '#c59575',
    orange: '#dd9362',
    purple: '#857b9c',
    indigo: '#597895',
    grey: '#939695',
    banana: '#91b246',
    tube: '#e6c498',
    screen: '#355452',
    shadow: '#535852'
  }).map(([key, color], i) => [key, { id: 201 + i, color }])
);
const instrumentTiles = {
  crtFace: 249,
  waveDial: 250,
  nixieLeft: 251,
  nixieRight: 252,
  ovenGlass: 253,
  circuit: 254,
  photograph: 255
};
for (const [name, id] of Object.entries(instrumentTiles)) sw[name] = { id, color: '#394640' };
const numerals = [
  '111101101101111',
  '010110010010111',
  '111001111100111',
  '111001111001111',
  '101101111001001',
  '111100111001111',
  '111100111101111',
  '111001010010010',
  '111101111101111',
  '111101111001111'
];
export const faceTiles = (id) => [id * 3 + 1, id * 3 + 1, id * 3, id * 3 + 2, id * 3 + 1, id * 3 + 1];

function tint(hex, delta) {
  const value = parseInt(hex.slice(1), 16);
  return (
    'rgb(' +
    [value >> 16, (value >> 8) & 255, value & 255]
      .map((n) => Math.max(0, Math.min(255, n + delta)))
      .join(',') +
    ')'
  );
}
function art(ctx, id, key, kind) {
  const r = seeded(8253 + (id * 131 + kind) * 271);
  const fill = (x, y, w, h, c) => {
    ctx.fillStyle = c;
    ctx.fillRect(x, y, w, h);
  };
  const base = BASE[key] || '#959283';
  const grain = (color, range = 15, n = 110) => {
    fill(0, 0, 16, 16, color);
    for (let i = 0; i < n; i++)
      fill(
        (r() * 16) | 0,
        (r() * 16) | 0,
        1 + (r() > 0.76 ? 1 : 0),
        1,
        tint(color, Math.round((r() - 0.5) * range * 2))
      );
  };
  const dark = tint(base, -30),
    light = tint(base, 22);
  grain(key === 'plank' ? '#bd946b' : base, key === 'plank' ? 6 : 15);
  switch (key) {
    case 'grass':
      if (kind === 2) {
        grain(BASE.dirt);
        break;
      }
      if (kind === 1) {
        grain(BASE.dirt);
        for (let x = 0; x < 16; x++) {
          const d = 2 + ((r() * 4) | 0);
          fill(x, 0, 1, d, tint(base, (r() - 0.5) * 27));
          fill(x, d, 1, 1, '#716447');
        }
      } else
        for (let i = 0; i < 25; i++)
          fill((r() * 16) | 0, (r() * 16) | 0, 2, 1, tint(base, (r() - 0.5) * 44));
      break;
    case 'log':
      if (kind !== 1) {
        for (let i = 0; i < 7; i += 2) {
          fill(i, i, 16 - i * 2, 16 - i * 2, i % 4 ? '#a98654' : '#b69561');
          fill(i + 1, i + 1, 14 - i * 2, 14 - i * 2, '#cbab75');
        }
        fill(8, 7, 2, 2, '#957147');
      } else
        for (let i = 0; i < 34; i++)
          fill((r() * 16) | 0, (r() * 13) | 0, 1, 3 + ((r() * 8) | 0), r() > 0.48 ? dark : light);
      break;
    case 'roof':
      if (kind === 2) {
        grain('#d4cbb7', 5);
        fill(0, 15, 16, 1, '#bcb6a5');
        fill(0, 0, 1, 16, '#c4beac');
      } else {
        grain('#85918c', 10);
        fill(0, 0, 1, 16, '#657670');
      }
      break;
    case 'plank':
    case 'table':
      for (let y = 0; y < 16; y += 4) {
        fill(0, y, 16, 1, '#a78360');
        fill(0, y + 1, 16, 1, '#c8a17a');
        fill(y === 4 || y === 12 ? 11 : 3, y, 1, 4, '#a58763');
      }
      for (let i = 0; i < 10; i++) fill((r() * 14) | 0, ((r() * 4) | 0) * 4 + 2, 2, 1, '#b48f69');
      break;
    case 'stone':
    case 'coal_ore':
    case 'copper_ore':
    case 'crystal_ore':
      for (let i = 0; i < 25; i++) {
        const x = (r() * 14) | 0,
          y = (r() * 16) | 0;
        fill(x, y, 2 + ((r() * 3) | 0), 1, tint(base, (r() - 0.5) * 42));
      }
      if (key !== 'stone') {
        const ore =
          key === 'coal_ore'
            ? ['#42474a', '#5c6261']
            : key === 'copper_ore'
              ? ['#a66345', '#dda070']
              : ['#369394', '#8be4d0'];
        for (const [x, y] of [
          [2, 3],
          [9, 2],
          [6, 8],
          [12, 11],
          [2, 13]
        ]) {
          fill(x, y, 3, 3, ore[0]);
          fill(x + 1, y - 1, 2, 2, ore[1]);
        }
      }
      break;
    case 'cobble':
    case 'brick':
    case 'bedrock':
      fill(0, 0, 16, 16, key === 'brick' ? '#cab999' : dark);
      for (let row = 0; row < 4; row++)
        for (let col = -1; col < 4; col++) {
          const x = col * (key === 'brick' ? 8 : 6) + (row % 2) * 3,
            y = row * 4,
            w = key === 'brick' ? 7 : 5;
          fill(x, y, w, 3, tint(base, (r() - 0.5) * 30));
          fill(x + 1, y, w - 2, 1, light);
        }
      break;
    case 'leaves':
    case 'pine':
      for (let i = 0; i < 52; i++) {
        let x = (r() * 16) | 0,
          y = (r() * 16) | 0;
        fill(x, y, 3, 2, r() > 0.5 ? dark : light);
        if (i % 6 === 0) ctx.clearRect(x, y, 1, 2);
      }
      break;
    case 'glass':
      ctx.clearRect(0, 0, 16, 16);
      fill(0, 0, 16, 1, '#9eaca3');
      fill(0, 0, 1, 16, '#b4bcb0');
      fill(15, 0, 1, 16, '#889288');
      fill(0, 15, 16, 1, '#949f94');
      fill(2, 4, 2, 1, '#c3d8d2');
      fill(4, 3, 1, 1, '#c3d8d2');
      fill(11, 12, 3, 1, '#c3d8d2');
      break;
    case 'tile':
      for (let i = 0; i < 4; i++) {
        const x = (i % 2) * 8,
          y = (i >> 1) * 8;
        fill(x, y, 8, 8, i === 1 || i === 2 ? '#8b9488' : '#d0cabb');
        fill(x, y, 8, 1, '#a6a79a');
      }
      break;
    case 'steel':
    case 'slab':
      fill(0, 0, 16, 1, light);
      fill(0, 15, 16, 1, dark);
      for (const x of [1, 13]) for (const y of [2, 13]) fill(x, y, 2, 1, '#596460');
      break;
    case 'plaster':
      grain(base, 5);
      for (let i = 0; i < 8; i++) fill((r() * 16) | 0, (r() * 16) | 0, 2, 1, '#cdc6ae');
      break;
    case 'shelf':
      if (kind !== 1) {
        grain('#a08059');
        break;
      }
      for (let j = 0; j < 2; j++) {
        fill(0, j * 8, 16, 7, '#514a40');
        let x = 1;
        for (let i = 0; i < 6; i++) {
          const w = 1 + ((r() * 2) | 0),
            y = 1 + ((r() * 2) | 0) + j * 8;
          fill(x, y, w, j * 8 + 7 - y, ['#a46052', '#b59966', '#58756e', '#778894', '#dfc49a'][i % 5]);
          fill(x, y + 1, w, 1, '#d8c6a1');
          x += w + 1;
        }
        fill(0, j * 8 + 7, 16, 1, '#c19766');
      }
      break;
    case 'cabinet':
      if (kind === 0) {
        grain('#bda984', 7);
        fill(0, 0, 16, 1, '#cdb996');
      } else {
        fill(1, 1, 14, 14, '#947958');
        fill(2, 2, 12, 12, base);
        fill(5, 4, 6, 1, '#5b6259');
      }
      break;
    case 'bench':
      if (kind === 0) {
        grain('#b98c58');
        for (const x of [3, 8, 13]) {
          fill(x, 2, 1, 12, '#674a35');
          fill(2, x, 12, 1, '#674a35');
        }
      } else {
        for (const x of [0, 13]) fill(x, 0, 3, 16, '#79543b');
        fill(3, 3, 10, 2, '#554c3c');
        fill(5, 6, 2, 5, '#8b9991');
        fill(9, 6, 2, 6, '#63513c');
      }
      break;
    case 'furnace':
      if (kind === 1) {
        fill(2, 2, 12, 4, '#464d4c');
        fill(3, 3, 10, 1, '#a8aaa0');
        fill(2, 8, 12, 6, '#343e3b');
        fill(4, 11, 8, 2, '#b06b41');
        fill(6, 10, 4, 2, '#efb864');
      } else {
        fill(2, 2, 12, 12, dark);
        fill(3, 3, 10, 10, base);
      }
      break;
    case 'lamp':
    case 'beacon':
      fill(0, 0, 16, 1, '#a8956d');
      fill(0, 0, 1, 16, '#ae9b78');
      fill(15, 0, 1, 16, '#afa37e');
      fill(3, 3, 9, 1, '#ffefc4');
      break;
    case 'water':
      for (let j = 0; j < 13; j++) fill((r() * 16) | 0, (r() * 16) | 0, 4, 1, light);
      break;
    case 'stripe':
      grain(BASE.road);
      fill(0, 2, 16, 11, base);
      break;
    case 'curtain':
      for (let x = 0; x < 16; x += 4) {
        fill(x, 0, 2, 16, dark);
        fill(x + 2, 0, 1, 16, light);
      }
      break;
  }
}
// Fine marks are drawn on a 32-texel surface. Keep the low-frequency pattern legible on distant mip levels.
function surfaceArt(ctx, id, key, side) {
  art(ctx, id, key, side);
  ctx.save();
  ctx.scale(0.5, 0.5);
  const rand = seeded(971 + id * 743 + side * 83);
  function mark(x, y, w, h, color) {
    ctx.fillStyle = color;
    ctx.fillRect(x, y, w, h);
  }
  const scatter = (n, color, w = 1, h = 1) => {
    for (let i = 0; i < n; i++) mark((rand() * 32) | 0, (rand() * 32) | 0, w, h, color);
  };
  if (key === 'plank' || (key === 'roof' && side === 2)) {
    const ceiling = key === 'roof';
    mark(0, 0, 32, 32, ceiling ? '#dfdac8' : '#bd946a');
    if (ceiling) {
      mark(0, 0, 32, 1, '#a7a493');
      mark(0, 1, 32, 1, '#f0ead5');
      mark(0, 0, 1, 32, '#b5b09e');
      scatter(74, '#9693822b');
      scatter(42, '#fff8dc46', 2);
    } else {
      for (let row = 0; row < 4; row++) {
        const y = row * 8,
          joint = [5, 23, 12, 28][row];
        mark(0, y, 32, 8, ['#bd966e', '#c8a279', '#c49a70', '#ba9169'][row]);
        mark(0, y, 32, 1, '#987a5c');
        mark(0, y + 1, 32, 1, '#d4af86');
        mark(joint, y + 1, 1, 7, '#9c7d60');
        mark(joint + 1, y + 1, 1, 7, '#d5ad80');
        for (let k = 0; k < 8; k++)
          mark(
            (rand() * 32) | 0,
            y + 2 + ((rand() * 6) | 0),
            2 + ((rand() * 7) | 0),
            1,
            k % 3 ? '#8d715636' : '#f3cf9654'
          );
      }
      mark(15, 19, 5, 1, '#9e7a53');
      mark(17, 18, 2, 1, '#ac875d');
    }
  } else if (key === 'plaster') {
    mark(0, 0, 32, 32, '#e2dac4');
    scatter(130, '#a79b841e');
    scatter(46, '#fff6da3b', 2);
    for (let k = 0; k < 4; k++) mark((rand() * 30) | 0, (rand() * 32) | 0, 2, 1, '#a89e8b58');
  } else if (key === 'brick' || key === 'cobble') {
    const brick = key === 'brick';
    mark(0, 0, 32, 32, brick ? '#b5a797' : '#767b76');
    for (let row = 0; row < 4; row++)
      for (let col = -1; col < 3; col++) {
        const x = col * 16 + (row % 2) * 8,
          y = row * 8;
        const tone = brick
          ? ['#a16754', '#915c50', '#b17860', '#a8725c']
          : ['#8f9186', '#9c9c8d', '#8a9089', '#aaa89b'];
        mark(x + 1, y + 1, 15, 7, tone[(rand() * 4) | 0]);
        mark(x + 2, y + 1, 12, 1, '#e2c3a253');
        mark(x + 15, y + 3, 1, 5, '#49463945');
        mark(x + 2, y + 7, 13, 1, '#5d524546');
      }
    scatter(55, '#37383228');
    scatter(32, '#eee2b940');
  } else if (key === 'leaves' || key === 'pine') {
    const pine = key === 'pine';
    for (let i = 0; i < 34; i++) {
      const x = (rand() * 30) | 0,
        y = (rand() * 30) | 0;
      mark(x, y, 3, 2, pine ? '#3c6556' : '#538650');
      mark(x + 1, y, 2, 1, pine ? '#7c9b77' : '#87ad65');
      if (i % 6 === 0) ctx.clearRect(x - 1, y + 1, 2, 2);
    }
  } else if (key === 'water') {
    for (let i = 0; i < 15; i++) {
      const x = (rand() * 30) | 0,
        y = (rand() * 31) | 0;
      mark(x, y, (2 + rand() * 4) | 0, 1, '#b9ded08a');
      mark(x + 2, y + 1, 3, 1, '#437e8869');
    }
  } else if (key === 'glass') {
    ctx.clearRect(0, 0, 32, 32);
    mark(0, 0, 32, 1, '#758884');
    mark(0, 0, 1, 32, '#879a92');
    mark(31, 0, 1, 32, '#667772');
    mark(0, 31, 32, 1, '#71837b');
    mark(1, 1, 30, 1, '#d2d9c6');
    mark(1, 1, 1, 30, '#aebfaf');
    for (const [x, y] of [
      [4, 10],
      [10, 5],
      [23, 25]
    ]) {
      mark(x, y, 3, 1, '#c7dcd5');
      mark(x + 3, y - 1, 2, 1, '#c7dcd5');
    }
  } else if (key === 'roof' || key === 'steel' || key === 'slab') {
    scatter(34, '#e7edcf30', 3);
    scatter(31, '#46534d34', 2);
    for (const x of [3, 27])
      for (const y of [3, 26]) {
        mark(x, y, 2, 2, '#65716b');
        mark(x, y, 1, 1, '#cad0bd');
      }
  } else if (key === 'log' && side === 1) {
    for (let i = 0; i < 33; i++)
      mark(
        (rand() * 32) | 0,
        (rand() * 30) | 0,
        1,
        2 + ((rand() * 7) | 0),
        i % 2 ? '#58483259' : '#c4a4684d'
      );
  } else if (
    ['stone', 'coal_ore', 'copper_ore', 'crystal_ore', 'road', 'grass', 'dirt', 'sand', 'snow'].includes(
      key
    )
  ) {
    scatter(65, '#343b3828', 2);
    scatter(65, '#f3e8be37');
    if (key.endsWith('_ore'))
      for (const [x, y] of [
        [4, 6],
        [18, 4],
        [12, 16],
        [24, 22],
        [4, 26]
      ]) {
        mark(
          x + 2,
          y - 2,
          3,
          1,
          key === 'crystal_ore' ? '#d0fff1' : key === 'coal_ore' ? '#858f8b' : '#ffbe8b'
        );
        mark(x + 1, y + 5, 3, 1, '#4e514c');
      }
  } else if (key === 'cabinet' && side === 1) {
    for (let i = 0; i < 13; i++) mark((rand() * 22 + 4) | 0, (rand() * 19 + 9) | 0, 1, 2, '#5c4b362b');
    mark(10, 8, 12, 1, '#c5b394');
    mark(11, 9, 10, 1, '#534d40');
    mark(4, 22, 5, 3, '#dcd4ae');
    mark(5, 23, 3, 1, '#8c8775');
  }
  ctx.restore();
}
function panelArt(ctx, key) {
  const fill = (x, y, w, h, c) => {
    ctx.fillStyle = c;
    ctx.fillRect(x, y, w, h);
  };
  ctx.save();
  ctx.scale(0.5, 0.5);
  if (key === 'ovenGlass') {
    fill(0, 0, 32, 32, '#344548');
    fill(2, 2, 28, 27, '#4a5851');
    for (let y = 3; y < 28; y += 3) for (let x = 2 + (y % 2); x < 30; x += 3) fill(x, y, 1, 1, '#293c3b');
    fill(3, 2, 17, 1, '#91a499');
    fill(2, 3, 1, 13, '#738b7c');
    fill(24, 23, 5, 1, '#607868');
  } else if (key === 'circuit') {
    fill(0, 0, 32, 32, '#294442');
    fill(2, 2, 28, 3, '#537c69');
    for (let y = 7; y < 28; y += 4) {
      fill(3, y, 3, 1, '#a9c990');
      fill(8, y, 4 + ((y * 3) % 10), 1, '#86ac88');
      fill(6, y + 1, 1, 1, '#668e76');
    }
    fill(23, 8, 6, 12, '#324f49');
    for (let x = 23; x < 29; x++) fill(x, 18 - ((x * 3) % 7), 1, 1, '#b3d09c');
    fill(3, 29, 15, 1, '#4b7463');
    fill(7, 25, 2, 2, '#daaa6b');
  } else {
    fill(0, 0, 32, 32, '#e9dfbb');
    fill(3, 3, 26, 20, '#9eafa1');
    fill(3, 13, 26, 10, '#728d85');
    fill(6, 10, 4, 11, '#526b64');
    fill(9, 15, 8, 8, '#60736c');
    fill(18, 6, 5, 17, '#697b72');
    fill(20, 4, 1, 3, '#687b73');
    fill(6, 26, 17, 1, '#99987f');
    fill(5, 4, 3, 3, '#be6d53');
  }
  ctx.restore();
}
export function sampleAtlas(shader) {
  if (!shader.defines.VOXEL_PRECISE) return;
  // Keep pixel boundaries crisp in close-up, filtering across an edge only when a screen pixel straddles it.
  // Preserve the original UV derivatives so mip selection and anisotropy still work at grazing angles.
  shader.fragmentShader = shader.fragmentShader.replace(
    '#include <map_fragment>',
    `#ifdef USE_MAP
    vec2 pixel=vMapUv*1024.0, seam=floor(pixel+.5);
    vec2 footprint=clamp(fwidth(pixel),vec2(.0001),vec2(1.));
    vec2 uv=(seam+clamp((pixel-seam)/footprint,-.5,.5))/1024.0;
    diffuseColor*=textureGrad(map,uv,dFdx(vMapUv),dFdy(vMapUv));
  #endif`
  );
}
export function makeAtlas(items) {
  const atlas = document.createElement('canvas');
  atlas.width = atlas.height = 1024;
  const out = atlas.getContext('2d');
  out.imageSmoothingEnabled = false;
  const tile = document.createElement('canvas');
  tile.width = tile.height = 32;
  const ctx = tile.getContext('2d');
  ctx.scale(2, 2);
  function put(index, paint) {
    ctx.clearRect(0, 0, 16, 16);
    paint(ctx);
    const x = (index % 16) * 64 + 16,
      y = (index >> 4) * 64 + 16;
    // Extrude all four edges and corners before mipmapping the atlas.
    out.drawImage(tile, x, y);
    out.drawImage(tile, 0, 0, 1, 32, x - 16, y, 16, 32);
    out.drawImage(tile, 31, 0, 1, 32, x + 32, y, 16, 32);
    for (const [src, dst] of [
      [0, -16],
      [31, 32]
    ]) {
      out.drawImage(tile, 0, src, 32, 1, x, y + dst, 32, 16);
      out.drawImage(tile, 0, src, 1, 1, x - 16, y + dst, 16, 16);
      out.drawImage(tile, 31, src, 1, 1, x + 32, y + dst, 16, 16);
    }
  }
  items.forEach((item, id) => {
    for (let i = 0; i < 3; i++) put(id * 3 + i, (c) => surfaceArt(c, id, item.key, i));
  });
  for (const s of Object.values(sw))
    put(s.id, (c) => {
      c.fillStyle = s.color;
      c.fillRect(0, 0, 16, 16);
      const r = seeded(s.id * 2127);
      const cloth = [
        sw.green.id,
        sw.denim.id,
        sw.khaki.id,
        sw.indigo.id,
        sw.navy.id,
        sw.rose.id
      ].includes(s.id);
      if (cloth) {
        for (let y = 0; y < 32; y++)
          for (let x = y % 2; x < 32; x += 3) {
            c.fillStyle = y % 3 ? '#101e2010' : '#f0e9c91a';
            c.fillRect(x / 2, y / 2, 0.5, 0.5);
          }
      }
      for (let i = 0; i < 65; i++) {
        c.fillStyle = tint(s.color, (r() - 0.5) * (cloth || s.id === sw.skin.id ? 5 : 10));
        c.fillRect((r() * 16) | 0, (r() * 16) | 0, 1 + (r() > 0.7), 1);
      }
    });
  for (const name of ['ovenGlass', 'circuit', 'photograph'])
    put(instrumentTiles[name], (c) => panelArt(c, name));
  const texture = new CanvasTexture(atlas);
  texture.colorSpace = SRGBColorSpace;
  texture.magFilter = LinearFilter;
  texture.minFilter = LinearMipmapLinearFilter;
  const icons = [];
  const itemIcon = (id) => (icons[id] ??= icon(atlas, items[id], id));
  let last = '';
  function instruments(s) {
    const key = [s.line, s.crt, s.armed].join('/');
    if (last === key) return;
    last = key;
    put(instrumentTiles.crtFace, (c) => {
      c.fillStyle = s.crt ? '#53746b' : '#393f40';
      c.fillRect(0, 0, 16, 16);
      for (let i = 0; i < 16; i += 2) {
        c.fillStyle = s.crt ? '#628576' : '#434b49';
        c.fillRect(0, i, 16, 1);
      }
      c.fillStyle = s.crt ? '#b1dfb4' : '#68726e';
      c.fillRect(2, 3, s.crt ? 10 : 4, 1);
      if (s.crt) {
        c.fillRect(2, 5, 6, 1);
        c.fillRect(3, 8, 9, 1);
      }
    });
    put(instrumentTiles.waveDial, (c) => {
      c.fillStyle = s.armed ? '#255a43' : '#35423a';
      c.fillRect(0, 0, 16, 16);
      c.fillStyle = s.armed ? '#b8ff97' : '#738d68';
      c.fillRect(2, 6, 12, 4);
      c.clearRect(5, 6, 1, 4);
      c.clearRect(10, 6, 1, 4);
    });
    const digits = s.number.replace('.', '');
    const colors = s.line === 'gate' ? ['#4e523c', '#d5efa0'] : ['#604333', '#ffc573'];
    [instrumentTiles.nixieLeft, instrumentTiles.nixieRight].forEach((tile, part) =>
      put(tile, (c) => {
        c.fillStyle = '#5c4a39';
        c.fillRect(0, 0, 16, 16);
        [...digits].forEach((n, i) => {
          const x = i * 4 + (i > 0 ? 2 : 0) - part * 16 + 1;
          c.fillStyle = colors[0];
          c.fillRect(x - 1, 1, 5, 14);
          c.fillStyle = colors[1];
          [...numerals[Number(n)]].forEach((ink, j) => {
            if (ink === '1') c.fillRect(x + (j % 3), 4 + Math.floor(j / 3) * 1.5, 1, 1.5);
          });
        });
        if (!part) {
          c.fillStyle = colors[1];
          c.fillRect(5, 11, 1, 2);
        }
      })
    );
    texture.needsUpdate = true;
  }
  return { texture, itemIcon, instruments };
}
function icon(atlas, item, id) {
  const c = document.createElement('canvas');
  c.width = c.height = 32;
  const g = c.getContext('2d', { willReadFrequently: true });
  g.imageSmoothingEnabled = false;
  const face = (n, mat, shade) => {
    g.save();
    g.setTransform(...mat);
    g.drawImage(atlas, (n % 16) * 64 + 16, (n >> 4) * 64 + 16, 32, 32, 0, 0, 16, 16);
    if (shade) {
      g.fillStyle = 'rgba(28,28,30,' + shade + ')';
      g.fillRect(0, 0, 16, 16);
    }
    g.restore();
  };
  if (
    item.form === 'item' ||
    ['torch', 'flower', 'sapling', 'fern', 'upa', 'phone', 'leap'].includes(item.key)
  ) {
    const px = (x, y, w, h, col) => {
      g.fillStyle = col;
      g.fillRect(x * 2, y * 2, w * 2, h * 2);
    };
    const b = BASE[item.key];
    if (item.key.endsWith('_pick') || item.key === 'stick' || item.key === 'torch') {
      for (let i = 0; i < 9; i++) {
        px(3 + i, 12 - i, 2, 2, '#614936');
        px(3 + i, 12 - i, 1, 1, '#cca16d');
      }
      if (item.key.endsWith('_pick')) {
        for (const [x, y, w, h] of [
          [4, 2, 6, 2],
          [9, 3, 3, 2],
          [11, 4, 2, 3],
          [12, 7, 2, 4]
        ]) {
          px(x, y + 1, w, h, '#454640');
          px(x, y, w, h, b);
          px(x, y, w, 1, tint(b, 36));
        }
      } else if (item.key === 'torch') {
        px(10, 2, 3, 3, '#ec9655');
        px(11, 1, 2, 3, '#ffdd7d');
      }
    } else if (['sapling', 'flower', 'fern'].includes(item.key)) {
      px(7, 6, 2, 8, '#6c7642');
      px(3, 6, 4, 3, '#74a851');
      px(8, 4, 4, 3, '#659443');
      px(5, 2, 5, 4, item.key === 'flower' ? '#fff0b3' : '#89b64f');
    } else {
      px(5, 3, 6, 2, tint(b, -25));
      px(3, 5, 10, 6, tint(b, -30));
      px(5, 11, 6, 2, tint(b, -32));
      px(5, 4, 6, 6, b);
      px(4, 6, 4, 4, tint(b, 20));
      px(6, 5, 3, 2, tint(b, 62));
    }
  } else {
    face(id * 3, [0.82, 0.4, -0.82, 0.4, 16, 2], 0);
    face(id * 3 + 1, [0.82, 0.4, 0, 0.87, 3, 8.4], 0.09);
    face(id * 3 + 1, [0.82, -0.4, 0, 0.87, 16, 14.8], 0.24);
  }
  return c.toDataURL();
}
