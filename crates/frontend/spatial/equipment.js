import { field, actor, actorOffset } from './match-frame.js';
import * as T from 'three';
import { Batch, canvasTexture, surface } from './model.js';

const forward = [Math.PI / 2, 0, 0];
function profile(b, m, points, width, bevel = 0.008, cutout) {
  const outline = new T.Shape();
  points.forEach(([z, y], i) => (i ? outline.lineTo(z, y) : outline.moveTo(z, y)));
  outline.closePath();
  if (cutout) {
    const hole = new T.Path();
    cutout.forEach(([x, y], i) => (i ? hole.lineTo(x, y) : hole.moveTo(x, y)));
    hole.closePath();
    outline.holes.push(hole);
  }
  const g = new T.ExtrudeGeometry(outline, {
    depth: width,
    bevelEnabled: true,
    bevelThickness: bevel,
    bevelSize: bevel,
    bevelSegments: 1
  });
  b.piece(g, m, width / 2, 0, 0, 1, 1, 1, [0, -Math.PI / 2, 0]);
  g.dispose();
}
function kit(a) {
  const wear = {
    map: a['paint-color'],
    normalMap: a['paint-normal'],
    normalScale: new T.Vector2(0.09, 0.09)
  };
  return {
    body: surface(0x525c5c, 0.35, 0.72, wear),
    steel: surface(0x333b42, 0.28, 0.76, wear),
    edge: surface(0x8a9393, 0.33, 0.72, wear),
    rubber: surface(0x272d2e, 0.86, 0.06),
    sand: surface(0x81795b, 0.69, 0.19, wear),
    mark: new T.MeshBasicMaterial({ color: 0xc4c4ae })
  };
}
function carbine(m, mag) {
  const b = new Batch();
  profile(
    b,
    m.body,
    [
      [-0.31, 0.025],
      [-0.31, 0.105],
      [-0.24, 0.135],
      [0.092, 0.135],
      [0.155, 0.102],
      [0.14, 0.02],
      [0.047, 0.008],
      [0.012, -0.045],
      [-0.07, -0.045],
      [-0.11, 0.012]
    ],
    0.08
  );
  b.bevel(m.sand, 0, 0.074, -0.377, 0.09, 0.104, 0.213);
  for (const s of [-1, 1]) {
    for (let k = 0; k < 6; k++)
      b.bevel(m.rubber, s * 0.046, 0.071, -0.292 - k * 0.032, 0.006, 0.032, 0.02, 0.004);
    b.bevel(m.steel, s * 0.041, 0.086, -0.06, 0.009, 0.043, 0.134, 0.005);
    b.bevel(m.edge, s * 0.048, 0.077, -0.027, 0.009, 0.009, 0.052, 0.002);
    for (const z of [-0.238, 0.084]) b.cyl(m.steel, s * 0.046, 0.07, z, 0.011, 0.007, 8, [0, 0, Math.PI / 2]);
  }
  b.cyl(m.steel, 0, 0.071, -0.524, 0.027, 0.12, 12, forward);
  b.cyl(m.body, 0, 0.071, -0.604, 0.034, 0.09, 12, forward);
  b.cyl(m.rubber, 0, 0.071, -0.65, 0.017, 0.004, 12, forward);
  for (let i = 0; i < 4; i++) b.cyl(m.edge, 0, 0.071, -0.57 - i * 0.018, 0.0345, 0.004, 12, forward);
  b.bevel(m.steel, 0, 0.139, -0.13, 0.056, 0.018, 0.55, 0.005);
  for (let i = 0; i < 23; i++) b.box(m.body, 0, 0.153, -0.385 + i * 0.023, 0.081, 0.011, 0.012);
  profile(
    b,
    m.rubber,
    [
      [0.006, 0.008],
      [0.071, 0.011],
      [0.109, -0.128],
      [0.073, -0.142],
      [0.032, -0.102]
    ],
    0.055
  );
  // Open trigger guard and offset, open-window reflex optic.
  b.beam(m.steel, [-0.012, -0.024, -0.066], [-0.012, -0.077, -0.056], 0.007);
  b.beam(m.steel, [-0.012, -0.077, -0.056], [-0.012, -0.077, 0.043], 0.007);
  b.beam(m.edge, [0, -0.024, -0.02], [0, -0.056, -0.013], 0.006);
  b.cyl(m.steel, 0, 0.074, 0.177, 0.028, 0.12, 12, forward);
  profile(
    b,
    m.sand,
    [
      [0.137, 0.122],
      [0.296, 0.122],
      [0.349, 0.082],
      [0.342, -0.005],
      [0.27, -0.044],
      [0.18, 0.051],
      [0.137, 0.063]
    ],
    0.073,
    0.005,
    [
      [0.215, 0.08],
      [0.272, 0.08],
      [0.283, 0.061],
      [0.272, 0.029]
    ]
  );
  b.bevel(m.rubber, 0, 0.035, 0.344, 0.087, 0.163, 0.031, 0.01);
  b.bevel(m.steel, 0, 0.17, -0.015, 0.084, 0.03, 0.105, 0.007);
  const hood = new T.Shape();
  hood.moveTo(-0.045, 0);
  hood.lineTo(0.045, 0);
  hood.lineTo(0.043, 0.036);
  hood.quadraticCurveTo(0.039, 0.065, 0.02, 0.066);
  hood.lineTo(-0.02, 0.066);
  hood.quadraticCurveTo(-0.041, 0.064, -0.044, 0.039);
  hood.closePath();
  const window = new T.Path();
  window.moveTo(-0.034, 0.01);
  window.lineTo(-0.033, 0.035);
  window.quadraticCurveTo(-0.031, 0.055, -0.015, 0.056);
  window.lineTo(0.016, 0.056);
  window.quadraticCurveTo(0.032, 0.054, 0.033, 0.036);
  window.lineTo(0.034, 0.01);
  window.closePath();
  hood.holes.push(window);
  const optic = new T.ExtrudeGeometry(hood, {
    depth: 0.045,
    bevelEnabled: true,
    bevelThickness: 0.0025,
    bevelSize: 0.0025,
    bevelSegments: 2,
    curveSegments: 6
  });
  b.piece(optic, m.steel, 0, 0.177, -0.062);
  optic.dispose();
  const lens = new T.PlaneGeometry(0.064, 0.041);
  b.piece(
    lens,
    new T.MeshBasicMaterial({ color: 0x7ac3ac, transparent: true, opacity: 0.095, depthWrite: false }),
    0,
    0.207,
    -0.053
  );
  lens.dispose();
  const tag = canvasTexture(256, 96, (c) => {
    c.clearRect(0, 0, 256, 96);
    c.fillStyle = '#d5d7c7';
    c.font = 'bold 29px monospace';
    c.fillText('NORTHLINE', 4, 36);
    c.font = '20px monospace';
    c.fillText('AR-31   //   204', 5, 67);
  });
  const plate = new T.PlaneGeometry(0.112, 0.04);
  b.piece(
    plate,
    new T.MeshBasicMaterial({ map: tag, transparent: true, depthWrite: false }),
    -0.044,
    0.058,
    -0.158,
    1,
    1,
    1,
    [0, -Math.PI / 2, 0]
  );
  plate.dispose();
  b.cyl(m.steel, 0.063, 0.197, -0.01, 0.017, 0.028, 12, [0, 0, Math.PI / 2]);
  for (let i = 0; i < 5; i++) b.box(m.mark, 0.046, 0.053, -0.149 + i * 0.009, 0.002, 0.008, 0.004);
  profile(
    mag,
    m.sand,
    [
      [-0.131, 0.007],
      [-0.06, 0.007],
      [-0.058, -0.1],
      [-0.08, -0.184],
      [-0.163, -0.172],
      [-0.137, -0.093]
    ],
    0.062
  );
  for (const s of [-1, 1])
    for (let i = 0; i < 4; i++)
      mag.bevel(m.rubber, s * 0.032, -0.091, -0.071 - i * 0.017, 0.004, 0.11, 0.006, 0.002, [-0.13, 0, 0]);
  mag.bevel(m.rubber, 0, -0.179, -0.121, 0.074, 0.022, 0.091, 0.004, [0.1, 0, 0]);
  return b;
}
function sidearm(m, mag) {
  const b = new Batch();
  profile(
    b,
    m.sand,
    [
      [-0.194, 0.03],
      [-0.19, 0.08],
      [0.072, 0.078],
      [0.1, 0.042],
      [0.068, 0.011],
      [0.112, -0.121],
      [0.039, -0.141],
      [-0.003, -0.025],
      [-0.113, -0.025],
      [-0.139, 0.007]
    ],
    0.052
  );
  b.bevel(m.steel, 0, 0.086, -0.057, 0.066, 0.052, 0.278, 0.009);
  for (let i = 0; i < 8; i++)
    for (const s of [-1, 1])
      b.box(m.edge, s * 0.033, 0.089, 0.01 + i * 0.008, 0.003, 0.031, 0.003, [0.21, 0, 0]);
  for (const s of [-1, 1])
    b.bevel(m.rubber, s * 0.03, -0.073, 0.049, 0.008, 0.1, 0.051, 0.003, [-0.23, 0, 0]);
  b.cyl(m.rubber, 0, 0.082, -0.201, 0.016, 0.012, 12, forward);
  b.box(m.steel, 0, 0.125, -0.169, 0.012, 0.018, 0.014);
  for (const s of [-1, 1]) b.box(m.steel, s * 0.017, 0.128, 0.06, 0.012, 0.018, 0.018);
  b.beam(m.steel, [0, -0.019, -0.095], [0, -0.063, -0.088], 0.005);
  b.beam(m.steel, [0, -0.063, -0.088], [0, -0.065, 0.017], 0.005);
  mag.bevel(m.rubber, 0, -0.14, 0.078, 0.06, 0.019, 0.078, 0.005);
  return b;
}
function insignia(shader) {
  shader.vertexShader =
    'attribute float _band; varying float vBand;\n' +
    shader.vertexShader.replace('#include <color_vertex>', '#include <color_vertex>\n vBand=_band;');
  shader.fragmentShader =
    'varying float vBand;\n' +
    shader.fragmentShader.replace(
      '#include <color_fragment>',
      `
    #ifdef USE_COLOR
      float fabric=.34+dot(diffuseColor.rgb,vec3(.23,.64,.13));
      diffuseColor.rgb=mix(diffuseColor.rgb,vColor.rgb*fabric,smoothstep(.18,.80,vBand));
    #endif
  `
    );
}
export function equipment(view, a) {
  const m = kit(a),
    hold = new T.Group();
  view.weaponScene.add(hold);
  const models = [false, true].map((pistol) => {
    const mag = new Batch(),
      group = new T.Group();
    const solid = pistol ? sidearm(m, mag) : carbine(m, mag);
    solid.make(group, false);
    const clip = mag.make(group, false);
    for (const side of ['left', 'right']) {
      const sleeve = a.crew.getObjectByName(side + 'Sleeve').clone();
      sleeve.castShadow = sleeve.receiveShadow = false;
      if (pistol && side === 'left') sleeve.position.set(0.016, -0.02, 0.278);
      group.add(sleeve);
    }
    hold.add(group);
    return { group, clip };
  });
  const flashTex = canvasTexture(128, 128, (c) => {
    const g = c.createRadialGradient(64, 64, 1, 64, 64, 61);
    g.addColorStop(0, 'white');
    g.addColorStop(0.12, '#fff6a6');
    g.addColorStop(0.38, '#ffab3be8');
    g.addColorStop(1, '#fd501000');
    c.fillStyle = g;
    c.beginPath();
    for (let i = 0; i < 24; i++) {
      const r = i % 2 ? 12 : 35 + 24 * Math.sin(i * 12) ** 2,
        a = (i * Math.PI) / 12;
      const x = 64 + Math.cos(a) * r,
        y = 64 + Math.sin(a) * r;
      i ? c.lineTo(x, y) : c.moveTo(x, y);
    }
    c.closePath();
    c.fill();
  });
  const flash = new T.Sprite(
    new T.SpriteMaterial({
      map: flashTex,
      blending: T.AdditiveBlending,
      transparent: true,
      depthWrite: false,
      toneMapped: false
    })
  );
  hold.add(flash);
  let recoil = 0,
    ads = 0,
    previous = -1,
    travel = 0;
  const fatigues = a.crew.getObjectByName('close').material.clone();
  fatigues.onBeforeCompile = insignia;
  const uniforms = ['close', 'far'].map((key) => {
    const original = a.crew.getObjectByName(key);
    const mesh = new T.InstancedMesh(original.geometry, fatigues, 7);
    mesh.instanceMatrix.setUsage(T.DynamicDrawUsage);
    mesh.frustumCulled = false;
    mesh.receiveShadow = true;
    original.morphTargetInfluences.fill(0);
    mesh.setMorphAt(0, original);
    mesh.setColorAt(0, new T.Color(1, 1, 1));
    view.scene.add(mesh);
    return { mesh, pose: original, visible: 0 };
  });
  const carry = new Batch();
  carry.bevel(m.steel, 0.073, 1.33, -0.24, 0.075, 0.09, 0.35, 0.01);
  carry.bevel(m.sand, 0.073, 1.33, -0.456, 0.071, 0.074, 0.16, 0.008);
  carry.cyl(m.steel, 0.073, 1.335, -0.594, 0.021, 0.153, 10, forward);
  carry.bevel(m.sand, 0.073, 1.235, -0.289, 0.058, 0.14, 0.069, 0.008, [-0.12, 0, 0]);
  carry.bevel(m.rubber, 0.073, 1.32, 0.015, 0.074, 0.098, 0.17, 0.01);
  const accessories = carry.make().children.map((original) => {
    const mesh = new T.InstancedMesh(original.geometry, original.material, 7);
    mesh.instanceMatrix.setUsage(T.DynamicDrawUsage);
    mesh.frustumCulled = false;
    mesh.receiveShadow = true;
    view.scene.add(mesh);
    return mesh;
  });
  const markerGeometry = new T.ConeGeometry(0.068, 0.09, 4);
  const friends = new T.InstancedMesh(markerGeometry, new T.MeshBasicMaterial({ color: 0x92d2e1 }), 3);
  friends.frustumCulled = false;
  view.scene.add(friends);
  const team = [new T.Color(0x72b8d5), new T.Color(0xe6a05c)];
  const frustum = new T.Frustum(),
    sphere = new T.Sphere(new T.Vector3(), 1.18),
    projection = new T.Matrix4();
  const shadowTex = canvasTexture(64, 64, (c) => {
    const g = c.createRadialGradient(32, 32, 0, 32, 32, 32);
    g.addColorStop(0, '#121814b3');
    g.addColorStop(0.56, '#151a1466');
    g.addColorStop(1, '#151a1400');
    c.fillStyle = g;
    c.fillRect(0, 0, 64, 64);
  });
  const blobs = new T.InstancedMesh(
    new T.PlaneGeometry(1.25, 0.8),
    new T.MeshBasicMaterial({ map: shadowTex, transparent: true, depthWrite: false }),
    7
  );
  blobs.frustumCulled = false;
  view.scene.add(blobs);
  const bodyPose = new T.Object3D(),
    limb = new T.Object3D(),
    mat = new T.Matrix4(),
    phases = Array(8).fill(0);
  function draw(f, dt, aim, time, reduced) {
    const slot = f[field.weapon],
      phase = f[field.phase];
    models[0].group.visible = slot === 0;
    models[1].group.visible = slot === 1;
    if (previous !== f[field.shots] && previous >= 0 && f[field.shots] > previous) {
      recoil = 1;
      flash.material.rotation = time * 16;
    }
    previous = f[field.shots];
    recoil = Math.max(0, recoil - dt * 8.8);
    ads += ((aim && phase === 2 && f[field.reload] < 0 ? 1 : 0) - ads) * (1 - Math.exp(-dt * 14));
    travel += dt * f[field.speed] * 2.7;
    const bob = reduced ? 0 : Math.min(f[field.speed] / 5, 1) * 0.009;
    const reload = f[field.reload] < 0 ? 0 : Math.sin(Math.PI * f[field.reload]);
    hold.visible = phase === 2;
    hold.scale.setScalar(0.84);
    hold.position.set(
      0.202 * Math.min(1, view.weaponCamera.aspect / 0.87) * (1 - ads) + Math.sin(travel * 0.5) * bob,
      -0.217 + ads * (slot ? 0.118 : 0.045) + Math.cos(travel) * bob - 0.082 * reload,
      -0.473 + 0.025 * recoil + 0.095 * ads
    );
    hold.rotation.set(
      0.018 + recoil * 0.048 - 0.26 * reload,
      0.119 * (1 - ads) - reload * 0.19,
      -0.045 * (1 - ads) - reload * 0.41
    );
    models[slot].clip.position.y = -0.17 * reload ** 3;
    flash.visible = recoil > 0.57 && !reload;
    flash.position.set(0, 0.075, slot ? -0.232 : -0.697);
    flash.scale.setScalar(0.1 + recoil * 0.065);
    view.camera.updateMatrixWorld();
    frustum.setFromProjectionMatrix(
      projection.multiplyMatrices(view.camera.projectionMatrix, view.camera.matrixWorldInverse)
    );
    for (const lod of uniforms) lod.visible = 0;
    let count = 0,
      friendly = 0;
    for (let id = 1; id < 8; id++) {
      const i = actorOffset(id);
      if (f[i + actor.health] <= 0) continue;
      sphere.center.set(f[i], f[i + actor.y] + 0.9, f[i + actor.z]);
      if (!frustum.intersectsSphere(sphere)) continue;
      // While spectating, keep the observed player's head outside the first-person camera.
      if (phase === 3 && f[field.elimination] && sphere.center.distanceToSquared(view.camera.position) < 1.0)
        continue;
      const dist = sphere.center.distanceToSquared(view.camera.position);
      const lod = uniforms[dist < (view.precise() ? 32 : 12) ** 2 ? 0 : 1],
        index = lod.visible++;
      phases[id] += dt * f[i + actor.speed] * 3;
      const swing = Math.sin(phases[id]) * Math.min(f[i + actor.speed] / 3.1, 1);
      lod.pose.morphTargetInfluences[0] = Math.max(0, swing);
      lod.pose.morphTargetInfluences[1] = Math.max(0, -swing);
      bodyPose.position.set(f[i], f[i + actor.y], f[i + actor.z]);
      bodyPose.rotation.set(0, f[i + actor.yaw], 0);
      bodyPose.updateMatrix();
      lod.mesh.setMatrixAt(index, bodyPose.matrix);
      lod.mesh.setMorphAt(index, lod.pose);
      lod.mesh.setColorAt(index, team[Number(id >= 4)]);
      for (const part of accessories) {
        part.setMatrixAt(count, bodyPose.matrix);
      }
      limb.position.set(0, 0.043, 0);
      limb.rotation.set(-Math.PI / 2, 0, 0);
      limb.updateMatrix();
      blobs.setMatrixAt(count, mat.multiplyMatrices(bodyPose.matrix, limb.matrix));
      if (id < 4) {
        limb.position.set(0, 1.96, 0);
        limb.rotation.set(Math.PI, 0, 0);
        limb.updateMatrix();
        friends.setMatrixAt(friendly++, mat.multiplyMatrices(bodyPose.matrix, limb.matrix));
      }
      count++;
    }
    friends.count = friendly;
    friends.instanceMatrix.needsUpdate = true;
    for (const lod of uniforms) {
      lod.mesh.count = lod.visible;
      lod.mesh.instanceMatrix.needsUpdate = true;
      lod.mesh.morphTexture.needsUpdate = true;
      if (lod.mesh.instanceColor) lod.mesh.instanceColor.needsUpdate = true;
    }
    for (const part of accessories) {
      part.count = count;
      part.instanceMatrix.needsUpdate = true;
      if (part.instanceColor) part.instanceColor.needsUpdate = true;
    }
    blobs.count = count;
    blobs.instanceMatrix.needsUpdate = true;
    return ads;
  }
  function prewarm(renderer, camera) {
    // Allocate both equipment sets before the first weapon switch.
    hold.visible = true;
    for (const model of models) model.group.visible = true;
    flash.visible = true;
    renderer.clear();
    renderer.render(view.weaponScene, camera);
    flash.visible = false;
  }
  return { draw, prewarm };
}
