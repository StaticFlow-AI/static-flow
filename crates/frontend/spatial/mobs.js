import { Group, Mesh, Vector3 } from 'three';
import { Batch } from './mesh.js';

const zombieKind = 3;
const trades = [
  { coat: 'hide', apron: 'moss', edging: 'cream' },
  { coat: 'darkblue', apron: 'grey', edging: 'metal' },
  { coat: 'green', apron: 'deepwood', edging: 'khaki' }
];

// The head, sleeves and boots share the hand-painted pixel swatches used by the story cast.
function part(parent, material, name, pivot, draw) {
  const batch = new Batch();
  draw((x, y, z, w, h, d, color) => batch.box(x / 16, y / 16, z / 16, w / 16, h / 16, d / 16, color));
  const mesh = new Mesh(batch.geometry(), material);
  mesh.name = name;
  mesh.position.set(pivot[0] / 16, pivot[1] / 16, pivot[2] / 16);
  mesh.receiveShadow = true;
  parent.add(mesh);
  return mesh;
}

function villager(kind, b) {
  const { coat, apron, edging } = trades[kind];
  b(-3.8, 5.8, -2.7, 7.6, 12.9, 5.3, coat);
  b(-3.85, 6.4, -2.75, 7.7, 0.8, 5.4, edging);
  b(-2.9, 7, 2.6, 5.8, 9, 0.3, apron);
  b(-3.86, 11, -2.76, 7.72, 1.3, 5.5, 'deepwood');
  b(-0.8, 11, 2.76, 1.6, 1.2, 0.28, 'gold');
  // Short upper sleeves, folded forearms and a sliver of joined hands.
  for (const x of [-5.2, 3.3]) {
    b(x, 13.1, -1.8, 1.9, 5, 3.7, coat);
    b(x, 12.3, 0.6, 1.9, 2.5, 3.8, coat);
  }
  b(-4.7, 12.1, 2.9, 3.6, 2.7, 2.25, coat);
  b(1.1, 12.1, 2.9, 3.6, 2.7, 2.25, coat);
  b(-1.15, 12.3, 3.25, 2.3, 2.1, 1.85, 'darkskin');
  b(-3.65, 17.6, -3.05, 7.3, 8.2, 6.1, 'darkskin');
  b(-3.5, 18.3, 3.05, 7, 6.7, 0.18, 'skin');
  b(-3.05, 22.2, 3.24, 6.1, 0.95, 0.42, 'deepwood');
  for (const x of [-2.95, 1]) {
    b(x, 21.2, 3.24, 2, 1, 0.18, 'ivory');
    b(x + (x < 0 ? 1 : 0), 21.2, 3.44, 1, 1, 0.15, 'green');
  }
  b(-1, 18.8, 3.25, 2, 3.45, 1.9, 'darkskin');
  b(-1, 18.55, 3.65, 2, 1.3, 1.5, 'hide');
  if (kind === 0) {
    b(-5.15, 24.65, -4.3, 10.3, 1, 8.8, 'gold');
    b(-3.55, 25.6, -2.95, 7.1, 1.95, 6, 'cream');
    b(-3.58, 25.65, -3, 7.16, 0.6, 6.1, 'hide');
    b(-2.1, 8, 2.95, 2.4, 1.5, 0.24, 'sage');
  } else if (kind === 1) {
    b(-3.95, 24.45, -3.3, 7.9, 1.3, 6.7, 'navy');
    b(-3.45, 25.7, -2.9, 6.9, 1.1, 5.8, 'grey');
    b(-1.2, 24.5, 3.4, 2.4, 1.7, 0.6, 'metal');
    b(-0.75, 24.8, 4.04, 1.5, 1.1, 0.22, 'lit');
    b(1.1, 8, 2.95, 1.65, 1.4, 0.24, 'metal');
  } else {
    b(-3.9, 23.95, -3.22, 7.8, 1.55, 6.5, 'moss');
    b(-3.55, 25.43, -3.1, 7.1, 1.1, 5.9, 'green');
    b(2.8, 21.6, -3.4, 1.4, 3, 1.7, 'moss');
    b(-2.9, 8.05, 2.95, 2, 1.65, 0.25, 'khaki');
  }
}

