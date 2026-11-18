import {
  BackSide,
  BoxGeometry,
  BufferGeometry,
  Color,
  DirectionalLight,
  DataTexture,
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
  LinearFilter,
  LinearMipmapLinearFilter,
  NearestFilter,
  NearestMipmapLinearFilter,
  InstancedMesh,
  Object3D,
  Points,
  PointsMaterial,
  ShaderMaterial,
  SphereGeometry,
  Vector3
} from 'three';
import { sampleAtlas } from './atlas.js';
import { stage, seeded } from './stage.js';
import { heldGeometry } from './mesh.js';
import { voxelMaterials } from './lighting.js';
import { streamChunks } from './chunks.js';
import { character, hand } from './actors.js';

function atmosphere(scene, seed) {
  const gradient = new ShaderMaterial({
    side: BackSide,
    depthWrite: false,
    fog: false,
    uniforms: {
      top: { value: new Color() },
      horizon: { value: new Color() },
      sunDir: { value: new Vector3() },
      day: { value: 1 }
    },
    vertexShader: `varying vec3 ray; void main(){ray=position;gl_Position=projectionMatrix*modelViewMatrix*vec4(position,1.);}`,
    fragmentShader: `varying vec3 ray; uniform vec3 top,horizon,sunDir; uniform float day;
      void main() {
        vec3 d=normalize(ray);float h=clamp(d.y,0.,1.);
        vec3 sky=mix(horizon,top,pow(h,.55));
        float facing=max(0.,dot(d,sunDir));
        sky+=day*vec3(.43,.27,.12)*(pow(facing,22.)+.55*pow(facing,146.));
        gl_FragColor=vec4(sky,1.);
        #include <colorspace_fragment>
      }`
  });
  const dome = new Mesh(new SphereGeometry(127, 20, 10), gradient);
  dome.renderOrder = 4;
  scene.add(dome);
  const random = seeded(seed);
  const cloudMat = new MeshBasicMaterial({ color: 0xffffff, vertexColors: true, fog: false });
  const cube = new BoxGeometry(),
    cloudBox = new BoxGeometry();
  const normal = cloudBox.attributes.normal,
    tones = [];
  for (let i = 0; i < normal.count; i++) {
    const light =
      normal.getY(i) > 0.5 ? 1 : normal.getY(i) < -0.5 ? 0.69 : normal.getX(i) > 0.5 ? 0.87 : 0.8;
    tones.push(light * 0.99, light, Math.min(1, light * 1.045));
  }
  cloudBox.setAttribute('color', new Float32BufferAttribute(tones, 3));
  const clouds = new InstancedMesh(cloudBox, cloudMat, 48),
    cloud = new Object3D();
  for (let bank = 0; bank < 16; bank++) {
    const a = ((bank + random() * 0.6) * Math.PI) / 8,
      d = 66 + random() * 24;
    const x = Math.cos(a) * d,
      z = Math.sin(a) * d,
      y = 17 + random() * 12,
      w = 6 + random() * 8;
    for (let layer = 0; layer < 3; layer++) {
      cloud.position.set(x + (layer - 1) * w * 0.52, y + (layer === 1 ? 1.2 : 0), z + random() * 1.7);
      cloud.scale.set(w * (0.64 + random() * 0.6), 0.8 + random() * 1.2, 3 + random() * 4);
      cloud.updateMatrix();
      clouds.setMatrixAt(bank * 3 + layer, cloud.matrix);
    }
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
  const noon = new Color('#5599c4'),
    dusk = new Color('#b6909b'),
    night = new Color('#233453');
  const haze = new Color('#d5e2df'),
    sunset = new Color('#f1b38b'),
    navy = new Color('#656d89');
  const axis = new Vector3(),
    forward = new Vector3(0, 0, 1);
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
      .set(Math.cos(angle), day * 0.55, -day * 0.84)
      .normalize()
      .multiplyScalar(100);
    gradient.uniforms.sunDir.value.copy(axis).normalize();
    gradient.uniforms.day.value = light;
    sun.position.copy(axis);
    moon.position.copy(axis).multiplyScalar(-1);
    sun.quaternion.setFromUnitVectors(forward, gradient.uniforms.sunDir.value);
    moon.rotation.copy(sun.rotation);
    cloudMat.color.setRGB(0.46 + light * 0.52, 0.49 + light * 0.45, 0.56 + light * 0.29);
    clouds.rotation.y = (time % 4000) * 0.00045;
    starMat.opacity = 1 - light;
    return 0.58 + 0.42 * light;
  };
}
export function worldScene(host, game, catalog, texture, signal, report, assets) {
  const view = stage(host, signal, report),
    { engine, scene, camera, canvas } = view;
  camera.rotation.order = 'YXZ';
  scene.add(camera);
  canvas.setAttribute('aria-label', '方块世界第一人称视图');
  const lighting = voxelMaterials(texture);
  const actorMaterial = new MeshLambertMaterial({ map: texture, vertexColors: true });
  actorMaterial.onBeforeCompile = sampleAtlas;
  const skyLight = new HemisphereLight(0xfff3e2, 0xaaafb0, 1.1);
  const sunlight = new DirectionalLight(0xffdfac, 1.12);
  scene.add(skyLight, sunlight, sunlight.target);
  engine.shadowMap.enabled = true;
  engine.shadowMap.type = PCFShadowMap;
  engine.shadowMap.autoUpdate = false;
  sunlight.castShadow = true;
  sunlight.shadow.mapSize.setScalar(matchMedia('(pointer:coarse)').matches ? 1024 : 2048);
  Object.assign(sunlight.shadow.camera, {
    left: -23,
    right: 23,
    top: 23,
    bottom: -23,
    near: 1,
    far: 110
  });
  sunlight.shadow.camera.updateProjectionMatrix();
  sunlight.shadow.bias = -0.00018;
  sunlight.shadow.normalBias = 0.025;
  const handMaterial = new MeshBasicMaterial({ map: texture, vertexColors: true });
  handMaterial.fog = false;
  handMaterial.onBeforeCompile = (shader) => {
    sampleAtlas(shader);
    // Reserve the nearest sliver of clip depth so the tool occludes itself without clearing the world.
    shader.vertexShader = shader.vertexShader.replace(
      '#include <project_vertex>',
      '#include <project_vertex>\n gl_Position.z=-gl_Position.w+.0004*(gl_Position.z+gl_Position.w);'
    );
  };
  view.onQuality((precise) => {
    texture.magFilter = precise ? LinearFilter : NearestFilter;
    texture.minFilter = precise ? LinearMipmapLinearFilter : NearestMipmapLinearFilter;
    texture.anisotropy = precise ? Math.min(8, engine.capabilities.getMaxAnisotropy()) : 1;
    texture.needsUpdate = true;
    for (const m of [...lighting.materials, actorMaterial, handMaterial]) {
      m.defines = { ...m.defines, VOXEL_PRECISE: precise ? 1 : 0 };
      m.needsUpdate = true;
    }
    engine.shadowMap.enabled = precise;
    engine.shadowMap.needsUpdate = true;
    view.request();
  });
  const first = hand(handMaterial),
    grip = new Group();
  grip.add(first);
  grip.renderOrder = -5;
  camera.add(grip);
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
  const people = catalog.actors.map((a) => character(a, actorMaterial));
  const ink = new Uint8Array(32 * 32 * 4);
  for (let y = 0; y < 32; y++)
    for (let x = 0; x < 32; x++) {
      const dx = (x - 15.5) / 16,
        dy = (y - 15.5) / 16;
      ink.set(
        [255, 255, 255, Math.round(225 * Math.max(0, 1 - dx * dx - dy * dy) ** 1.4)],
        (y * 32 + x) * 4
      );
    }
  const contactMap = new DataTexture(ink, 32, 32);
  contactMap.magFilter = LinearFilter;
  contactMap.needsUpdate = true;
  const contact = new PlaneGeometry(0.87, 0.74),
    shadowInk = new MeshBasicMaterial({
      map: contactMap,
      color: 0x42392e,
      transparent: true,
      opacity: 0.47,
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
    line = '',
    elapsed = 0,
    lastPose;
  const chunks = streamChunks(view, game, catalog, lighting.materials, assets, signal, report);
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
    shadowY,
    sunStep;
  const sunAxis = new Vector3(),
    right = new Vector3(),
    vertical = new Vector3(),
    up = new Vector3(0, 1, 0);
  function frame(dt, pose, time, reduced, swing) {
    const [x, y, z, yaw, pitch] = pose;
    elapsed += dt;
    chunks.frame(x, z);
    const moving = lastPose && Math.hypot(x - lastPose[0], z - lastPose[2]) > 0.002;
    lastPose = pose;
    if (moving) walkTime += dt * 10;
    camera.position.set(x, y + 1.58, z);
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
    const brightness = sky(x, y + 1.58, z, time, line);
    // Snap the shadow camera in light space so a walking player never drags the shadow texels.
    const tick = Math.floor(time / 1.5);
    const px = Math.floor(x / 4) * 4,
      py = Math.floor(y / 4) * 4,
      pz = Math.floor(z / 4) * 4;
    if (shadowX !== px || shadowZ !== pz || shadowY !== py || sunStep !== tick) {
      shadowX = px;
      shadowZ = pz;
      shadowY = py;
      sunStep = tick;
      const angle = (tick * 1.5 * Math.PI * 2) / 960 + 0.8;
      sunAxis
        .set(Math.cos(angle), Math.max(0.13, Math.abs(Math.sin(angle)) * 0.55), -Math.sin(angle) * 0.84)
        .normalize();
      right.crossVectors(up, sunAxis).normalize();
      vertical.crossVectors(sunAxis, right);
      const texel = 46 / sunlight.shadow.mapSize.x;
      const target = sunlight.target.position.set(px, py, pz);
      for (const axis of [right, vertical]) {
        const k = target.dot(axis);
        target.addScaledVector(axis, Math.round(k / texel) * texel - k);
      }
      sunlight.position.copy(target).addScaledVector(sunAxis, 52);
      engine.shadowMap.needsUpdate = true;
    }
    sunlight.intensity = (brightness - 0.58) * 3.65 + 0.06;
    skyLight.intensity = 0.7 + (brightness - 0.58) * 2.5;
    lighting.update(sunAxis, (brightness - 0.58) * 2.46 + 0.035, 0.35 + (brightness - 0.58) * 1.12);
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
  return {
    canvas,
    start: view.start,
    get playable() {
      return view.playable;
    },
    get quality() {
      return view.setting;
    },
    setQuality: view.setQuality,
    request: view.request,
    frame,
    focus,
    state,
    flush: chunks.flush,
    dispose: view.dispose
  };
}
