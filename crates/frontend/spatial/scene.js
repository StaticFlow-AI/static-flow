import {
  BoxGeometry,
  DirectionalLight,
  DataTexture,
  HemisphereLight,
  PlaneGeometry,
  PCFShadowMap,
  EdgesGeometry,
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
  Vector3
} from 'three';
import { sampleAtlas } from './atlas.js';
import { stage } from './stage.js';
import { worldWeather } from './weather.js';
import { heldGeometry } from './mesh.js';
import { voxelMaterials } from './lighting.js';
import { streamChunks } from './chunks.js';
import { character, hand } from './actors.js';
import { createMobView } from './mobs.js';

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
      ink.set([255, 255, 255, Math.round(225 * Math.max(0, 1 - dx * dx - dy * dy) ** 1.4)], (y * 32 + x) * 4);
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
  const wildlife = createMobView(scene, actorMaterial, game, contact, shadowInk);
  let surveying = false;
  const sky = worldWeather(scene, game, JSON.parse(game.snapshot()));
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
    sky.state(s);
    if (surveying !== s.survey) {
      surveying = s.survey;
      grip.visible = !surveying;
      view.setSurvey(surveying);
      const range = surveying ? 50 : 23;
      Object.assign(sunlight.shadow.camera, { left: -range, right: range, top: range, bottom: -range });
      sunlight.shadow.camera.updateProjectionMatrix();
      shadowY = undefined;
      camera.far = surveying ? 430 : 150;
      camera.updateProjectionMatrix();
    }
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
    chunks.frame(x, z, surveying);
    wildlife.frame(time, reduced);
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
    const light = sky.frame(camera, time, dt, reduced);
    // Snap the shadow camera in light space so a walking player never drags the shadow texels.
    const tick = light.step;
    const lead = surveying ? Math.min(60, Math.max(0, y - 14) / Math.max(0.45, -Math.tan(pitch))) : 0;
    const px = Math.floor((x - Math.sin(yaw) * lead) / 4) * 4,
      py = Math.floor((surveying ? Math.min(y, 16) : y) / 4) * 4,
      pz = Math.floor((z - Math.cos(yaw) * lead) / 4) * 4;
    if (shadowX !== px || shadowZ !== pz || shadowY !== py || sunStep !== tick) {
      shadowX = px;
      shadowZ = pz;
      shadowY = py;
      sunStep = tick;
      sunAxis.copy(light.direction);
      right.crossVectors(up, sunAxis).normalize();
      vertical.crossVectors(sunAxis, right);
      const texel = (2 * sunlight.shadow.camera.right) / sunlight.shadow.mapSize.x;
      const target = sunlight.target.position.set(px, py, pz);
      for (const axis of [right, vertical]) {
        const k = target.dot(axis);
        target.addScaledVector(axis, Math.round(k / texel) * texel - k);
      }
      sunlight.position.copy(target).addScaledVector(sunAxis, 52);
      engine.shadowMap.needsUpdate = true;
    }
    sunlight.color.copy(light.color);
    sunlight.intensity = 1.45 * light.direct;
    skyLight.intensity = 0.16 + 1.9 * light.ambient;
    lighting.update(sunAxis, light.direct, light.ambient, reduced ? 0 : time);
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
    flush() {
      sky.invalidate(chunks.flush());
    },
    dispose() {
      wildlife.dispose();
      view.dispose();
    }
  };
}
