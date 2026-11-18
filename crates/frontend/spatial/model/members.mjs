import * as T from 'three';
import { group, mesh, box, oval, cyl, beam, tube, ring, loft, patch, TAU, random } from './kit.mjs';

const SKIN = 0xf1c6ac,
  SHADE = 0xc49385,
  INK = 0x342c32;
const members = [
  { id: 'okabe', h: 1.77, hair: 0x272b2b, eyes: 0x786b56, shirt: 0xd4c8af, pants: 0x48443b },
  { id: 'mayuri', h: 1.52, hair: 0x222f36, eyes: 0x638caa, shirt: 0xa7cdde, pants: 0x363b46 },
  { id: 'daru', h: 1.65, hair: 0x554438, eyes: 0x615642, shirt: 0x85885e, pants: 0x565f62 },
  { id: 'kurisu', h: 1.6, hair: 0x964837, eyes: 0x706395, shirt: 0xeee9df, pants: 0x35343d },
  { id: 'moeka', h: 1.67, hair: 0xb98a58, eyes: 0x817148, shirt: 0xe6dcd3, pants: 0x484450 },
  { id: 'luka', h: 1.61, hair: 0x283039, eyes: 0x827185, shirt: 0xf2e9d9, pants: 0x873a41 },
  { id: 'faris', h: 1.53, hair: 0xdb8298, eyes: 0xa36a78, shirt: 0x39323e, pants: 0x36333a },
  { id: 'suzuha', h: 1.63, hair: 0xa28b65, eyes: 0xb29549, shirt: 0x4a565b, pants: 0x424b51 }
];

