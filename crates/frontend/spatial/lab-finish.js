import * as T from 'three';
import { Sky } from 'three/addons/objects/Sky.js';
import parquet from './art/parquet.webp';
import plaster from './art/plaster.webp';
import facade from './art/facade.webp';

function surface(width, height, paint) {
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  paint(canvas.getContext('2d'), width, height);
  const texture = new T.CanvasTexture(canvas);
  texture.colorSpace = T.SRGBColorSpace;
  return texture;
}

function printed(kind) {
  return surface(768, 448, (g, w, h) => {
    const text = (s, x, y, size, color = '#304440', font = 'monospace') => {
      g.fillStyle = color;
      g.font = size + 'px ' + font;
      g.fillText(s, x, y);
    };
    g.fillStyle = '#e4e0cd';
    g.fillRect(0, 0, w, h);
    if (kind === 'board') {
      g.fillStyle = '#dce2d5';
      g.fillRect(0, 0, w, h);
      text('PhoneWave   #008', 36, 68, 44, '#31433e', 'cursive');
      text('D-Mail  ↤  t − 48 h', 50, 144, 39, '#534039', 'cursive');
      for (let i = 0; i < 3; i++) {
        g.beginPath();
        for (let x = 0; x < 650; x += 4) {
          const y = 257 + i * 11 + Math.sin(x / 93 + i) * 43 * Math.sin(x / 210);
          x ? g.lineTo(x + 48, y) : g.moveTo(x + 48, y);
        }
        g.strokeStyle = ['#415e66', '#a65d4e', '#526b55'][i];
        g.lineWidth = 3;
        g.stroke();
      }
      text('17:43  →  ?', 242, 387, 42, '#985346', 'cursive');
    } else if (kind === 'calendar') {
      g.fillStyle = '#7d9a9d';
      g.fillRect(0, 0, w, 124);
      text('AUGUST   2010', 38, 85, 65, '#f6edda');
      for (let i = 0; i < 31; i++) {
        const x = 53 + (i % 7) * 102,
          y = 184 + Math.floor(i / 7) * 53;
        text(String(i + 1), x, y, 40, i % 7 === 0 ? '#9d524a' : '#394444');
        if (i === 12) {
          g.beginPath();
          g.ellipse(x + 23, y - 13, 38, 24, -0.2, 0, 7);
          g.strokeStyle = '#ae5046';
          g.lineWidth = 4;
          g.stroke();
        }
      }
    } else if (kind === 'notice') {
      g.fillStyle = '#b2b7a5';
      g.fillRect(0, 0, w, 145);
      text('未来ガジェット', 65, 103, 70, '#40524a', 'serif');
      text('研究所', 62, 274, 115, '#39473f', 'serif');
      text('MEMBERS ONLY', 70, 370, 44, '#936251');
    } else if (kind === 'shop') {
      g.fillStyle = '#48655e';
      g.fillRect(0, 0, w, h);
      text('ブラウン管工房', 65, 279, 97, '#f5ebd0', 'serif');
    } else {
      text('修 理 受 付 票', 56, 140, 66);
      text('42型  /  通電確認', 56, 277, 48);
      text('08.13  17:43', 56, 367, 39, '#a44f3d');
    }
  });
}

