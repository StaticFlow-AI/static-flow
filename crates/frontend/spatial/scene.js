import {
  BackSide,
  BoxGeometry,
  BufferGeometry,
  Color,
  DirectionalLight,
  HemisphereLight,
  PlaneGeometry,
  PCFShadowMap,
  EdgesGeometry,
  Float32BufferAttribute,
  Fog,
  Group,
  LineBasicMaterial,
  LineSegments,
  Mesh,
  MeshBasicMaterial,
  MeshLambertMaterial,
  PointLight,
  Points,
  PointsMaterial,
  ShaderMaterial,
  SphereGeometry,
  Vector3
} from 'three';
import { stage, seeded } from './stage.js';
import { chunkGeometry, heldGeometry } from './mesh.js';
import { character, hand } from './actors.js';

function atmosphere(scene, seed) {
  const gradient = new ShaderMaterial({
    side: BackSide,
    depthWrite: false,
    fog: false,
    uniforms: { top: { value: new Color('#648fc2') }, horizon: { value: new Color('#eadbbf') } },
    vertexShader:
      'varying float alt; void main(){alt=normalize(position).y;gl_Position=projectionMatrix*modelViewMatrix*vec4(position,1.);}',
    fragmentShader:
      'varying float alt; uniform vec3 top; uniform vec3 horizon; void main(){gl_FragColor=vec4(mix(horizon,top,smoothstep(-.08,.74,alt)),1.);\n#include <colorspace_fragment>\n}'
  });
  const dome = new Mesh(new SphereGeometry(127, 20, 10), gradient);
  dome.renderOrder = -3;
  scene.add(dome);
  const clouds = new Group(),
    random = seeded(seed);
  const cloudMat = new MeshBasicMaterial({ color: 0xfaf1da, fog: false });
  const cube = new BoxGeometry();
  for (let i = 0; i < 41; i++) {
    const a = random() * Math.PI * 2,
      d = 54 + random() * 41;
    const cloud = new Mesh(cube, cloudMat);
    cloud.position.set(Math.cos(a) * d, 21 + random() * 13, Math.sin(a) * d);
    cloud.scale.set(3 + random() * 13, 0.7 + random() * 1.15, 2 + random() * 5);
    clouds.add(cloud);
  }
  dome.add(clouds);
  const sun = new Mesh(cube, new MeshBasicMaterial({ color: 0xffe1a3, fog: false }));
  sun.scale.set(5.8, 5.8, 1.1);
  dome.add(sun);
  const moon = new Mesh(cube, new MeshBasicMaterial({ color: 0xe0ecf0, fog: false }));
  moon.scale.set(4.1, 4.1, 1.1);
  dome.add(moon);
  const stars = [];
  for (let i = 0; i < 170; i++) {
    const a = random() * Math.PI * 2,
      b = random() * 1.5;
    stars.push(Math.cos(a) * Math.cos(b) * 105, Math.sin(b) * 105, Math.sin(a) * Math.cos(b) * 105);
  }
  const geo = new BufferGeometry();
  geo.setAttribute('position', new Float32BufferAttribute(stars, 3));
  const starMat = new PointsMaterial({ size: 0.28, color: 0xe7eada, fog: false, transparent: true });
  dome.add(new Points(geo, starMat));
  scene.fog = new Fog(0xb9cbd0, 41, 71);
  const noon = new Color('#74a7ca'),
    dusk = new Color('#b6909b'),
    night = new Color('#233453');
  const haze = new Color('#e0d8bb'),
    sunset = new Color('#edc59d'),
    navy = new Color('#656d89');
  const axis = new Vector3();
  return (x, y, z, time, line) => {
    const angle = (time * Math.PI * 2) / 960 + 0.8,
      day = Math.sin(angle),
      light = Math.max(0, Math.min(1, day * 1.8 + 0.3));
    dome.position.set(x, y, z);
    gradient.uniforms.top.value.copy(night).lerp(dusk, light).lerp(noon, Math.max(0, day));
    gradient.uniforms.horizon.value.copy(navy).lerp(sunset, light).lerp(haze, Math.max(0, day));
    if (line === 'echo') {
      gradient.uniforms.top.value.lerp(night, 0.45);
      gradient.uniforms.horizon.value.lerp(navy, 0.38);
    }
    if (line === 'beta') {
      gradient.uniforms.top.value.lerp(dusk, 0.4);
      gradient.uniforms.horizon.value.lerp(sunset, 0.58);
    }
    scene.fog.color.copy(gradient.uniforms.horizon.value);
    axis
      .set(Math.cos(angle) * 78, day * 88, -43)
      .normalize()
      .multiplyScalar(100);
    sun.position.copy(axis);
    moon.position.copy(axis).multiplyScalar(-1);
    sun.lookAt(dome.position.x, dome.position.y, dome.position.z);
    moon.rotation.copy(sun.rotation);
    cloudMat.color.setRGB(0.46 + light * 0.52, 0.49 + light * 0.45, 0.56 + light * 0.29);
    clouds.rotation.y = (time % 4000) * 0.00045;
    starMat.opacity = 1 - light;
    return 0.58 + 0.42 * light;
  };
}
export function worldScene(host, game, catalog, texture, signal, report) {
  const view = stage(host, signal, report),
    { engine, scene, camera, canvas } = view;
  camera.rotation.order = 'YXZ';
  scene.add(camera);
  canvas.setAttribute('aria-label', '方块世界第一人称视图');
  const material = new MeshLambertMaterial({ map: texture, vertexColors: true, alphaTest: 0.4 });
  const skyLight = new HemisphereLight(0xfff3db, 0xbdc7cb, 1.02);
  const sunlight = new DirectionalLight(0xffe0ad, 1.12);
  scene.add(skyLight, sunlight, sunlight.target);
  engine.shadowMap.enabled = true;
  engine.shadowMap.type = PCFShadowMap;
  engine.shadowMap.autoUpdate = false;
  sunlight.castShadow = true;
  sunlight.shadow.mapSize.setScalar(matchMedia('(pointer:coarse)').matches ? 512 : 1024);
  Object.assign(sunlight.shadow.camera, {
    left: -23,
    right: 23,
    top: 23,
    bottom: -23,
    near: 1,
    far: 110
  });
  sunlight.shadow.camera.updateProjectionMatrix();
  sunlight.shadow.bias = -0.0004;
  sunlight.shadow.normalBias = 0.025;
  const handMaterial = new MeshBasicMaterial({ map: texture, vertexColors: true });
  handMaterial.fog = false;
  const first = hand(handMaterial),
    grip = new Group();
  grip.add(first);
  view.overlay.add(grip);
  let heldItem,
    heldId = -1;
  function held(id) {
    if (id === heldId) return;
    heldId = id;
    if (heldItem) {
      grip.remove(heldItem);
      heldItem.geometry.dispose();
      heldItem = null;
    }
    if (!id) return;
    heldItem = new Mesh(heldGeometry(catalog.items[id], id), handMaterial);
    heldItem.position.set(-0.025, 0.17, -0.042);
    if (catalog.items[id].key.endsWith('_pick')) {
      heldItem.position.set(-0.072, 0.185, -0.036);
      heldItem.rotation.z = 0.65;
      heldItem.scale.setScalar(1.32);
    }
    heldItem.renderOrder = 9;
    grip.add(heldItem);
  }
  first.renderOrder = 8;
  const people = catalog.actors.map((a) => character(a, material));
  const contact = new PlaneGeometry(0.65, 0.47),
    shadowInk = new MeshBasicMaterial({
      color: 0x3e4337,
      transparent: true,
      opacity: 0.23,
      depthWrite: false
    });
  for (const p of people) {
    p.root.traverse((o) => {
      if (o.isMesh) {
        o.castShadow = true;
        o.receiveShadow = true;
      }
    });
    const shade = new Mesh(contact, shadowInk);
    shade.rotation.x = -Math.PI / 2;
    shade.position.y = 0.008;
    p.root.add(shade);
    scene.add(p.root);
  }
  const sky = atmosphere(scene, JSON.parse(game.snapshot()).seed);
  const outline = new LineSegments(
    new EdgesGeometry(new BoxGeometry(1, 1, 1)),
    new LineBasicMaterial({ color: 0x332d23, transparent: true, opacity: 0.78 })
  );
  outline.visible = false;
  scene.add(outline);
  let breaking = 0,
    aim = [],
    cx = Infinity,
    cz = Infinity,
    line = '',
    elapsed = 0,
    lastPose;
  const chunks = new Map(),
    dirty = new Set();
  const radius = matchMedia('(pointer:coarse)').matches ? 3 : 4;
  const coords = (k) => k.split(',').map(Number);
  const lights = Array.from({ length: 4 }, () => {
    const light = new PointLight(0xffe3b3, 0, 10, 2);
    scene.add(light);
    return light;
  });
  let lightCell = '';
  function updateLights(x, y, z) {
    const cell = [Math.floor(x), Math.floor(y), Math.floor(z)].join(',');
    if (lightCell === cell) return;
    lightCell = cell;
    const nearby = [];
    for (const mesh of chunks.values())
      for (const source of mesh.geometry.userData.lights) {
        const d = (source.x - x) ** 2 + (source.y - y - 1) ** 2 + (source.z - z) ** 2;
        if (d < source.range ** 2) nearby.push([d, source]);
      }
    nearby.sort((a, b) => a[0] - b[0]);
    lights.forEach((light, i) => {
      const source = nearby[i]?.[1];
      light.intensity = source?.power || 0;
      if (source) {
        light.position.set(source.x, source.y, source.z);
        light.color.setHex(source.color);
        light.distance = source.range;
      }
    });
  }
  function bake(x, z) {
    const k = x + ',' + z,
      old = chunks.get(k);
    lightCell = '';
    const g = chunkGeometry(game.region(x, z), x, z, catalog);
    if (old) {
      old.geometry.dispose();
      old.geometry = g;
    } else {
      const mesh = new Mesh(g, material);
      mesh.castShadow = true;
      mesh.receiveShadow = true;
      scene.add(mesh);
      chunks.set(k, mesh);
    }
    if (Math.abs(cx - x) < 2 && Math.abs(cz - z) < 2) engine.shadowMap.needsUpdate = true;
  }
  function stream(x, z) {
    if (cx === x && cz === z) return;
    cx = x;
    cz = z;
    game.trim(x, z);
    for (const [k, mesh] of chunks) {
      const [px, pz] = coords(k);
      if (Math.max(Math.abs(px - x), Math.abs(pz - z)) > radius) {
        mesh.geometry.dispose();
        scene.remove(mesh);
        chunks.delete(k);
      }
    }
    const queue = [];
    for (let dx = -radius; dx <= radius; dx++)
      for (let dz = -radius; dz <= radius; dz++) {
        if (dx * dx + dz * dz > (radius + 0.45) ** 2) continue;
        const k = x + dx + ',' + (z + dz);
        if (!chunks.has(k)) queue.push([dx * dx + dz * dz, k]);
      }
    for (const k of dirty) {
      const [px, pz] = coords(k);
      if (Math.max(Math.abs(px - x), Math.abs(pz - z)) > radius) dirty.delete(k);
    }
    queue.sort((a, b) => a[0] - b[0]);
    for (const [, k] of queue) dirty.add(k);
  }
  function flush() {
    const changes = game.changes();
    if (!changes.length) return false;
    const rebuild = new Set();
    for (let i = 0; i < changes.length; i += 3) {
      for (const x of [changes[i] - 1, changes[i] + 1])
        for (const z of [changes[i + 2] - 1, changes[i + 2] + 1]) {
          const k = Math.floor(x / 16) + ',' + Math.floor(z / 16);
          if (chunks.has(k)) rebuild.add(k);
        }
    }
    for (const k of rebuild) {
      dirty.delete(k);
      bake(...coords(k));
    }
    return true;
  }
  function focus(hit, progress) {
    aim = hit;
    breaking = progress;
    outline.visible = hit.length > 2;
    if (outline.visible) {
      const [lo, hi] = catalog.bounds[hit[0]];
      outline.position.set(...lo.map((n, i) => hit[1 + i] + (n + hi[i]) * 0.5));
      outline.scale.set(...lo.map((n, i) => hi[i] - n + 0.006));
      outline.material.color.setHex(progress > 0 ? 0xffcf7d : 0x423b31);
    }
  }
  function state(s) {
    people.forEach((p, i) => {
      p.root.position.y = s.actorY[i];
    });
    held(s.bag[s.bar[s.slot]] ? s.bar[s.slot] : 0);
    if (s.line !== line) {
      line = s.line;
      people.forEach((p, i) => {
        const id = catalog.actors[i].id;
        p.root.visible = !((id === 'mayuri' && line === 'echo') || (id === 'daru' && line === 'beta'));
      });
    }
  }
  let walkTime = 0,
    shadowX,
    shadowZ,
    sunStep;
  function frame(dt, pose, time, reduced, swing) {
    const [x, y, z, yaw, pitch] = pose;
    elapsed += dt;
    stream(Math.floor(x / 16), Math.floor(z / 16));
    // Generate neighbouring chunks incrementally and keep mesh and collision on the same seed.
    const k = dirty.values().next().value;
    if (k) {
      dirty.delete(k);
      bake(...coords(k));
    }
    const moving = lastPose && Math.hypot(x - lastPose[0], z - lastPose[2]) > 0.002;
    lastPose = pose;
    if (moving) walkTime += dt * 10;
    camera.position.set(
      x,
      y + 1.58 + (moving && pose[5] && !reduced ? Math.sin(walkTime) * 0.024 : 0),
      z
    );
    camera.rotation.set(pitch, yaw, 0, 'YXZ');
    const bob = reduced ? 0 : Math.sin(walkTime) * 0.012;
    const action = reduced ? 0 : Math.sin(Math.PI * Math.min(1, swing || breaking));
    grip.position.set(
      Math.min(0.44, 0.29 * camera.aspect),
      -0.26 + bob + action * 0.023,
      -0.51 - action * 0.08
    );
    grip.rotation.set(-0.66 - action * 0.6, -0.26, -0.41 + action * 0.2);
    grip.scale.setScalar(0.74);
    updateLights(x, y, z);
    const brightness = sky(x, y + 1.58, z, time, line);
    const px = Math.floor(x / 5) * 5,
      pz = Math.floor(z / 5) * 5,
      tick = Math.floor(time / 18);
    if (shadowX !== px || shadowZ !== pz || sunStep !== tick) {
      shadowX = px;
      shadowZ = pz;
      sunStep = tick;
      const a = ((tick * 18) / 960) * Math.PI * 2 + 0.8;
      sunlight.position.set(px + Math.cos(a) * 29, y + Math.abs(Math.sin(a)) * 28 + 4, pz - 32);
      sunlight.target.position.set(px, y, pz);
      engine.shadowMap.needsUpdate = true;
    }
    sunlight.intensity = (brightness - 0.58) * 2.35 + 0.09;
    skyLight.intensity = 0.68 + (0.37 * (brightness - 0.58)) / 0.42;
    if (!reduced)
      people.forEach((p, i) => {
        p.head.rotation.y = Math.sin(elapsed * 0.62 + i * 3) * 0.075;
        p.arms[0].rotation.x = -0.1 + Math.sin(elapsed * 1.1 + i) * 0.035;
        if (aim[0] === -1 && aim[1] === i)
          p.head.rotation.y = Math.max(
            -0.55,
            Math.min(0.55, Math.atan2(x - p.root.position.x, z - p.root.position.z) - p.root.rotation.y)
          );
      });
  }
  const p = game.pose();
  stream(Math.floor(p[0] / 16), Math.floor(p[2] / 16));
  const initial = [Math.floor(p[0] / 16), Math.floor(p[2] / 16)];
  for (const a of [initial[0], Math.floor((p[0] - 7) / 16), Math.floor((p[0] + 7) / 16)])
    for (const b of [initial[1], Math.floor((p[2] - 7) / 16), Math.floor((p[2] + 7) / 16)]) {
      const k = a + ',' + b;
      if (!chunks.has(k)) {
        dirty.delete(k);
        bake(a, b);
      }
    }
  return {
    canvas,
    start: view.start,
    request: view.request,
    frame,
    focus,
    state,
    flush,
    dispose: view.dispose
  };
}
