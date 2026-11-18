import { CanvasTexture, SRGBColorSpace, NearestFilter, NearestMipmapLinearFilter } from 'three';
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
const instrumentTiles = { crtFace: 249, waveDial: 250, nixieLeft: 251, nixieRight: 252 };
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
export function makeAtlas(items) {
  const atlas = document.createElement('canvas');
  atlas.width = atlas.height = 512;
  const out = atlas.getContext('2d');
  out.imageSmoothingEnabled = false;
  const tile = document.createElement('canvas');
  tile.width = tile.height = 16;
  const ctx = tile.getContext('2d');
  function put(index, paint) {
    ctx.clearRect(0, 0, 16, 16);
    paint(ctx);
    const x = (index % 16) * 32 + 8,
      y = (index >> 4) * 32 + 8;
    out.drawImage(tile, x, y);
    // Extend the edges into the gutter for linear mip interpolation at distance.
    for (const [sx, dx] of [
      [0, -8],
      [15, 16]
    ])
      out.drawImage(tile, sx, 0, 1, 16, x + dx, y, 8, 16);
    for (const [sy, dy] of [
      [0, -8],
      [15, 16]
    ])
      out.drawImage(atlas, x - 8, y + sy, 32, 1, x - 8, y + dy, 32, 8);
  }
  items.forEach((item, id) => {
    for (let i = 0; i < 3; i++) put(id * 3 + i, (c) => art(c, id, item.key, i));
  });
  for (const s of Object.values(sw))
    put(s.id, (c) => {
      c.fillStyle = s.color;
      c.fillRect(0, 0, 16, 16);
      const r = seeded(s.id * 2127);
      for (let i = 0; i < 65; i++) {
        c.fillStyle = tint(s.color, (r() - 0.5) * 12);
        c.fillRect((r() * 16) | 0, (r() * 16) | 0, 1 + (r() > 0.7), 1);
      }
    });
  const texture = new CanvasTexture(atlas);
  texture.colorSpace = SRGBColorSpace;
  texture.magFilter = NearestFilter;
  texture.minFilter = NearestMipmapLinearFilter;
  texture.anisotropy = 2;
  const icons = items.map((item, id) => icon(atlas, item, id));
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
  return { texture, icons, instruments };
}
function icon(atlas, item, id) {
  const c = document.createElement('canvas');
  c.width = c.height = 32;
  const g = c.getContext('2d');
  g.imageSmoothingEnabled = false;
  const face = (n, mat, shade) => {
    g.save();
    g.setTransform(...mat);
    g.drawImage(atlas, (n % 16) * 32 + 8, (n >> 4) * 32 + 8, 16, 16, 0, 0, 16, 16);
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