export function dressRoom(scene, building, request) {
  const loader = new T.TextureLoader();
  function load(asset, rx, ry) {
    const tex = loader.load(new URL(asset, import.meta.url).href, request);
    tex.colorSpace = T.SRGBColorSpace;
    tex.wrapS = tex.wrapT = T.RepeatWrapping;
    tex.repeat.set(rx, ry);
    tex.anisotropy = 4;
    return tex;
  }
  const tile = surface(128, 128, (g) => {
    g.fillStyle = '#deded0';
    g.fillRect(0, 0, 128, 128);
    g.fillStyle = '#c6cec3';
    g.fillRect(0, 0, 64, 64);
    g.fillRect(64, 64, 64, 64);
    g.strokeStyle = '#9caa9d';
    g.lineWidth = 1;
    for (const edge of [0, 64, 128]) {
      g.beginPath();
      g.moveTo(edge, 0);
      g.lineTo(edge, 128);
      g.moveTo(0, edge);
      g.lineTo(128, edge);
      g.stroke();
    }
  });
  tile.wrapS = tile.wrapT = T.RepeatWrapping;
  tile.repeat.set(11, 9);
  const maps = {
    concrete: load(plaster, 8, 7),
    linoleum: tile,
    floor: load(parquet, 3.5, 2.5),
    timber: load(parquet, 0.66, 0.4),
    city: load(facade, 2, 2),
    plaster: load(plaster, 4, 2)
  };
  const seen = new Set();
  building.traverse((obj) => {
    if (!obj.isMesh) return;
    obj.receiveShadow = true;
    obj.castShadow = !['floor', 'concrete', 'linoleum', 'plaster', 'city', 'lamp'].includes(
      obj.material.name
    );
    for (const m of [].concat(obj.material)) {
      if (seen.has(m)) continue;
      seen.add(m);
      if (maps[m.name]) m.map = maps[m.name];
      if (m.name.startsWith('print_')) {
        m.map = printed(m.name.slice(6));
        m.map.flipY = true;
        m.color.set(0xffffff);
      }
      if (m.name === 'crt') {
        m.emissive.setHex(0x325344);
        m.emissiveIntensity = 0.32;
      }
      m.needsUpdate = true;
    }
  });
  const skyDome = new Sky();
  skyDome.scale.setScalar(180);
  skyDome.renderOrder = -2;
  scene.add(skyDome);
  const atmosphere = skyDome.material.uniforms;
  atmosphere.turbidity.value = 4;
  atmosphere.rayleigh.value = 1.3;
  atmosphere.mieCoefficient.value = 0.0045;
  atmosphere.mieDirectionalG.value = 0.83;
  const hemi = new T.HemisphereLight(0xffedce, 0x969386, 1.52);
  scene.add(hemi);
  const sun = new T.DirectionalLight(0xffdda0, 2.08);
  sun.position.set(-7, 5.4, 2.6);
  scene.add(sun);
  sun.castShadow = true;
  sun.shadow.mapSize.set(2048, 2048);
  Object.assign(sun.shadow.camera, { left: -9, right: 9, top: 8, bottom: -8, near: 0.5, far: 35 });
  sun.shadow.camera.updateProjectionMatrix();
  sun.shadow.normalBias = 0.018;
  sun.shadow.bias = -0.00027;
  sun.shadow.radius = 3;
  const fill = new T.DirectionalLight(0xd4e8f4, 0.65);
  fill.position.set(3.5, 2, -6);
  scene.add(fill);
  function bulb(x, y, z, color, intensity, distance) {
    const l = new T.PointLight(color, intensity, distance, 2);
    l.position.set(x, y, z);
    scene.add(l);
    return l;
  }
  bulb(-2.8, 2.46, -0.53, 0xffdfaa, 5, 8);
  bulb(2.5, 2.46, -0.53, 0xffe8c8, 5, 8);
  bulb(-2.8, -0.6, 0.68, 0xbbdddf, 7, 10);
  const pulse = bulb(0.49, 1.16, -1.41, 0xadf3a4, 0, 3);
  const shadowMap = surface(128, 128, (g) => {
    const grad = g.createRadialGradient(64, 64, 0, 64, 64, 64);
    grad.addColorStop(0, 'rgba(21,17,13,.64)');
    grad.addColorStop(0.5, 'rgba(27,22,16,.46)');
    grad.addColorStop(1, 'rgba(32,27,20,0)');
    g.fillStyle = grad;
    g.fillRect(0, 0, 128, 128);
  });
  const shadowMaterial = new T.MeshBasicMaterial({
    map: shadowMap,
    transparent: true,
    depthWrite: false,
    toneMapped: false
  });
  function shadow(x, y, z, w, d) {
    const plane = new T.Mesh(new T.PlaneGeometry(w, d), shadowMaterial);
    plane.rotation.x = -Math.PI / 2;
    plane.position.set(x, y + 0.009, z);
    scene.add(plane);
    return plane;
  }
  for (const [x, y, z, w, d] of [
    [-3.19, 0, -3.27, 3.25, 1.75],
    [-3.07, 0, -1.42, 2.65, 1.42],
    [0.72, 0, -1.52, 2.57, 1.51],
    [3.46, 0, -2.74, 3.42, 1.62],
    [-2.76, -3.2, 0.46, 3.37, 1.6],
    [-3.29, 3.2, -2.32, 2.81, 2.32]
  ])
    shadow(x, y, z, w, d);
  const shafts = new T.Group();
  scene.add(shafts);
  const shaftsMaterial = new T.MeshBasicMaterial({
    color: 0xffedb0,
    opacity: 0.085,
    transparent: true,
    depthWrite: false,
    blending: T.AdditiveBlending
  });
  for (let i = 0; i < 11; i++) {
    const stripe = new T.Mesh(new T.PlaneGeometry(3.66, 0.072), shaftsMaterial);
    stripe.rotation.set(-Math.PI / 2, 0, -0.24);
    stripe.position.set(-3.51, 0.018, -0.82 + i * 0.195);
    shafts.add(stripe);
  }
  const palettes = {
    alpha: [0xefcb9c, 0xffcf91, 0xffe6c8, 1],
    echo: [0x766b88, 0x8e99e6, 0xacc7db, 0.53],
    beta: [0xdd9e80, 0xffa779, 0xcfb9aa, 0.84],
    gate: [0x9fcddd, 0xffe9ad, 0xe5eee0, 1.15]
  };
  return {
    shadow,
    set(line, armed) {
      const [sky, light, ambient, k] = palettes[line] || palettes.alpha;
      scene.background = new T.Color(sky);
      scene.fog = new T.FogExp2(sky, 0.017);
      const elevation = { alpha: 0.18, echo: -0.026, beta: 0.057, gate: 0.27 }[line] ?? 0.18;
      atmosphere.sunPosition.value.set(-0.94, elevation, 0.34).normalize().multiplyScalar(450000);
      sun.color.setHex(light);
      sun.intensity = 2 * k;
      hemi.color.setHex(ambient);
      hemi.intensity = 1.48 * k + 0.18;
      shafts.visible = k > 0.7;
      pulse.intensity = armed ? 2.6 : 0;
    }
  };
}

export function displayPanel(parent, position, width, height, paint) {
  const map = surface(512, 160, paint),
    material = new T.MeshBasicMaterial({ map, toneMapped: false });
  const mesh = new T.Mesh(new T.PlaneGeometry(width, height), material);
  mesh.position.set(...position);
  parent.add(mesh);
  return (draw) => {
    const c = map.image;
    draw(c.getContext('2d'), c.width, c.height);
    map.needsUpdate = true;
  };
}
