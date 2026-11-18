import * as T from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { stage, seeded } from './stage.js';
import archive from './art/archive.webp';
import music from './art/music.webp';
import gallery from './art/gallery.webp';
import studio from './art/studio.webp';
import tools from './art/tools.webp';
import search from './art/search.webp';
import sun from './art/sun.webp';
import sky from './art/deep-sky.webp';

const bodies = [
  ['archive', archive, [-6.75, 0.46, 0.4], 1.17, 0x78b8e0],
  ['music', music, [4.65, 0.76, -2.5], 0.92, 0xe7bda0],
  ['gallery', gallery, [-3.18, -0.52, 3.75], 0.65, 0xbf93cb],
  ['studio', studio, [6.26, -0.42, 2.78], 0.75, 0xeaab84],
  ['tools', tools, [-3.54, 1.0, -4.04], 0.61, 0x8ad4ce],
  ['search', search, [1.77, -0.73, 4.54], 0.51, 0x9dd3ed],
  ['lab', sun, [0, 0, 0], 1.12, 0xffc883]
];
function glowTexture() {
  const c = document.createElement('canvas');
  c.width = c.height = 128;
  const g = c.getContext('2d');
  const gradient = g.createRadialGradient(64, 64, 0, 64, 64, 64);
  for (const [stop, alpha] of [
    [0, 1],
    [0.12, 0.9],
    [0.28, 0.51],
    [0.51, 0.19],
    [0.78, 0.038],
    [1, 0]
  ])
    gradient.addColorStop(stop, 'rgba(255,255,255,' + alpha + ')');
  g.fillStyle = gradient;
  g.fillRect(0, 0, 128, 128);
  return new T.CanvasTexture(c);
}
function dust(scene, map) {
  const rand = seeded(83107),
    tau = 2 * Math.PI;
  function scatter(count, coords, color, size, opacity) {
    const points = [],
      colors = [];
    for (let i = 0; i < count; i++) {
      points.push(...coords(i));
      colors.push(...color(i).toArray());
    }
    const geo = new T.BufferGeometry();
    geo.setAttribute('position', new T.Float32BufferAttribute(points, 3));
    geo.setAttribute('color', new T.Float32BufferAttribute(colors, 3));
    const mat = new T.PointsMaterial({
      map,
      size,
      color: 0xffffff,
      opacity,
      vertexColors: true,
      transparent: true,
      depthWrite: false,
      blending: T.AdditiveBlending
    });
    const p = new T.Points(geo, mat);
    scene.add(p);
    return p;
  }
  const starColor = () =>
    new T.Color().setHSL(0.54 + rand() * 0.25, 0.16 + rand() * 0.35, 0.59 + rand() * 0.39);
  scatter(
    6200,
    () => {
      const a = rand() * tau,
        h = rand() * 2 - 1,
        r = 82 * Math.sqrt(1 - h * h);
      return [r * Math.cos(a), 82 * h, r * Math.sin(a)];
    },
    starColor,
    0.38,
    1
  );
  scatter(
    340,
    () => {
      const a = rand() * tau,
        h = rand() * 2 - 1,
        r = 80 * Math.sqrt(1 - h * h);
      return [r * Math.cos(a), 80 * h, r * Math.sin(a)];
    },
    starColor,
    0.85,
    1
  );
  let radius = 0;
  function arm(i) {
    radius = 0.65 + Math.pow(rand(), 0.66) * 16;
    const a = ((i % 4) * tau) / 4 + radius * 0.35 + (rand() + rand() - 1) * (0.36 + 0.05 * radius);
    return [
      Math.cos(a) * radius,
      (rand() + rand() - 1) * (0.28 + radius * 0.038) - 0.35,
      Math.sin(a) * radius * 0.76
    ];
  }
  const armColor = () =>
    new T.Color(0xe8ad87)
      .lerp(new T.Color(0x8696de), Math.min(1, radius / 10))
      .lerp(new T.Color(0xf7eee1), rand() * 0.7);
  scatter(13800, arm, armColor, 0.065, 0.87);
  scatter(1100, arm, () => new T.Color().setHSL(0.58 + rand() * 0.24, 0.39, 0.38), 1.23, 0.092);
}
function band(inner, outer, color, opacity) {
  const g = new T.RingGeometry(inner, outer, 144);
  const m = new T.Mesh(
    g,
    new T.MeshBasicMaterial({ color, side: T.DoubleSide, transparent: true, opacity, depthWrite: false })
  );
  m.rotation.x = -Math.PI / 2;
  return m;
}
function atmosphere(radius, tint) {
  return new T.Mesh(
    new T.SphereGeometry(radius * 1.055, 40, 26),
    new T.ShaderMaterial({
      uniforms: { tint: { value: new T.Color(tint) } },
      vertexShader:
        'varying vec3 n; varying vec3 eye; void main(){vec4 mv=modelViewMatrix*vec4(position,1.);n=normalize(normalMatrix*normal);eye=normalize(-mv.xyz);gl_Position=projectionMatrix*mv;}',
      fragmentShader:
        'uniform vec3 tint;varying vec3 n;varying vec3 eye;void main(){float f=pow(1.-clamp(dot(normalize(n),normalize(eye)),0.,1.),4.);gl_FragColor=vec4(tint,f*.56);}',
      transparent: true,
      depthWrite: false,
      blending: T.AdditiveBlending
    })
  );
}
export async function mount(host, emit, _walker, signal) {
  const s = stage(host, signal, emit),
    { scene, camera, canvas } = s;
  canvas.setAttribute('aria-label', '三维星系，拖动旋转，滚轮或双指缩放，点星球前往目的地');
  scene.background = new T.Color(0x070b1a);
  camera.fov = 43;
  camera.updateProjectionMatrix();
  camera.position.set(0, 7.1, 15);
  camera.lookAt(0, 0, 0);
  const controls = new OrbitControls(camera, canvas);
  controls.enablePan = false;
  controls.enableDamping = false;
  controls.minPolarAngle = 0.48;
  controls.maxPolarAngle = 1.52;
  controls.minDistance = 8;
  controls.maxDistance = 32;
  controls.rotateSpeed = 0.55;
  controls.addEventListener('change', s.request);
  const skyMap = new T.TextureLoader().load(new URL(sky, import.meta.url).href, s.request);
  skyMap.colorSpace = T.SRGBColorSpace;
  const dome = new T.Mesh(
    new T.SphereGeometry(100, 48, 28),
    new T.MeshBasicMaterial({ map: skyMap, side: T.BackSide, toneMapped: false, depthWrite: false })
  );
  dome.renderOrder = -2;
  scene.add(dome);
  const map = glowTexture();
  dust(scene, map);
  scene.add(new T.HemisphereLight(0xd5e4ff, 0x22273d, 0.62));
  const starLight = new T.PointLight(0xffe9c4, 105, 38, 1.65);
  starLight.position.set(0.1, 1.1, 1);
  scene.add(starLight);
  const distant = new T.DirectionalLight(0xffefd9, 1.48);
  distant.position.set(-8, 4, 3);
  scene.add(distant);
  const rim = new T.DirectionalLight(0x8699e2, 1.05);
  rim.position.set(5, 2, -7);
  scene.add(rim);
  const labels = document.createElement('div');
  labels.className = 'orbit-labels';
  host.append(labels);
  const loader = new T.TextureLoader(),
    targets = [],
    worlds = [];
  for (const [id, asset, at, r, tint] of bodies) {
    const tex = loader.load(new URL(asset, import.meta.url).href, s.request);
    tex.colorSpace = T.SRGBColorSpace;
    const node = new T.Group();
    node.position.set(...at);
    scene.add(node);
    const surface =
      id === 'lab'
        ? new T.MeshBasicMaterial({ map: tex, color: 0xffebba })
        : new T.MeshStandardMaterial({ map: tex, roughness: 0.88, metalness: 0.025 });
    const globe = new T.Mesh(new T.SphereGeometry(r, 72, 44), surface);
    globe.rotation.z = id === 'music' ? 0.28 : 0.13;
    globe.rotation.y = id === 'archive' ? 1.84 : 0.5;
    globe.userData.port = id;
    targets.push(globe);
    node.add(globe);
    if (id === 'lab') {
      for (const [scale, color, opacity] of [
        [6.5, 0xffac4e, 0.8],
        [11.0, 0xfb7c44, 0.35]
      ]) {
        const corona = new T.Sprite(
          new T.SpriteMaterial({
            map,
            color,
            opacity,
            transparent: true,
            blending: T.AdditiveBlending,
            depthWrite: false
          })
        );
        corona.scale.setScalar(scale);
        node.add(corona);
      }
      const plane = new T.Group();
      plane.rotation.set(0.2, 0.22, -0.16);
      node.add(plane);
      for (let i = 0; i < 17; i++) {
        const ri = 1.37 + i * 0.055;
        plane.add(
          band(ri, ri + 0.012 + (i % 4) * 0.004, i % 3 ? 0xd89d63 : 0xf9d6a0, 0.14 + (i % 4) * 0.035)
        );
      }
    } else {
      node.add(atmosphere(r, tint));
      if (id === 'music' || id === 'tools') {
        const rings = new T.Group();
        rings.rotation.set(0.47, -0.19, -0.34);
        node.add(rings);
        for (let i = 0; i < 27; i++) {
          if (i === 17 || i === 18) continue;
          const ri = r * (1.26 + i * 0.029);
          rings.add(band(ri, ri + r * 0.022, i % 5 === 0 ? 0xb399ac : tint, 0.4 + (i % 3) * 0.13));
        }
      }
      const curve = new T.EllipseCurve(
        0,
        0,
        Math.hypot(at[0], at[2]),
        Math.hypot(at[0], at[2]) * 0.78,
        0,
        Math.PI * 2
      );
      const g = new T.BufferGeometry().setFromPoints(
        curve.getPoints(160).map((p) => new T.Vector3(p.x, -0.48, p.y))
      );
      scene.add(
        new T.LineLoop(g, new T.LineBasicMaterial({ color: tint, transparent: true, opacity: 0.15 }))
      );
    }
    const label = document.createElement('button');
    label.type = 'button';
    label.className = 'orbit-tag' + (id === 'lab' ? ' orbit-tag-sun' : '');
    label.dataset.port = id;
    label.addEventListener('click', () => emit('planet', id), { signal });
    labels.append(label);
    worlds.push({ id, at, r, label, node, globe });
  }
  const ray = new T.Raycaster(),
    pointer = new T.Vector2();
  let down;
  canvas.addEventListener(
    'pointerdown',
    (e) => {
      down = [e.clientX, e.clientY];
    },
    { signal }
  );
  canvas.addEventListener(
    'pointerup',
    (e) => {
      if (!down || Math.hypot(e.clientX - down[0], e.clientY - down[1]) > 7) return;
      const rect = canvas.getBoundingClientRect();
      pointer.set(
        (2 * (e.clientX - rect.left)) / rect.width - 1,
        1 - (2 * (e.clientY - rect.top)) / rect.height
      );
      ray.setFromCamera(pointer, camera);
      const hit = ray.intersectObjects(targets, false)[0];
      if (hit) emit('planet', hit.object.userData.port);
    },
    { signal }
  );
  const position = new T.Vector3();
  let narrow,
    elapsed = 0;
  function layout(dt, reduced) {
    const slim = camera.aspect < 0.93;
    if (slim !== narrow) {
      narrow = slim;
      camera.position.set(0, slim ? 13.5 : 7.1, slim ? 18.0 : 15);
      controls.update();
    }
    elapsed += reduced ? 0 : dt;
    const w = canvas.clientWidth,
      h = canvas.clientHeight;
    for (const [i, b] of worlds.entries()) {
      const [x, y, z] = b.at;
      b.node.position.set(
        x * (slim ? 0.37 : 1),
        y * (slim ? 1.84 : 1) + 0.07 * Math.sin(elapsed * 0.13 + i),
        z * (slim ? 1.24 : 1)
      );
      b.node.scale.setScalar(slim ? 0.83 : 1);
      if (!reduced) b.globe.rotation.y += dt * (b.id === 'lab' ? 0.023 : 0.041);
      position.copy(b.node.position);
      position.y -= b.r * b.node.scale.y * 1.18;
      position.project(camera);
      b.label.style.transform =
        'translate(' +
        ((position.x * 0.5 + 0.5) * w).toFixed(1) +
        'px,' +
        ((0.5 - position.y * 0.5) * h).toFixed(1) +
        'px) translateX(-50%)';
      b.label.hidden = position.z > 1 || Math.abs(position.x) > 1.06 || Math.abs(position.y) > 1.05;
    }
    return false;
  }
  s.start(layout);
  return {
    sync(raw) {
      const st = JSON.parse(raw);
      for (const b of worlds) {
        b.label.textContent = st.labels[b.id];
        b.label.classList.toggle('picked', st.selected === b.id);
      }
      s.request();
    },
    dispose() {
      controls.dispose();
      labels.remove();
      s.dispose();
    }
  };
}