function zombie(b) {
  b(-3.85, 9.1, -2.05, 7.7, 9.4, 4.1, 'teal');
  b(-3.85, 9.35, 2.05, 2.5, 2, 0.21, 'darkblue');
  b(1.65, 8.2, 1.75, 1.7, 2.45, 0.55, 'teal');
  b(-3.86, 16.8, -2.11, 3, 1.7, 4.22, 'blue');
  b(-1.2, 17.3, 2.08, 2.4, 1.25, 0.17, 'moss');
  b(-3.95, 18.5, -3.65, 7.9, 7.85, 7.3, 'moss');
  b(-3.98, 25.5, -3.68, 7.96, 0.9, 7.36, 'green');
  b(-3.6, 22.5, 3.65, 2.95, 1.55, 0.2, 'seam');
  b(0.6, 22.55, 3.65, 3, 1.45, 0.2, 'seam');
  b(-2.7, 22.9, 3.88, 0.95, 0.6, 0.16, 'khaki');
  b(1.65, 22.95, 3.88, 0.95, 0.6, 0.16, 'khaki');
  b(-1, 20.6, 3.68, 1.9, 1.45, 0.45, 'green');
  b(-2, 19.7, 3.67, 3.7, 0.55, 0.22, 'seam');
  b(-3.55, 20.1, 3.66, 1.1, 1, 0.19, 'sage');
  b(1.9, 24.7, 3.67, 1.45, 1.45, 0.18, 'green');
  b(-3.98, 23.4, -1, 0.24, 1.35, 1.8, 'sage');
}

function model(kind, material) {
  const group = new Group();
  const hostile = kind === zombieKind;
  part(group, material, 'body', [0, 0, 0], hostile ? zombie : (b) => villager(kind, b));
  for (const side of [-1, 1]) {
    part(group, material, side < 0 ? 'leftLeg' : 'rightLeg', [side * 1.9, 9.2, 0], (b) => {
      b(-1.65, -8.4, -1.6, 3.3, 8.6, 3.2, hostile ? 'denim' : 'deepwood');
      b(-1.67, -9.2, -1.65, 3.34, 1.45, 3.8, hostile ? 'shadow' : 'hide');
      if (hostile) b(-1.35, side < 0 ? -4.2 : -6.2, 1.61, 1.65, 1.25, 0.2, 'navy');
    });
    if (hostile) {
      part(group, material, side < 0 ? 'leftArm' : 'rightArm', [side * 5.35, 17.7, 0], (b) => {
        b(-1.6, -3.1, -1.6, 3.2, 3.8, 3.3, 'teal');
        b(-1.5, -8.95, -1.5, 3, 5.85, 3.05, 'moss');
        b(-1.5, -7.75, 1.57, 1.55, 1.35, 0.18, 'green');
        b(-1.5, -9.45, -1.5, 3, 0.6, 2.9, 'sage');
      });
    }
  }
  if (hostile) {
    const fire = part(group, material, 'fire', [0, 1, 0], (b) => {
      for (const [x, z, h] of [
        [-4.9, 1.9, 6],
        [3.5, 1.4, 8],
        [-1.2, 3.6, 5],
        [2.2, -3.6, 6]
      ]) {
        b(x, 0, z, 2.4, h, 1.25, 'orange');
        b(x + 0.25, 0, z + 0.38, 1.45, h * 0.55, 0.96, 'lit');
        b(x + 0.9, h, z + 0.1, 0.9, 2.6, 0.9, 'amber');
      }
    });
    fire.visible = false;
  }
  return group;
}

