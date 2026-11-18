import * as T from 'three';
import { group, box, oval, cyl, beam, tube, ring, random, TAU } from './kit.mjs';

const wood = 0xa4855e,
  steel = 0x596665,
  cream = 0xd3ceb8,
  dark = 0x363a39;
const rng = random(1948);
function legs(p, x, y, z, w, h, d, color, r = 0.032) {
  for (const a of [-1, 1])
    for (const b of [-1, 1])
      beam(
        p,
        [x + a * (w / 2 - r), y, z + b * (d / 2 - r)],
        [x + a * (w / 2 - r), y + h, z + b * (d / 2 - r)],
        r,
        r,
        color
      );
}
function table(p, x, y, z, w, h, d) {
  box(p, [x, y + h, z], [w, 0.085, d], wood, 'timber', 0.025);
  legs(p, x, y, z, w - 0.14, h, d - 0.14, dark);
  box(p, [x, y + h - 0.1, z - d * 0.37], [w - 0.16, 0.16, 0.035], 0x766248, 'timber');
}
function carton(p, x, y, z, w, h, d) {
  box(p, [x, y + h / 2, z], [w, h, d], rng() > 0.5 ? 0xb09572 : 0xbfa581, 'cardboard', 0.01);
  box(p, [x, y + h + 0.002, z], [0.11, 0.006, d], 0xc8aa80);
  box(p, [x, y + h - 0.07, z + d / 2], [0.11, 0.14, 0.006], 0xc8aa80);
  box(p, [x - w * 0.2, y + h * 0.5, z + d / 2 + 0.003], [w * 0.26, h * 0.23, 0.007], 0xd9cdb2);
}
function book(p, x, y, z, h, color, w = 0.065) {
  box(p, [x, y + h / 2, z], [w, h, 0.21], color);
  box(p, [x, y + 0.75 * h, z + 0.108], [w * 0.72, 0.022, 0.004], 0xc6bba5);
}
function mug(p, x, y, z, color) {
  cyl(p, [x, y + 0.066, z], 0.052, 0.044, 0.13, color, 'ceramic');
  cyl(p, [x, y + 0.133, z], 0.045, 0.045, 0.003, 0x5b4130);
  ring(p, [x + 0.054, y + 0.07, z], 0.034, 0.008, color, 'ceramic', 16);
}
function shelves(p, x, y, z, w, h, d) {
  for (const a of [-1, 1])
    for (const b of [-1, 1])
      box(
        p,
        [x + a * (w / 2 - 0.024), y + h / 2, z + b * (d / 2 - 0.024)],
        [0.04, h, 0.045],
        steel,
        'metal'
      );
  for (let j = 0; j < 5; j++)
    box(p, [x, y + 0.12 + (j * (h - 0.22)) / 4, z], [w, 0.045, d], steel, 'metal');
}
function crt(p, x, y, z, w, body = 0xb5af97) {
  const d = w * 0.82,
    h = 0.77 * w;
  box(p, [x, y + h * 0.48, z - 0.046], [w * 0.85, h * 0.83, d * 0.86], body, 'paint', w * 0.07);
  box(p, [x, y + h * 0.53, z + d * 0.18], [w, h, d * 0.4], body, 'paint', w * 0.042);
  box(p, [x, y + h * 0.58, z + d * 0.397], [w * 0.83, h * 0.73, d * 0.07], 0x323b38, 'paint', w * 0.045);
  box(p, [x, y + h * 0.58, z + d * 0.44], [w * 0.75, h * 0.61, 0.016], 0x648775, 'crt', w * 0.022);
  for (let i = 0; i < 4; i++)
    box(
      p,
      [x + w * 0.23 + i * w * 0.04, y + h * 0.12, z + d * 0.405],
      [w * 0.025, 0.015, 0.013],
      0x5a5a4e
    );
  box(p, [x, y + 0.015, z], [w * 0.74, 0.04, d * 0.72], 0x5e5c51, 'paint', 0.012);
}
function keyboard(p, x, y, z, w = 0.62) {
  const d = w * 0.37;
  box(p, [x, y + 0.018, z], [w, 0.038, d], 0xb4ada0, 'paint', 0.014);
  for (let j = 0; j < 4; j++)
    for (let i = 0; i < 14; i++)
      box(
        p,
        [x + (w * (i - 6.5)) / 15, y + 0.041, z + (d * (j - 1.5)) / 5],
        [w * 0.056, 0.014, d * 0.147],
        i > 10 ? 0x87897f : 0xd6ceba
      );
}
function chair(p, x, y, z, angle) {
  const g = group(p, '', [x, y, z]);
  g.rotation.y = angle;
  box(g, [0, 0.48, 0], [0.46, 0.1, 0.45], 0x6f7771, 'cloth', 0.045);
  box(g, [0, 0.78, -0.22], [0.44, 0.45, 0.084], 0x676e67, 'cloth', 0.04).rotation.x = -0.11;
  cyl(g, [0, 0.28, 0], 0.035, 0.045, 0.32, steel, 'metal');
  for (let i = 0; i < 5; i++) {
    const a = i * 1.2566,
      ex = Math.sin(a) * 0.25,
      ez = Math.cos(a) * 0.25;
    beam(g, [0, 0.17, 0], [ex, 0.075, ez], 0.018, 0.014, dark);
    oval(g, [ex, 0.044, ez], [0.045, 0.04, 0.03], 0x333636);
  }
}
function softSeat(p, x, z) {
  const g = group(p, 'Velour_settee', [x, 0, z]);
  legs(g, 0, 0.035, 0, 2.32, 0.17, 0.79, 0x514630, 0.055);
  box(g, [0, 0.33, 0.03], [2.42, 0.31, 0.86], 0x73724d, 'cloth', 0.11);
  for (const s of [-1, 1]) {
    box(g, [s * 0.46, 0.55, 0.08], [0.89, 0.18, 0.68], 0x88865c, 'cloth', 0.085);
    box(g, [s * 0.48, 0.82, -0.265], [0.98, 0.62, 0.23], 0x7b7851, 'cloth', 0.11).rotation.x = -0.12;
    box(g, [s * 1.11, 0.54, 0.015], [0.26, 0.5, 0.9], 0x79754b, 'cloth', 0.11);
    tube(
      g,
      [
        [s * 0.91, 0.59, 0.4],
        [s * 0.46, 0.62, 0.43],
        [s * 0.07, 0.59, 0.4]
      ],
      0.012,
      0xa19a69
    );
  }
  // Mayuri's hand-sewn panda and knotted cushion.
  const toy = group(g, 'Panda_cushion', [-0.66, 0.56, 0.12]);
  oval(toy, [0, 0.18, 0], [0.19, 0.18, 0.13], 0xded9b6, 'cloth');
  for (const s of [-1, 1]) {
    oval(toy, [s * 0.126, 0.32, 0], [0.06, 0.058, 0.038], dark);
    oval(toy, [s * 0.073, 0.205, 0.107], [0.043, 0.052, 0.017], dark);
    oval(toy, [s * 0.071, 0.211, 0.124], [0.012, 0.013, 0.005], cream);
  }
  oval(toy, [0, 0.163, 0.132], [0.025, 0.017, 0.009], dark);
  for (let i = 0; i < 4; i++)
    oval(
      g,
      [
        0.56 + 0.09 * Math.sin(i * 1.57),
        0.68 + 0.038 * Math.cos(i * 1.57),
        0.12 + 0.08 * Math.cos(i * 1.57)
      ],
      [0.16, 0.106, 0.12],
      0x9aab83,
      'cloth'
    );
}
function wallClock(p, x, y, z) {
  const body = cyl(p, [x, y, z], 0.3, 0.3, 0.06, 0x3f4947, 'metal', 48);
  body.rotation.x = Math.PI / 2;
  const dial = cyl(p, [x, y, z + 0.037], 0.261, 0.261, 0.018, 0xe0dccb, 'ceramic', 48);
  dial.rotation.x = Math.PI / 2;
  for (let i = 0; i < 12; i++) {
    const a = (i * Math.PI) / 6;
    box(
      p,
      [x + 0.22 * Math.sin(a), y + 0.22 * Math.cos(a), z + 0.052],
      [0.014, i % 3 ? 0.025 : 0.043, 0.009],
      dark
    ).rotation.z = -a;
  }
  const hour = group(p, 'Clock_hour', [x, y, z + 0.065], { motion: 'hour' });
  const minute = group(p, 'Clock_minute', [x, y, z + 0.075], { motion: 'minute' });
  beam(hour, [0, -0.025, 0], [0, 0.14, 0], 0.011, 0.008, dark);
  beam(minute, [0, -0.027, 0], [0, 0.198, 0], 0.008, 0.005, dark);
}
function sign(p, name, at, size, face = 'sign') {
  // Front-facing material is replaced by a canvas-printed surface in the runtime.
  const s = group(p, name, at, { print: face });
  box(s, [0, 0, 0], [size[0], size[1], 0.035], 0x57584c, 'metal', 0.012);
  const g = new T.PlaneGeometry(size[0] - 0.045, size[1] - 0.042);
  const m = new T.Mesh(g, new T.MeshStandardMaterial({ color: 0xe5dfcd }));
  m.material.name = 'print_' + face;
  m.position.z = 0.021;
  s.add(m);
}
function fan(p, x, y, z) {
  const g = group(p, 'Ventilator', [x, y, z]);
  cyl(g, [0, 0.04, 0], 0.13, 0.17, 0.06, 0x73786b);
  beam(g, [0, 0.04, 0], [0, 0.43, 0], 0.026, 0.02, steel);
  const rotor = group(g, 'Fan_blades', [0, 0.49, 0.03], { motion: 'fan' });
  for (let i = 0; i < 4; i++) {
    const a = (i * Math.PI) / 2;
    oval(
      rotor,
      [0.105 * Math.sin(a), 0.105 * Math.cos(a), 0],
      [0.056, 0.114, 0.015],
      0x809487,
      'metal'
    ).rotation.z = -a;
  }
  for (let i = 1; i <= 5; i++) ring(g, [0, 0.49, 0.077], i * 0.044, 0.0045, steel, 'metal', 32);
  for (let i = 0; i < 12; i++) {
    const a = (i * Math.PI) / 6;
    beam(
      g,
      [0, 0.49, 0.094],
      [0.22 * Math.sin(a), 0.49 + 0.22 * Math.cos(a), 0.071],
      0.003,
      0.003,
      steel
    );
  }
}
function phoneWave(p) {
  const g = group(p, 'PhoneWave', [0.52, 0.96, -1.48], { interact: 'wave' });
  box(g, [0, 0.029, 0], [0.81, 0.058, 0.56], 0xa8a99a, 'metal', 0.014);
  box(g, [0, 0.46, 0], [0.81, 0.046, 0.56], 0xcac9b5, 'metal', 0.015);
  for (const s of [-1, 1]) box(g, [s * 0.387, 0.245, 0], [0.043, 0.44, 0.55], 0xb8b9a8, 'metal');
  box(g, [0, 0.243, -0.259], [0.8, 0.45, 0.04], 0xb8b9ab, 'metal');
  cyl(g, [-0.068, 0.068, 0], 0.201, 0.201, 0.013, 0xabae9a, 'ceramic', 36);
  const plate = group(g, 'Turntable', [-0.07, 0.076, 0], { motion: 'plate' });
  for (let i = 0; i < 3; i++)
    tube(
      plate,
      [
        [-0.14, 0.052, i * 0.035 - 0.04],
        [-0.06, 0.009, i * 0.035 - 0.057],
        [0.05, 0.017, i * 0.032 - 0.043],
        [0.13, 0.075, i * 0.027 - 0.024]
      ],
      0.023,
      i === 2 ? 0x889a49 : 0xb0bc69
    );
  const door = group(g, 'PhoneWave_door', [-0.393, 0.053, 0.282], { motion: 'door' });
  for (const [at, size] of [
    [
      [0.305, 0.034, 0],
      [0.61, 0.068, 0.046]
    ],
    [
      [0.305, 0.382, 0],
      [0.61, 0.067, 0.046]
    ],
    [
      [0.025, 0.212, 0],
      [0.05, 0.36, 0.046]
    ],
    [
      [0.579, 0.212, 0],
      [0.057, 0.36, 0.046]
    ]
  ])
    box(door, at, size, 0x938e80, 'metal', 0.008);
  for (let i = 0; i < 23; i++)
    beam(
      door,
      [0.054 + i * 0.022, 0.073, 0.007],
      [0.054 + i * 0.022, 0.35, 0.007],
      0.0019,
      0.0019,
      0x55584c
    );
  box(door, [0.557, 0.211, 0.039], [0.026, 0.235, 0.038], 0x5b5a50, 'metal', 0.012);
  box(g, [0.306, 0.25, 0.277], [0.16, 0.43, 0.045], 0xbab8a6, 'paint', 0.014);
  box(g, [0.305, 0.382, 0.304], [0.12, 0.062, 0.012], 0x30493c, 'crt');
  for (const y of [0.226, 0.112]) {
    const d = cyl(g, [0.303, y, 0.324], 0.048, 0.05, 0.032, 0x797b70, 'metal', 24);
    d.rotation.x = Math.PI / 2;
    box(g, [0.303, y + 0.019, 0.345], [0.006, 0.029, 0.006], cream);
  }
  const phone = box(g, [0.16, 0.5, 0.07], [0.12, 0.026, 0.205], 0x68665d, 'metal', 0.012);
  phone.rotation.y = 0.26;
  for (let k = 0; k < 11; k++)
    beam(
      door,
      [0.052, 0.083 + k * 0.025, 0.006],
      [0.548, 0.083 + k * 0.025, 0.006],
      0.0014,
      0.0014,
      0x55584c
    );
  for (let i = 0; i < 5; i++)
    box(
      g,
      [0.16 + Math.sin(i) * 0.025, 0.52, 0.04 + i * 0.021],
      [0.068, 0.003, 0.008],
      i < 2 ? 0x91a59c : 0x9f9c8f
    );
  tube(
    g,
    [
      [0.15, 0.507, 0.17],
      [0.33, 0.48, 0.22],
      [0.45, 0.22, 0.16],
      [0.46, -0.12, 0.2],
      [0.63, -0.17, 0.12],
      [0.79, 0.02, -0.17]
    ],
    0.008,
    0x40413e
  );
  for (let i = 0; i < 12; i++)
    box(g, [-0.19 + i * 0.026, 0.485, -0.136], [0.008, 0.004, 0.112], 0x71746a);
}
function smallPhone(p) {
  const g = group(p, 'Handset', [-2.6, 0.49, -1.28], { interact: 'phone' });
  g.rotation.y = -0.38;
  box(g, [0, 0.015, 0], [0.102, 0.027, 0.17], 0x6e484a, 'metal', 0.009);
  const top = box(g, [0, 0.079, -0.105], [0.099, 0.157, 0.019], 0x604047, 'metal', 0.008);
  top.rotation.x = -0.22;
  box(g, [0, 0.089, -0.089], [0.079, 0.096, 0.006], 0x91bbab, 'crt');
  for (let i = 0; i < 9; i++)
    box(
      g,
      [((i % 3) - 1) * 0.027, 0.031, (Math.floor(i / 3) - 1) * 0.026],
      [0.019, 0.003, 0.015],
      cream
    );
}
function upa(p) {
  const g = group(p, 'Metal_Upa', [-3.91, 0.58, -2.7], { interact: 'upa' });
  oval(g, [0, 0.093, 0], [0.075, 0.086, 0.066], 0xaeb9b5, 'metal');
  oval(g, [0, 0.17, 0.004], [0.095, 0.093, 0.08], 0xc2cbc4, 'metal');
  for (const s of [-1, 1]) {
    oval(g, [s * 0.073, 0.242, 0], [0.029, 0.035, 0.023], 0x839496, 'metal');
    oval(g, [s * 0.039, 0.18, 0.073], [0.01, 0.014, 0.005], dark);
    oval(g, [s * 0.061, 0.033, 0.02], [0.044, 0.024, 0.045], 0x919b96, 'metal');
  }
  oval(g, [0, 0.148, 0.081], [0.012, 0.009, 0.004], dark);
}
function bicycle(p, x, y, z) {
  const g = group(p, 'Road_bicycle', [x, y, z]);
  g.rotation.y = -0.45;
  for (const a of [-0.53, 0.53]) {
    ring(g, [a, 0.33, 0], 0.31, 0.026, 0x303735);
    ring(g, [a, 0.33, 0], 0.282, 0.009, 0xb3b8ad);
    for (let i = 0; i < 14; i++) {
      const ang = i * 0.449;
      beam(
        g,
        [a, 0.33, 0],
        [a + Math.sin(ang) * 0.28, 0.33 + Math.cos(ang) * 0.28, 0],
        0.003,
        0.003,
        steel
      );
    }
  }
  for (const [a, b] of [
    [
      [-0.53, 0.33, 0],
      [-0.13, 0.77, 0]
    ],
    [
      [-0.13, 0.77, 0],
      [0.02, 0.35, 0]
    ],
    [
      [0.02, 0.35, 0],
      [-0.53, 0.33, 0]
    ],
    [
      [0.02, 0.35, 0],
      [0.36, 0.79, 0]
    ],
    [
      [-0.13, 0.77, 0],
      [0.36, 0.79, 0]
    ],
    [
      [0.36, 0.79, 0],
      [0.53, 0.33, 0]
    ]
  ])
    beam(g, a, b, 0.018, 0.016, 0x94875a);
  oval(g, [-0.145, 0.82, 0], [0.105, 0.027, 0.063], dark);
  tube(
    g,
    [
      [0.36, 0.78, 0],
      [0.34, 0.94, 0],
      [0.39, 0.95, 0.18],
      [0.46, 0.92, 0.2]
    ],
    0.013,
    steel
  );
}
export function createLab() {
  const scene = new T.Scene(),
    root = group(scene, 'Braun_tube_workshop');
  const floors = [],
    solids = [];
  root.userData.floors = floors;
  root.userData.solids = solids;
  const floor = (x0, x1, z0, z1, h0, h1 = h0) => floors.push(x0, x1, z0, z1, h0, h1);
  const block = (x, z, w, d, y0, y1) => solids.push(x - w / 2, x + w / 2, z - d / 2, z + d / 2, y0, y1);
  const lab = group(root, 'Second_floor_laboratory'),
    shop = group(root, 'Braun_CRT_shop'),
    roof = group(root, 'Roof_and_stairwell'),
    town = group(root, 'Akihabara_street');
  const architecture = (p, at, size, col, mat = 'plaster') => {
    box(p, at, size, col, mat);
    block(at[0], at[2], size[0], size[2], at[1] - size[1] / 2, at[1] + size[1] / 2);
  };
  for (const [p, y] of [
    [lab, 0],
    [shop, -3.2],
    [roof, 3.2]
  ]) {
    floor(-5.5, 5.5, -4.35, 4.35, y);
    box(
      p,
      [0, y - 0.085, 0],
      [11.1, 0.17, 8.8],
      p === lab ? 0x937d5e : 0xacae9e,
      p === lab ? 'floor' : p === roof ? 'concrete' : 'linoleum'
    );
  }
  // The apartment ceiling and shop soffit are plaster beneath the floor slabs.
  for (const level of [0, -3.2])
    box(level === 0 ? lab : shop, [0, level + 3.005, 0], [10.94, 0.043, 8.64], 0xd7d2bf, 'plaster');
  for (const x of [-3.95, -1.99, 0, 1.99, 3.95])
    box(lab, [x, 2.969, 0], [0.024, 0.022, 8.62], 0x9f9e8f, 'paint');
  for (const z of [-3.16, -0.37, 2.42])
    box(lab, [0, 2.969, z], [10.93, 0.022, 0.024], 0xaaa699, 'paint');
  // North wall links the familiar sofa, calendar, board and monitor shelves.
  architecture(lab, [0, 1.52, -4.35], [11.05, 3.04, 0.13], cream);
  architecture(lab, [0, 1.52, 4.35], [11.05, 3.04, 0.13], 0xcfcbbb);
  for (const z of [-4.24, 4.24]) box(lab, [0, 0.095, z], [10.95, 0.18, 0.07], 0x817b69, 'timber');
  for (const y of [0.48, 2.88])
    architecture(lab, [-5.49, y, 0], [0.14, y < 1 ? 0.96 : 0.48, 8.7], cream);
  for (const z of [-4.2, -1.43, 1.43, 4.2]) {
    architecture(lab, [-5.49, 1.8, z], [0.16, 1.72, 0.18], 0x8e9186, 'metal');
  }
  block(-5.49, 0, 0.18, 8.7, 0, 3.05);
  for (const y of [1.02, 1.91, 2.61]) box(lab, [-5.48, y, 0], [0.15, 0.043, 8.5], 0x797f77, 'metal');
  for (let i = 0; i < 14; i++)
    box(lab, [-5.37, 2.63 - i * 0.045, -0.02], [0.18, 0.021, 8.28], 0xb2ac92, 'metal').rotation.z = 0.27;
  for (const [y, z, h, d] of [
    [1.52, 0.68, 3.04, 7.35],
    [1.52, -4.1, 3.04, 0.5],
    [2.73, -3.39, 0.62, 1.04]
  ])
    architecture(lab, [5.5, y, z], [0.15, h, d], 0xc0bfae);
  for (const z of [-3.94, -2.99]) box(lab, [5.47, 1.17, z], [0.23, 2.35, 0.074], dark);
  // A pink fabric partition encloses the wash nook.
  architecture(lab, [-3.28, 1.33, 3.53], [0.09, 2.66, 1.54], 0xbfb7a5);
  beam(lab, [-5.34, 2.43, 2.71], [-3.28, 2.43, 2.71], 0.026, 0.026, steel);
  for (let i = 0; i < 12; i++) {
    const x = -5.3 + i * 0.091;
    box(
      lab,
      [x, 1.29, 2.7 + Math.cos(i * 1.9) * 0.045],
      [0.104, 2.12, 0.041],
      i % 2 ? 0xb7898a : 0xaa777a,
      'cloth',
      0.02
    );
  }
  block(-4.81, 2.7, 1.08, 0.1, 0, 2.43);
  box(lab, [-4.76, 0.23, 3.85], [1.2, 0.44, 0.7], 0xc1c6b6, 'ceramic', 0.07);
  tube(
    lab,
    [
      [-5.28, 0.45, 4.1],
      [-5.28, 1.98, 4.1],
      [-5.09, 2.01, 4.03]
    ],
    0.019,
    steel
  );
  softSeat(lab, -3.19, -3.3);
  block(-3.19, -3.3, 2.46, 0.92, 0, 1.03);
  table(lab, -3.07, 0, -1.42, 1.96, 0.44, 0.91);
  block(-3.07, -1.42, 1.96, 0.91, 0, 0.5);
  box(lab, [-3.09, 0.489, -1.47], [1.46, 0.009, 0.55], 0x718471, 'cloth');
  mug(lab, -3.53, 0.495, -1.47, 0xcbbd9f);
  mug(lab, -3.22, 0.495, -1.56, 0x668071);
  smallPhone(lab);
  upa(lab);
  wallClock(lab, -3.54, 2.35, -4.23);
  sign(lab, 'Orbital_calendar', [-4.47, 1.71, -4.22], [0.56, 0.78], 'calendar');
  sign(lab, 'Lab_whiteboard', [-1.99, 1.87, -4.22], [1.46, 0.84], 'board');
  // Everyday objects on the tea table, shoes at the sill, and a lab notice by the door.
  for (let i = 0; i < 3; i++) {
    const book = box(
      lab,
      [-3.85, 0.505 + i * 0.02, -1.19],
      [0.225, 0.018, 0.17],
      [0xbfb08e, 0x909886, 0xb39683][i],
      'cardboard',
      0.004
    );
    book.rotation.y = 0.13 * i;
  }
  for (const x of [4.88, 5.06]) oval(lab, [x, 0.045, -3.84], [0.063, 0.044, 0.139], 0x605546, 'cloth');
  sign(lab, 'Lab_notice', [4.15, 1.91, 4.235], [0.66, 0.9], 'notice');
  const note = lab.getObjectByName('Lab_notice');
  note.rotation.y = Math.PI;
  for (let i = 0; i < 3; i++) {
    const potX = -4.97 + i * 0.16;
    cyl(lab, [potX, 1.07, 1.9], 0.069, 0.05, 0.12, 0x9e7861, 'ceramic');
    for (let j = 0; j < 7; j++) {
      const t = j * 0.9;
      beam(
        lab,
        [potX, 1.12, 1.9],
        [potX + 0.072 * Math.sin(t), 1.21 + 0.057 * Math.cos(j), 1.9 + 0.074 * Math.cos(t)],
        0.005,
        0.002,
        0x556e49
      );
      const leaf = oval(
        lab,
        [potX + 0.073 * Math.sin(t), 1.22 + 0.057 * Math.cos(j), 1.9 + 0.074 * Math.cos(t)],
        [0.029, 0.048, 0.01],
        j % 2 ? 0x697e50 : 0x819567,
        'paint',
        10
      );
      leaf.rotation.z = -0.7 * Math.sin(t);
    }
  }
  box(lab, [-4.88, 0.985, 1.9], [0.77, 0.048, 0.29], wood, 'timber');
  // Fridge, workshop drawers, stacked parcels and a small transistor radio.
  box(lab, [-0.71, 0.66, -3.68], [0.81, 1.33, 0.72], 0xb8c0a6, 'metal', 0.055);
  block(-0.71, -3.68, 0.82, 0.75, 0, 1.38);
  for (const [y, h] of [
    [0.49, 0.89],
    [1.15, 0.37]
  ])
    box(lab, [-0.71, y, -3.296], [0.78, h, 0.075], 0xc7cbb5, 'paint', 0.035);
  for (let i = 0; i < 4; i++)
    box(
      lab,
      [-0.93 + i * 0.082, 0.88 + (i % 2) * 0.13, -3.25],
      [0.058, 0.075, 0.008],
      [0xbd746a, 0xbba25e, 0x7d8d94, 0xc3ad93][i]
    );
  cyl(lab, [-0.75, 1.42, -3.7], 0.18, 0.14, 0.2, 0x746b5d);
  for (let i = 0; i < 4; i++)
    beam(lab, [-0.77 + i * 0.032, 1.48, -3.7], [-0.82 + i * 0.07, 1.79, -3.74], 0.008, 0.01, steel);
  for (const [x, z, w, h, d, y] of [
    [0.13, -3.76, 0.71, 0.51, 0.58, 0],
    [0.14, -3.76, 0.65, 0.5, 0.55, 0.51],
    [0.19, -3.75, 0.56, 0.45, 0.51, 1.01],
    [0.77, -3.64, 0.56, 0.54, 0.62, 0],
    [-1.11, -2.76, 0.56, 0.4, 0.46, 0],
    [-1.1, -2.76, 0.5, 0.34, 0.42, 0.4]
  ]) {
    carton(lab, x, y, z, w, h, d);
    block(x, z, w, d, y, y + h);
  }
  const radio = group(lab, 'Portable_radio', [-1.1, 0.78, -2.76]);
  box(radio, [0, 0.1, 0], [0.35, 0.2, 0.14], 0x965e53, 'paint', 0.025);
  box(radio, [-0.068, 0.107, 0.071], [0.14, 0.1, 0.008], 0x443e37);
  ring(radio, [0.085, 0.107, 0.075], 0.043, 0.008, dark);
  beam(radio, [0.1, 0.18, -0.02], [0.2, 0.58, -0.02], 0.003, 0.002, steel);
  table(lab, 0.72, 0, -1.52, 1.94, 0.92, 0.9);
  block(0.72, -1.52, 1.94, 0.92, 0, 1.0);
  phoneWave(lab);
  mug(lab, 1.35, 0.965, -1.68, 0xddd5af);
  const bench = group(lab, 'Computer_bench');
  table(bench, 3.46, 0, -2.74, 2.78, 0.76, 0.88);
  block(3.46, -2.74, 2.78, 0.88, 0, 0.82);
  crt(bench, 3.45, 0.81, -2.87, 0.68);
  keyboard(bench, 3.38, 0.807, -2.41);
  crt(bench, 4.21, 0.805, -2.92, 0.46, 0x777a6e);
  box(bench, [2.51, 1.06, -2.84], [0.25, 0.49, 0.47], 0xa2a697, 'metal', 0.018);
  for (let i = 0; i < 3; i++) box(bench, [2.51, 1.16 + i * 0.035, -2.595], [0.19, 0.012, 0.008], dark);
  chair(lab, 3.45, 0, -1.89, Math.PI);
  block(3.45, -1.89, 0.52, 0.54, 0, 1.1);
  const headset = group(lab, 'Time_leap_headset', [2.34, 0.82, -2.43], { interact: 'leap' });
  ring(headset, [0, 0.14, 0], 0.121, 0.016, 0x3b4145);
  for (const s of [-1, 1]) oval(headset, [s * 0.113, 0.079, 0.011], [0.039, 0.065, 0.053], 0x5e6460);
  tube(
    headset,
    [
      [0.125, 0.07, 0],
      [0.27, 0.01, 0.07],
      [0.35, -0.09, 0.11]
    ],
    0.006,
    dark
  );
  const nix = group(lab, 'Divergence_meter', [1.07, 1.035, -1.18], { print: 'meter' });
  box(nix, [0, 0.015, 0], [0.4, 0.048, 0.12], 0x50614b, 'metal');
  for (let i = 0; i < 8; i++) {
    oval(nix, [(i - 3.5) * 0.043, 0.069, 0], [0.018, 0.046, 0.018], 0xc7884c, 'metal');
  }
  shelves(lab, 3.1, 0, -4.0, 3.68, 2.67, 0.53);
  block(3.1, -4.0, 3.7, 0.52, 0, 2.7);
  for (let l = 0; l < 4; l++)
    for (let i = 0; i < 10; i++)
      book(
        lab,
        1.44 + i * 0.097,
        0.145 + l * 0.612,
        -3.94,
        0.24 + rng() * 0.22,
        [0x846f62, 0x6d7564, 0xc0b59b, 0x666f70][i % 4],
        0.074
      );
  for (let i = 0; i < 3; i++) crt(lab, 3.07 + i * 0.6, 0.79, -4.0, 0.49, i % 2 ? 0x837a66 : 0x68746f);
  for (let i = 0; i < 4; i++) carton(lab, 2.76 + i * 0.53, 1.43, -4.0, 0.45, 0.34, 0.38);
  fan(lab, 4.31, 2.08, -4.0);
  tube(
    lab,
    [
      [3.46, 0.76, -2.4],
      [3.52, 0.29, -2.27],
      [3.87, 0.055, -2.15],
      [4.42, 0.046, -2.3],
      [4.85, 0.1, -3.32]
    ],
    0.012,
    dark
  );
  // Galley under the south windows, enamel sink, faucet, hotplate and kettle.
  box(lab, [1.04, 0.43, 3.85], [3.6, 0.86, 0.78], 0x939889, 'metal');
  block(1.04, 3.85, 3.6, 0.78, 0, 0.94);
  box(lab, [1.04, 0.89, 3.85], [3.72, 0.085, 0.84], 0xb6b9a8, 'metal', 0.024);
  for (let i = 0; i < 5; i++) {
    box(lab, [-0.38 + i * 0.7, 0.46, 3.443], [0.65, 0.72, 0.036], 0xb6b3a1, 'paint', 0.012);
    box(lab, [-0.25 + i * 0.7, 0.72, 3.417], [0.19, 0.027, 0.026], steel, 'metal');
  }
  box(lab, [0.18, 0.938, 3.84], [0.64, 0.026, 0.48], 0x7b8986, 'metal', 0.032);
  tube(
    lab,
    [
      [0.18, 0.94, 4.09],
      [0.18, 1.2, 4.09],
      [0.18, 1.23, 3.93],
      [0.18, 1.16, 3.91]
    ],
    0.018,
    steel
  );
  for (const x of [1.41, 1.88]) {
    const pot = ring(lab, [x, 0.953, 3.8], 0.15, 0.012, dark);
    pot.rotation.x = Math.PI / 2;
  }
  cyl(lab, [1.77, 1.07, 3.79], 0.14, 0.17, 0.22, 0x939f89, 'metal');
  ring(lab, [1.77, 1.24, 3.79], 0.12, 0.014, dark);
  mug(lab, -0.43, 0.936, 3.68, 0xe1d5bd);
  for (const x of [-2.8, 2.5]) {
    beam(lab, [x, 3.01, -0.53], [x, 2.68, -0.53], 0.012, 0.012, dark);
    cyl(lab, [x, 2.58, -0.53], 0.065, 0.255, 0.16, 0x889487, 'metal', 28);
    oval(lab, [x, 2.499, -0.53], [0.193, 0.014, 0.193], 0xffefba, 'lamp');
  }
  // Concrete stairs make a continuous walk from shop to lab to fenced roof.
  for (const [p, level, dir] of [
    [shop, -3.2, 1],
    [lab, 0, -1]
  ]) {
    const start = dir === 1 ? 3.0 : -3.0,
      end = -start,
      cx = dir === 1 ? 5.88 : 6.81;
    floor(cx - 0.46, cx + 0.46, -3, 3, dir === 1 ? level + 3.2 : level, dir === 1 ? level : level + 3.2);
    floor(
      5.36,
      7.23,
      Math.min(start, dir === 1 ? 4.38 : -4.38),
      Math.max(start, dir === 1 ? 4.38 : -4.38),
      level
    );
    floor(
      5.36,
      7.23,
      Math.min(end, dir === 1 ? -4.38 : 4.38),
      Math.max(end, dir === 1 ? -4.38 : 4.38),
      level + 3.2
    );
    for (let i = 0; i < 20; i++) {
      const z = start + ((end - start) * (i + 0.5)) / 20;
      box(roof, [cx, level + (i + 0.5) * 0.16 - 0.068, z], [0.9, 0.14, 0.312], 0x7c8177, 'metal');
    }
    for (const [z, y] of [
      [start, level],
      [end, level + 3.2]
    ]) {
      box(roof, [6.34, y - 0.075, z * 1.23], [1.8, 0.15, 1.42], 0xa09b87, 'metal');
    }
    beam(roof, [cx + 0.45, level + 0.92, start], [cx + 0.45, level + 4.12, end], 0.029, 0.029, steel);
    for (let i = 0; i < 8; i++) {
      const t = i / 7,
        z = start + (end - start) * t;
      beam(
        roof,
        [cx + 0.45, level + t * 3.2, z],
        [cx + 0.45, level + t * 3.2 + 0.92, z],
        0.017,
        0.017,
        steel
      );
    }
  }
  floor(5.38, 7.24, 3, 4.37, 3.2);
  for (const x of [-5.47, 5.47]) {
    const zEnd = x < 0 ? 4.35 : 2.82;
    for (let z = -4.31; z < zEnd; z += 0.6) beam(roof, [x, 3.2, z], [x, 4.2, z], 0.013, 0.013, steel);
    beam(roof, [x, 4.21, -4.32], [x, 4.21, zEnd], 0.026, 0.026, steel);
    block(x, (zEnd - 4.32) / 2, 0.1, zEnd + 4.32, 3.2, 4.25);
  }
  for (const z of [-4.34, 4.34]) {
    for (let x = -5.4; x < 5.46; x += 0.6) beam(roof, [x, 3.2, z], [x, 4.2, z], 0.013, 0.013, steel);
    beam(roof, [-5.45, 4.21, z], [5.45, 4.21, z], 0.026, 0.026, steel);
    block(0, z, 11, 0.09, 3.2, 4.25);
  }
  for (const x of [-3.96, -2.63])
    for (const z of [-2.95, -1.68]) beam(roof, [x, 3.2, z], [x, 3.87, z], 0.07, 0.07, steel);
  cyl(roof, [-3.29, 4.5, -2.32], 0.95, 0.95, 1.49, 0xb2aaa0, 'metal', 36);
  for (const y of [3.84, 4.49, 5.19]) {
    const band = ring(roof, [-3.29, y, -2.32], 0.96, 0.018, steel);
    band.rotation.x = Math.PI / 2;
  }
  block(-3.29, -2.32, 1.95, 1.95, 3.2, 5.25);
  for (let i = 0; i < 2; i++) {
    box(roof, [-3.7 + i * 1.19, 3.6, 2.75], [1.04, 0.78, 0.68], 0xb8b7a6, 'metal', 0.035);
    for (let j = 0; j < 11; j++)
      box(roof, [-4.13 + i * 1.19 + j * 0.073, 3.6, 3.1], [0.02, 0.55, 0.012], steel);
    block(-3.7 + i * 1.19, 2.75, 1.04, 0.68, 3.2, 4);
  }
  bicycle(roof, 2.6, 3.2, 2.92);
  block(2.6, 2.92, 1.38, 0.54, 3.2, 4.2);
  architecture(shop, [0, -1.68, -4.35], [11, 3.02, 0.14], 0xadafa4);
  architecture(shop, [-5.48, -1.65, 0], [0.14, 3.1, 8.7], 0xb4b5a6);
  architecture(shop, [5.48, -1.65, -0.64], [0.14, 3.1, 7.38], 0xb4b5a6);
  for (const x of [-4.16, 3.92]) {
    architecture(shop, [x, -1.7, 4.33], [2.66, 3, 0.13], 0x718378, 'metal');
  }
  box(shop, [0, -0.43, 4.42], [10.9, 0.48, 0.16], 0x668179, 'metal');
  sign(shop, 'Braun_storefront', [0, -0.45, 4.52], [4.8, 0.48], 'shop');
  for (const [x, z] of [
    [-3.87, -3.55],
    [-0.76, -3.55],
    [2.53, -3.55]
  ]) {
    shelves(shop, x, -3.2, z, 2.92, 2.31, 0.64);
    block(x, z, 2.92, 0.64, -3.2, -0.7);
    for (let row = 0; row < 3; row++)
      for (let i = 0; i < 4; i++)
        crt(
          shop,
          x - 1.09 + i * 0.71,
          -3.05 + row * 0.55,
          z,
          0.6,
          [0x8b796d, 0xb6b4a0, 0x676e68][(row + i) % 3]
        );
  }
  table(shop, -2.76, -3.2, 0.46, 2.62, 0.87, 0.98);
  block(-2.76, 0.46, 2.62, 0.98, -3.2, -2.25);
  const tv = group(shop, 'Forty_two_inch_CRT', [-2.78, -2.28, 0.45], { interact: 'crt' });
  crt(tv, 0, 0, 0, 1.09, 0x70695d);
  sign(shop, 'Repair_ticket', [-2.8, -1.13, -4.23], [1.28, 0.46], 'repair');
  floor(-6.91, 7.2, 4.2, 6.9, -3.2);
  box(town, [0, -3.31, 5.72], [18, 0.2, 3.25], 0x8b8d87, 'concrete');
  for (let i = 0; i < 33; i++) {
    const a = (TAU * i) / 33,
      rad = 16 + rng() * 26,
      x = Math.sin(a) * rad,
      z = Math.cos(a) * rad,
      w = 2.4 + rng() * 4,
      d = 2.4 + rng() * 4,
      h = 4 + rng() * 10;
    box(
      town,
      [x, -3.51 + h * 0.5, z],
      [w, h, d],
      new T.Color().setHSL(0.1 + rng() * 0.48, 0.07, 0.43 + rng() * 0.2).getHex(),
      'city'
    );
    box(town, [x, -3.42 + h, z], [w + 0.13, 0.16, d + 0.13], 0x807e77);
    if (i % 3 === 0) box(town, [x, -3.06 + h, z], [w * 0.48, 0.74, d * 0.43], 0x92938a, 'metal');
  }
  for (let i = 0; i < 4; i++) {
    const x = -12 + i * 8;
    beam(town, [x, -3.35, 8], [x, 4.7, 8], 0.097, 0.07, 0x716958);
    if (i < 3)
      for (let w = 0; w < 3; w++)
        tube(
          town,
          [
            [x, 4.45, 8 + w * 0.16],
            [x + 4, 4.15, 8 + w * 0.16],
            [x + 8, 4.45, 8 + w * 0.16]
          ],
          0.013,
          0x484b4c
        );
  }
  const spawns = {
    okabe: [-1.18, 0, -1.45, 0.35],
    mayuri: [-4.35, 0, -1.99, 0.78],
    daru: [3.45, 0, -1.85, Math.PI - 0.24],
    kurisu: [1.78, 0, -0.85, -0.56],
    moeka: [1.4, -3.2, -1.65, 0.32],
    luka: [-3.95, -3.2, 2.12, 0.95],
    faris: [2.52, -3.2, 2.18, -0.78],
    suzuha: [1.46, 3.2, 2.27, -0.32]
  };
  root.userData.spawns = spawns;
  return scene;
}