function lock(p, base, mid, end, width, color) {
  const points = [];
  for (let i = 0; i <= 6; i++) {
    const t = i / 6,
      k = 1 - t;
    const [x, y, z] = base.map((a, n) => k * k * a + 2 * k * t * mid[n] + t * t * end[n]);
    const w = width * (1 - 0.97 * t) * (0.8 + 0.35 * Math.sin(t * Math.PI));
    points.push([y, w, w * 0.43, x, z]);
  }
  return loft(p, [0, 0, 0], points.reverse(), color, { sides: 7 });
}
function eye(head, s, iris, male) {
  const x = 0.046 * s,
    y = 0.135,
    tall = male ? 0.013 : 0.018;
  const eyePts = [
    [-0.03, 0],
    [-0.021, 0.012],
    [0.002, tall],
    [0.022, 0.012],
    [0.033, 0.004],
    [0.018, -0.01],
    [-0.006, -tall],
    [-0.023, -0.009]
  ];
  const local = (dx, dy, offset = 0) => {
    const px = x + dx * s;
    return [px, y + dy, 0.089 * Math.sqrt(Math.max(0.02, 1 - (px / 0.099) ** 2)) + 0.003 + offset];
  };
  patch(
    head,
    eyePts.map((e) => local(...e)),
    0xfff5e6
  );
  oval(head, local(0.001, -0.001, 0.003), [0.013, tall * 0.95, 0.0035], iris);
  oval(head, local(0.001, 0.001, 0.007), [0.0058, tall * 0.66, 0.0015], INK);
  oval(head, local(-0.003, 0.006, 0.007), [0.0034, 0.0038, 0.001], 0xfffbed);
  tube(
    head,
    [
      local(-0.03, 0, 0.003),
      local(-0.014, 0.012, 0.004),
      local(0.009, tall, 0.004),
      local(0.026, 0.011, 0.003),
      local(0.035, 0.005, 0.003)
    ],
    0.0023,
    INK
  );
  tube(head, [local(-0.022, 0.036), local(0.002, 0.039), local(0.026, 0.03)], 0.0033, 0x60443e);
}
function face(p, c) {
  const male = c.id === 'okabe' || c.id === 'daru';
  loft(
    p,
    [0, 0, 0],
    [
      [0.005, 0.027, 0.045, 0, 0.018],
      [0.022, 0.053, 0.063, 0, 0.011],
      [0.063, 0.079, 0.078],
      [0.105, 0.095, 0.087],
      [0.152, 0.099, 0.089],
      [0.196, 0.095, 0.087],
      [0.228, 0.073, 0.069],
      [0.245, 0.025, 0.028]
    ],
    SKIN,
    { sides: 32 }
  );
  for (const s of [-1, 1]) {
    oval(p, [s * 0.095, 0.112, -0.003], [0.018, 0.029, 0.013], SKIN);
    oval(p, [s * 0.104, 0.113, 0.003], [0.008, 0.016, 0.009], SHADE);
    eye(p, s, c.eyes, male);
  }
  loft(
    p,
    [0, 0.071, 0.073],
    [
      [0, 0.006, 0.002],
      [0.008, 0.012, 0.017],
      [0.014, 0.014, 0.025],
      [0.048, 0.005, 0.01]
    ],
    SKIN,
    { sides: 10 }
  );
  tube(
    p,
    [
      [-0.02, 0.05, 0.077],
      [-0.006, 0.047, 0.084],
      [0.006, 0.046, 0.084],
      [0.021, 0.05, 0.078]
    ],
    0.0018,
    0x965c59
  );
  if (c.id === 'okabe')
    for (let i = 0; i < 11; i++) {
      const x = (i - 5) * 0.004;
      beam(p, [x, 0.026, 0.068], [x + 0.002, 0.031, 0.07], 0.0008, 0.0005, 0x66594d, 'paint', 4);
    }
}
function hair(p, c) {
  const id = c.id,
    rng = random(id.charCodeAt(0) * 79);
  // A hair shell with an open face, a short fringe and a lower nape.
  const v = [],
    idx = [],
    n = 48,
    rows = 12;
  // Follow the actual forehead and occiput all the way to the hairline.
  const profile = [
    [0.043, 0.096, 0.102],
    [0.11, 0.111, 0.11],
    [0.16, 0.116, 0.112],
    [0.205, 0.11, 0.105],
    [0.232, 0.088, 0.089],
    [0.251, 0.062, 0.063],
    [0.267, 0.025, 0.028],
    [0.272, 0.002, 0.003]
  ];
  for (let j = 0; j <= rows; j++)
    for (let i = 0; i <= n; i++) {
      const a = (TAU * i) / n,
        front = (1 + Math.cos(a)) / 2;
      const end = 0.054 + 0.149 * front ** 2.6 + (id === 'okabe' ? 0.012 : 0.004) * Math.sin(a * 2);
      const y = 0.272 + ((end - 0.272) * j) / rows;
      const k = Math.max(
          1,
          profile.findIndex((p) => p[0] >= y)
        ),
        lo = profile[k - 1],
        hi = profile[k];
      const t = T.MathUtils.clamp((y - lo[0]) / (hi[0] - lo[0]), 0, 1);
      const w = T.MathUtils.lerp(lo[1], hi[1], t),
        d = T.MathUtils.lerp(lo[2], hi[2], t);
      v.push(w * Math.sin(a), y, d * Math.cos(a) - 0.004);
      if (j && i) {
        const k = j * (n + 1) + i;
        idx.push(k - n - 2, k - 1, k, k - n - 2, k, k - n - 1);
      }
    }
  const g = new T.BufferGeometry();
  g.setAttribute('position', new T.Float32BufferAttribute(v, 3));
  g.setIndex(idx);
  g.computeVertexNormals();
  const cap = loft(
    p,
    [0, 0, 0],
    [
      [0.224, 0.066, 0.06],
      [0.256, 0.025, 0.025]
    ],
    c.hair
  );
  cap.geometry.dispose();
  cap.geometry = g;
  const rgb = new T.Color(c.hair),
    col = new Float32Array(v.length);
  for (let i = 0; i < col.length; i += 3) rgb.toArray(col, i);
  g.setAttribute('color', new T.Float32BufferAttribute(col, 3));
  g.setAttribute('uv', new T.Float32BufferAttribute(new Float32Array((v.length / 3) * 2), 2));
  const length = {
    kurisu: 0.42,
    moeka: 0.3,
    mayuri: 0.13,
    luka: 0.13,
    suzuha: 0.1,
    faris: 0.1,
    daru: 0.09,
    okabe: 0.07
  }[id];
  for (let i = 0; i < 21; i++) {
    const a = 0.92 + (i * (TAU - 1.84)) / 20,
      f = Math.sin(a),
      b = Math.cos(a);
    const shade = new T.Color(c.hair).multiplyScalar(0.76 + 0.46 * rng()).getHex();
    lock(
      p,
      [f * 0.058, 0.228, b * 0.057 - 0.01],
      [f * 0.127, 0.148, b * 0.123 - 0.015],
      [
        f * (id === 'okabe' || id === 'daru' ? 0.14 : 0.113) + 0.012 * Math.sin(3 * a),
        0.142 - length * (0.74 + 0.33 * rng()),
        b * 0.105 - 0.024
      ],
      0.033,
      shade
    );
  }
  for (let i = 0; i < 9; i++) {
    const x = (i - 4) * 0.021;
    const swept = id === 'okabe' ? 0.046 : id === 'kurisu' ? -0.029 : 0.018;
    const shade = new T.Color(c.hair).multiplyScalar(0.86 + 0.52 * rng()).getHex();
    lock(
      p,
      [x * 0.52, 0.255, 0.029],
      [x + swept * 0.45, 0.221, 0.122],
      [
        x + swept,
        0.151 + 0.025 * Math.cos(i * 0.92) + (id === 'okabe' ? 0.024 : 0),
        0.108 - (0.036 * Math.abs(i - 4)) / 4
      ],
      0.03,
      shade
    );
  }
  if (id === 'okabe') {
    for (let i = 0; i < 11; i++) {
      const a = (i * TAU) / 11;
      lock(
        p,
        [0.038 * Math.sin(a), 0.241, 0.035 * Math.cos(a)],
        [0.08 * Math.sin(a) - 0.018, 0.267, 0.071 * Math.cos(a) - 0.018],
        [0.117 * Math.sin(a) - 0.03, 0.218 + 0.041 * rng(), 0.101 * Math.cos(a) - 0.035],
        0.029,
        i % 3 ? 0x282e30 : 0x373d3d
      );
    }
  }
  if (id === 'kurisu' || id === 'moeka') {
    for (let i = 0; i < 13; i++) {
      const a = Math.PI * 0.43 + (i * Math.PI * 1.14) / 12;
      lock(
        p,
        [0.084 * Math.sin(a), 0.204, 0.086 * Math.cos(a) - 0.008],
        [0.146 * Math.sin(a), 0.037, 0.154 * Math.cos(a) - 0.043],
        [
          0.146 * Math.sin(a) + 0.015 * Math.cos(i),
          id === 'kurisu' ? -0.31 : -0.19,
          0.146 * Math.cos(a) - 0.058
        ],
        0.045,
        new T.Color(c.hair).multiplyScalar(0.76 + 0.31 * (i % 3)).getHex()
      );
    }
  }
  if (id === 'suzuha' || id === 'faris')
    for (const s of [-1, 1]) {
      const radius = id === 'faris' ? 0.044 : 0.024,
        count = id === 'faris' ? 7 : 13;
      for (let j = 0; j < count; j++) {
        const t = j / count,
          x = s * (0.095 + 0.055 * Math.sin(t * 3.0) + 0.009 * Math.cos(j * 2.8));
        oval(
          p,
          [x, 0.145 - t * 0.31, -0.051 + 0.033 * t],
          [radius * (1 - 0.48 * t), 0.033, radius * (1 - 0.5 * t)],
          c.hair,
          'paint',
          12
        );
      }
      box(
        p,
        [s * 0.125, -0.13, -0.019],
        [0.032, 0.017, 0.028],
        id === 'faris' ? 0xb33c61 : 0xa65341,
        'paint',
        0.004
      );
    }
  if (id === 'mayuri' || id === 'daru') {
    const color = id === 'mayuri' ? 0xafcbda : 0xd29b44;
    oval(p, [0, 0.231, -0.012], [0.119, id === 'mayuri' ? 0.051 : 0.063, 0.111], color);
    if (id === 'daru') oval(p, [0, 0.214, 0.086], [0.103, 0.009, 0.07], color);
    else {
      const rim = ring(p, [0, 0.207, 0], 0.094, 0.01, 0xcbdde1);
      rim.rotation.x = Math.PI / 2;
      oval(p, [-0.077, 0.223, 0.064], [0.029, 0.011, 0.015], 0x526f9d);
    }
  }
  if (id === 'faris') {
    for (const s of [-1, 1]) {
      patch(
        p,
        [
          [s * 0.048, 0.224, 0.012],
          [s * 0.105, 0.309, -0.015],
          [s * 0.113, 0.208, 0.016]
        ],
        0x342d38
      );
      patch(
        p,
        [
          [s * 0.069, 0.229, 0.02],
          [s * 0.1, 0.285, -0.001],
          [s * 0.103, 0.22, 0.024]
        ],
        0xe99ba6
      );
    }
    for (let i = 0; i < 12; i++) {
      const a = (i / 11 - 0.5) * 2.6;
      oval(p, [Math.sin(a) * 0.091, 0.203 + Math.cos(a) * 0.06, 0.041], [0.018, 0.017, 0.011], 0xf7ecde);
    }
  }
  if (id === 'moeka' || id === 'daru') {
    for (const s of [-1, 1]) {
      const x = s * 0.048;
      const pts = [
        [x - 0.04, 0.154, 0.087],
        [x + 0.036, 0.154, 0.087],
        [x + 0.032, 0.12, 0.091],
        [x - 0.031, 0.12, 0.091],
        [x - 0.04, 0.154, 0.087]
      ];
      for (let i = 1; i < pts.length; i++)
        beam(p, pts[i - 1], pts[i], 0.0024, 0.0024, id === 'moeka' ? 0x663746 : 0x373737);
      beam(p, [s * 0.082, 0.148, 0.086], [s * 0.108, 0.14, -0.022], 0.002, 0.002, INK);
    }
    beam(p, [-0.012, 0.146, 0.093], [0.012, 0.146, 0.093], 0.002, 0.002, INK);
  }
  if (id === 'luka') {
    beam(p, [0.038, 0.199, 0.078], [0.071, 0.181, 0.07], 0.004, 0.004, 0xe0cba6);
    oval(p, [0.05, 0.195, 0.08], [0.008, 0.007, 0.003], 0xe8dfc8);
  }
}
function shoe(p, x, y, z, c) {
  oval(p, [x, y + 0.057, z + 0.028], [0.063, 0.057, 0.123], c, 'paint');
  box(p, [x, y + 0.015, z + 0.028], [0.126, 0.026, 0.235], 0x332d2c, 'paint', 0.011);
}
function hand(p, at, side) {
  const [x, y, z] = at;
  oval(p, at, [0.03, 0.044, 0.018], SKIN);
  for (let f = 0; f < 4; f++) {
    const dx = (f - 1.5) * 0.012;
    beam(
      p,
      [x + dx, y - 0.022, z],
      [x + dx, y - 0.063 + 0.008 * Math.abs(f - 1.4), z + 0.006],
      0.007,
      0.0047,
      SKIN,
      'paint',
      7
    );
  }
  beam(
    p,
    [x - side * 0.021, y + 0.004, z],
    [x - side * 0.04, y - 0.031, z + 0.012],
    0.011,
    0.006,
    SKIN,
    'paint',
    8
  );
}
function member(scene, c) {
  const id = c.id,
    wide = id === 'daru' ? 1.55 : 1,
    male = id === 'daru' || id === 'okabe';
  const root = group(scene, id, [0, 0, 0], { actor: id });
  root.scale.setScalar(c.h / 1.66);
  const seat = id === 'daru',
    dy = seat ? -0.34 : 0;
  const body = group(root, id + '_body', [0, dy, 0]);
  const shoulder = (male ? 0.196 : 0.161) * wide,
    waist = 0.124 * wide;
  const shirt = c.shirt,
    pants = c.pants,
    boot = male ? 0x463b33 : 0x503e46;
  for (const s of [-1, 1]) {
    const x = s * 0.086 * wide;
    if (seat) {
      beam(root, [x, 0.54, -0.02], [x, 0.49, 0.32], 0.112, 0.091, pants, 'paint', 16);
      beam(root, [x, 0.49, 0.32], [x, 0.105, 0.32], 0.089, 0.062, pants, 'paint', 16);
      shoe(root, x, 0, 0.34, boot);
    } else {
      const hemHeight = { mayuri: 0.56, moeka: 0.56, faris: 0.63, luka: 0.17 }[id];
      const legs = id === 'kurisu' ? 0x46404b : id === 'mayuri' || id === 'faris' ? 0x453f49 : pants;
      loft(
        root,
        [x, 0, 0],
        [
          [0.085, 0.049, 0.05],
          [0.26, 0.059, 0.061],
          [0.47, 0.074, 0.071],
          [0.62, 0.079, 0.093],
          [0.84, 0.096, 0.11]
        ]
          .filter(([y]) => !hemHeight || y < hemHeight)
          .concat(hemHeight ? [[hemHeight + 0.055, 0.068, 0.075]] : []),
        legs,
        { sides: 16 }
      );
      shoe(root, x, 0, 0, boot);
      if (id === 'kurisu' || id === 'suzuha')
        loft(
          root,
          [x, 0, 0],
          [
            [0.072, 0.057, 0.067],
            [0.21, 0.061, 0.062],
            [0.24, 0.061, 0.061]
          ],
          boot,
          { sides: 16 }
        );
    }
  }
  loft(
    body,
    [0, 0, 0],
    [
      [id === 'kurisu' ? 0.854 : 0.77, waist * 1.07, 0.105],
      [0.87, waist, 0.097],
      [1.02, waist * 0.94, 0.093],
      [1.17, shoulder * 0.86, 0.108],
      [1.3, shoulder, 0.09],
      [1.355, 0.076, 0.067]
    ],
    shirt,
    { sides: 26 }
  );
  cyl(body, [0, 1.38, 0], 0.047, 0.055, 0.11, SKIN);
  const head = group(body, id + '_head', [0, 1.406, 0], { joint: 'head' });
  face(head, c);
  hair(head, c);
  if (id === 'okabe' || id === 'kurisu') {
    const coat = id === 'okabe' ? 0xe4e4db : 0x997b5a,
      hem = id === 'okabe' ? 0.46 : 0.85;
    const sections = [
      [hem, waist + 0.085, 0.165],
      [hem + 0.06, waist + 0.081, 0.161],
      [0.94, waist + 0.047, 0.133],
      [1.08, waist + 0.042, 0.13],
      [1.19, shoulder * 0.95, 0.13],
      [1.285, shoulder + 0.01, 0.113],
      [1.33, shoulder * 0.7, 0.093]
    ];
    const v = [],
      idx = [],
      pieces = 32;
    for (const [row, [y, w, d]] of sections.entries())
      for (let i = 0; i <= pieces; i++) {
        const a = 0.26 + (i * (TAU - 0.52)) / pieces,
          fold = 1 + 0.028 * Math.sin(7 * a + row * 0.23);
        v.push(
          w * Math.sin(a) * fold,
          y + (row === 0 ? 0.012 * Math.cos(5 * a) : 0),
          d * Math.cos(a) * fold
        );
        if (row && i) {
          const k = row * (pieces + 1) + i;
          idx.push(k - pieces - 2, k - pieces - 1, k, k - pieces - 2, k, k - 1);
        }
      }
    const shell = new T.BufferGeometry();
    shell.setAttribute('position', new T.Float32BufferAttribute(v, 3));
    shell.setIndex(idx);
    shell.computeVertexNormals();
    mesh(body, shell, coat, [0, 0, 0], 'cloth');
    for (const side of [-1, 1]) {
      patch(
        body,
        [
          [side * 0.049, 1.347, 0.075],
          [side * 0.14, 1.284, 0.093],
          [side * 0.1, 1.248, 0.122],
          [side * 0.129, 1.213, 0.116],
          [side * 0.041, 1.056, 0.13]
        ],
        id === 'okabe' ? 0xf5f1e3 : 0xb29b7a
      );
      const px = side * (waist * 0.8);
      patch(
        body,
        [
          [px - 0.037, hem + 0.167, 0.144],
          [px + 0.037, hem + 0.167, 0.144],
          [px + 0.031, hem + 0.109, 0.144],
          [px - 0.031, hem + 0.106, 0.144]
        ],
        coat
      );
    }
    for (const y of [0.86, 0.99, 1.12])
      if (y > hem + 0.02) oval(body, [0.059, y, 0.135], [0.005, 0.005, 0.002], 0x766d60);
    if (id === 'kurisu') {
      patch(
        body,
        [
          [-0.016, 1.322, 0.096],
          [0.016, 1.322, 0.096],
          [0.011, 1.292, 0.107],
          [-0.009, 1.292, 0.107]
        ],
        0x813630
      );
      patch(
        body,
        [
          [-0.008, 1.294, 0.108],
          [0.008, 1.294, 0.108],
          [0.021, 1.098, 0.138],
          [0, 1.06, 0.138],
          [-0.017, 1.1, 0.138]
        ],
        0xa74543
      );
      for (const side of [-1, 1])
        loft(
          body,
          [side * 0.081, 0, 0.006],
          [
            [0.695, 0.085, 0.113],
            [0.81, 0.097, 0.127],
            [0.86, 0.076, 0.112]
          ],
          0x332e32,
          { sides: 20 }
        );
      loft(
        body,
        [0, 0, 0],
        [
          [0.768, 0.151, 0.115],
          [0.844, 0.153, 0.12],
          [0.871, 0.143, 0.114]
        ],
        0x645244
      );
      box(body, [0, 0.856, 0.121], [0.035, 0.024, 0.008], 0xaf946b, 'metal');
    } else {
      for (const side of [-1, 1])
        patch(
          body,
          [
            [side * 0.023, 1.345, 0.075],
            [side * 0.068, 1.334, 0.069],
            [side * 0.054, 1.293, 0.1],
            [side * 0.013, 1.324, 0.096]
          ],
          0xd8cbb3
        );
      for (const y of [1.18, 1.08, 0.98]) oval(body, [0, y, 0.114], [0.004, 0.004, 0.002], 0x988b79);
    }
  }
  if (id === 'mayuri' || id === 'faris' || id === 'luka' || id === 'moeka') {
    const hem = id === 'luka' ? 0.17 : id === 'faris' ? 0.63 : 0.56;
    loft(
      body,
      [0, 0, 0],
      [
        [hem, 0.197, 0.151],
        [hem + 0.05, 0.194, 0.147],
        [0.82, 0.159, 0.117],
        [0.99, waist, 0.096]
      ],
      id === 'mayuri' ? shirt : id === 'faris' ? 0x332e37 : pants,
      { sides: 40, pleats: 0.045 }
    );
    if (id === 'faris') {
      loft(
        body,
        [0, 0, 0],
        [
          [hem - 0.015, 0.201, 0.151],
          [hem + 0.015, 0.201, 0.152]
        ],
        0xf2e8da,
        { sides: 40, pleats: 0.065 }
      );
      patch(
        body,
        [
          [-0.087, 1.26, 0.093],
          [0.087, 1.26, 0.093],
          [0.08, 0.974, 0.099],
          [0.132, 0.714, 0.121],
          [0, 0.678, 0.154],
          [-0.132, 0.714, 0.121],
          [-0.08, 0.974, 0.099]
        ],
        0xf2e9da
      );
    }
    if (id === 'luka') {
      patch(
        body,
        [
          [-0.068, 1.349, 0.06],
          [-0.112, 1.291, 0.083],
          [0.091, 1.063, 0.108],
          [0.117, 1.102, 0.104]
        ],
        0xdfd9d1
      );
      loft(
        body,
        [0, 0, 0],
        [
          [0.91, 0.139, 0.107],
          [1.015, 0.13, 0.102]
        ],
        0x9b4147
      );
    }
    if (id === 'mayuri') {
      for (const s of [-1, 1])
        tube(
          body,
          [
            [s * 0.093, 1.315, 0.068],
            [s * 0.102, 1.247, 0.098],
            [s * 0.066, 1.215, 0.108],
            [0, 1.208, 0.111]
          ],
          0.01,
          0x465350
        );
    }
  }
  if (id === 'suzuha') {
    tube(
      body,
      [
        [0, 0.79, 0.106],
        [0, 1.06, 0.106],
        [0, 1.31, 0.082]
      ],
      0.003,
      0xd7cdb6
    );
    cyl(body, [0, 1.347, 0], 0.062, 0.07, 0.052, shirt);
  }
  for (const s of [-1, 1]) {
    const arm = group(body, id + '_arm_' + (s > 0 ? 'R' : 'L'), [s * shoulder, 1.285, 0], {
      joint: 'arm'
    });
    const bent = seat || (id === 'moeka' && s > 0) || (id === 'okabe' && s < 0);
    let elbow = [s * 0.047, -0.238, 0.008],
      wrist = bent ? [s * 0.019, -0.242, 0.194] : [s * 0.05, -0.44, 0.041];
    if (id === 'kurisu') {
      elbow = [s * 0.032, -0.251, 0.07];
      wrist = [-s * 0.209, -0.177 + (s > 0 ? 0.029 : 0), 0.155];
    }
    if (id === 'mayuri' && s < 0) {
      elbow = [-0.085, -0.158, 0.032];
      wrist = [-0.144, 0.015, 0.084];
    }
    if (id === 'okabe' && s < 0) {
      elbow = [-0.098, -0.168, 0.05];
      wrist = [-0.032, 0.068, 0.135];
    }
    if (id === 'daru') {
      elbow = [s * 0.02, -0.218, 0.061];
      wrist = [-s * 0.013, -0.184, 0.264];
    }
    const sleeve = id === 'okabe' ? 0xe9e8de : id === 'kurisu' ? 0xa18a6b : shirt;
    const short = id === 'daru' || id === 'mayuri' || id === 'faris';
    oval(arm, [0, -0.011, 0], [0.072 * wide, 0.068, 0.071], sleeve);
    beam(arm, [0, 0, 0], elbow, 0.071 * wide, 0.049 * wide, short ? SKIN : sleeve, 'paint', 18);
    beam(
      arm,
      [0, 0.005, 0],
      short ? [s * 0.033, -0.13, 0.006] : elbow,
      0.074 * wide,
      0.056 * wide,
      sleeve,
      'paint',
      16
    );
    beam(arm, elbow, wrist, 0.048 * wide, 0.031, short ? SKIN : sleeve, 'paint', 14);
    oval(arm, elbow, [0.049, 0.05, 0.047], short ? SKIN : sleeve);
    if (!short) {
      const start = new T.Vector3(...wrist).lerp(new T.Vector3(...elbow), 0.13);
      beam(arm, start.toArray(), wrist, 0.039, 0.037, id === 'kurisu' ? 0x66574b : sleeve, 'cloth', 16);
    }
    hand(arm, [wrist[0], wrist[1] - 0.026, wrist[2]], s);
    if (id === 'moeka' && s > 0)
      box(
        arm,
        [wrist[0], wrist[1] + 0.024, wrist[2] + 0.013],
        [0.049, 0.083, 0.011],
        0x7d4e67,
        'metal',
        0.005
      );
  }
  if (id === 'daru') {
    box(body, [0, 0.91, 0.162], [0.201, 0.082, 0.05], 0x4c4440, 'paint', 0.02);
    oval(body, [-0.11, 1.215, 0.13], [0.04, 0.04, 0.008], 0xd3b15a);
  }
  return root;
}
export function createMembers() {
  const s = new T.Scene();
  s.name = 'Lab_members_001_008';
  for (const c of members) member(s, c);
  return s;
}