export function createMobView(scene, material, game, contactGeometry, contactMaterial) {
  const templates = new Map();
  const actors = new Map();
  let lastTick = -1;
  let sampleTime = 0;
  let previousTime = 0;

  function create(id, kind) {
    if (!templates.has(kind)) templates.set(kind, model(kind, material));
    const figure = templates.get(kind).clone(true);
    const root = new Group();
    root.add(figure);
    if (contactGeometry && contactMaterial) {
      const shadow = new Mesh(contactGeometry, contactMaterial);
      shadow.rotation.x = -Math.PI / 2;
      shadow.position.y = 0.012;
      root.add(shadow);
    }
    scene.add(root);
    const actor = {
      root,
      figure,
      legs: [figure.getObjectByName('leftLeg'), figure.getObjectByName('rightLeg')],
      arms:
        kind === zombieKind ? [figure.getObjectByName('leftArm'), figure.getObjectByName('rightArm')] : null,
      fire: figure.getObjectByName('fire'),
      from: new Vector3(),
      to: new Vector3(),
      fromYaw: 0,
      turn: 0,
      phase: id * 1.9,
      speed: 0,
      flags: 0,
      seen: -1
    };
    root.visible = false;
    actors.set(id, actor);
    return actor;
  }

  function sample(time, tick) {
    const data = game.mob_poses();
    for (let i = 0; i < data.length; i += 8) {
      const id = data[i];
      const actor = actors.get(id) || create(id, data[i + 1]);
      actor.to.set(data[i + 2], data[i + 3], data[i + 4]);
      const angle = data[i + 5];
      if (
        !actor.root.visible ||
        Math.abs(time - sampleTime) > 0.35 ||
        actor.root.position.distanceToSquared(actor.to) > 4
      ) {
        actor.root.position.copy(actor.to);
        actor.root.rotation.y = angle;
      }
      actor.from.copy(actor.root.position);
      actor.fromYaw = actor.root.rotation.y;
      const change = angle - actor.fromYaw;
      actor.turn = Math.atan2(Math.sin(change), Math.cos(change));
      actor.speed = data[i + 6];
      actor.flags = data[i + 7];
      actor.seen = tick;
      actor.root.visible = true;
    }
    for (const actor of actors.values()) actor.root.visible = actor.seen === tick;
    lastTick = tick;
    sampleTime = time;
  }

  function frame(time, reduced) {
    const tick = Math.floor(time * 10);
    if (tick !== lastTick) sample(time, tick);
    const alpha = Math.min(1, Math.max(0, (time - sampleTime) * 10));
    const dt = Math.max(0, Math.min(0.2, time - previousTime));
    previousTime = time;
    for (const a of actors.values()) {
      if (!a.root.visible) continue;
      a.root.position.lerpVectors(a.from, a.to, alpha);
      a.root.rotation.y = a.fromYaw + alpha * a.turn;
      a.phase += dt * a.speed * 5.6;
      const stride = reduced ? 0 : Math.sin(a.phase) * Math.min(0.56, a.speed * 0.4);
      a.legs[0].rotation.x = stride;
      a.legs[1].rotation.x = -stride;
      a.figure.rotation.z = !reduced && a.flags & 2 ? 0.09 : 0;
      if (a.arms) {
        const lunge = !reduced && a.flags & 4 ? -0.45 : 0;
        a.arms[0].rotation.x = -1.22 - stride * 0.18 + lunge;
        a.arms[1].rotation.x = -1.32 + stride * 0.18 + lunge;
        a.fire.visible = Boolean(a.flags & 1);
        if (a.fire.visible) a.fire.scale.y = reduced ? 1 : 0.95 + Math.sin(time * 12 + a.phase) * 0.2;
      }
    }
  }

  function dispose() {
    for (const actor of actors.values()) scene.remove(actor.root);
    for (const template of templates.values()) {
      template.traverse((node) => node.geometry?.dispose());
    }
    actors.clear();
    templates.clear();
  }

  return { frame, dispose };
}
