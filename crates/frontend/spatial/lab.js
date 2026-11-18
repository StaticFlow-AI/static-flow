import * as T from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { MeshoptDecoder } from 'three/addons/libs/meshopt_decoder.module.js';
import { stage } from './stage.js';
import { dressRoom, displayPanel } from './lab-finish.js';
import labAsset from './models/future-gadget-lab.glb';
import memberAsset from './models/lab-members.glb';

const clamp = T.MathUtils.clamp;
const targetHeights = { phone: 0.11, upa: 0.1, wave: 0.26, leap: 0.13, crt: 0.5 };
const targetNames = {
  phone: '翻盖手机',
  upa: '金属乌帕',
  wave: '电话微波炉',
  leap: '时间跳跃耳机',
  crt: '42 英寸显像管',
  suzuha: '阿万音铃羽',
  mayuri: '椎名真由理',
  daru: '桥田至'
};
const stairNorth = -3.48,
  stairSouth = 3.58;
function guide(player, goal) {
  const goalFloor = Math.round(goal.y / 3.2),
    here = Math.round(player.y / 3.2);
  if (player.x > 5.41) {
    const lane = player.x < 6.35 ? -1 : 1;
    if (player.z < -3 && goalFloor !== 0)
      return new T.Vector3(
        goalFloor < 0 ? 5.88 : 6.81,
        1.1,
        player.x < (goalFloor < 0 ? 5.59 : 6.52) || player.x > (goalFloor < 0 ? 6.17 : 7.1)
          ? stairNorth
          : stairSouth
      );
    if (player.z < 3 && player.z >= -3)
      return new T.Vector3(
        lane < 0 ? 5.88 : 6.81,
        (lane === goalFloor ? 3.2 * lane : 0) + 1.0,
        lane === goalFloor ? stairSouth : stairNorth
      );
    if (player.z >= 3 && here !== goalFloor)
      return new T.Vector3(here < 0 ? 5.88 : 6.81, 1.1, stairNorth);
    return new T.Vector3(4.86, player.y + 1.05, player.z < 0 ? stairNorth : stairSouth);
  }
  if (here !== goalFloor)
    return new T.Vector3(5.95, player.y + 1.1, here === 0 ? stairNorth : stairSouth);
  return null;
}
function blocked(origin, delta, length, boxes) {
  for (let i = 0; i < boxes.length; i += 6) {
    let near = 0,
      far = length - 0.23;
    for (const [a, min, max] of [
      ['x', boxes[i], boxes[i + 1]],
      ['z', boxes[i + 2], boxes[i + 3]],
      ['y', boxes[i + 4], boxes[i + 5]]
    ]) {
      if (Math.abs(delta[a]) < 0.0001) {
        if (origin[a] < min || origin[a] > max) {
          far = -1;
          break;
        }
      } else {
        const lo = (min - origin[a]) / delta[a],
          hi = (max - origin[a]) / delta[a];
        near = Math.max(near, Math.min(lo, hi));
        far = Math.min(far, Math.max(lo, hi));
      }
    }
    if (near <= far) return true;
  }
  return false;
}
export async function mount(host, emit, walker, signal) {
  const s = stage(host, signal, emit),
    { scene, canvas, camera } = s;
  canvas.setAttribute('aria-label', '第一人称研究所。WASD 移动，方向键转身，E 调查，M 查看手机。');
  s.engine.shadowMap.enabled = true;
  s.engine.shadowMap.type = T.PCFShadowMap;
  s.engine.shadowMap.autoUpdate = false;
  s.engine.shadowMap.needsUpdate = true;
  camera.fov = 67;
  camera.rotation.order = 'YXZ';
  camera.updateProjectionMatrix();
  const loader = new GLTFLoader().setMeshoptDecoder(MeshoptDecoder);
  async function load(asset) {
    const res = await fetch(new URL(asset, import.meta.url), { signal });
    if (!res.ok) throw Error('Model HTTP ' + res.status);
    const { scene: model } = await loader.parseAsync(await res.arrayBuffer(), '');
    if (signal.aborted) {
      model.traverse((n) => {
        n.geometry?.dispose();
        for (const m of [].concat(n.material || [])) m.dispose();
      });
      throw new DOMException('Aborted', 'AbortError');
    }
    scene.add(model);
    return model;
  }
  const [building, cast] = await Promise.all([load(labAsset), load(memberAsset)]);
  if (signal.aborted) throw new DOMException('Aborted', 'AbortError');
  const room = dressRoom(scene, building, s.request);
  const nodes = new Map(),
    motions = {},
    actors = [],
    joints = [];
  let root, meter;
  building.traverse((n) => {
    const d = n.userData;
    if (d.spawns) root = d;
    if (d.interact) nodes.set(d.interact, n);
    if (d.motion) motions[d.motion] = n;
    if (d.print === 'meter') meter = n;
  });
  cast.traverse((n) => {
    if (n.isMesh) {
      n.castShadow = true;
      n.receiveShadow = true;
    }
    const id = n.userData.actor;
    if (id) {
      const [x, y, z, yaw] = root.spawns[id];
      n.position.set(x, y, z);
      n.rotation.y = yaw;
      nodes.set(id, n);
      actors.push({ id, node: n, shadow: room.shadow(x, y, z, 0.85, 0.55) });
    }
    if (n.userData.joint)
      joints.push({ node: n, base: n.rotation.clone(), phase: joints.length * 1.37 });
  });
  scene.updateMatrixWorld(true);
  const clockMins = { alpha: 43, echo: 56, beta: 52, gate: 44 };
  const divergence = (text) => (g, w, h) => {
    g.fillStyle = '#382820';
    g.fillRect(0, 0, w, h);
    g.font = 'bold 92px monospace';
    g.textAlign = 'center';
    g.shadowColor = '#ffaa40';
    g.shadowBlur = 13;
    g.fillStyle = '#ffb15e';
    g.fillText(text, w / 2, 113);
    g.shadowBlur = 0;
  };
  const meterPaint = displayPanel(meter, [0, 0.077, 0.026], 0.36, 0.085, divergence('0.571024'));
  const screen = (on) => (g, w, h) => {
    g.fillStyle = on ? '#364d40' : '#282e2d';
    g.fillRect(0, 0, w, h);
    if (on) {
      g.fillStyle = '#a3dfaa';
      g.font = '25px monospace';
      g.fillText('AKIBA / CRT-42', 25, 46);
      g.fillText('SIGNAL LOCK   59.94 Hz', 25, 95);
      for (let y = 0; y < h; y += 5) {
        g.fillStyle = 'rgba(0,0,0,.13)';
        g.fillRect(0, y, w, 2);
      }
    }
  };
  const crtPaint = displayPanel(nodes.get('crt'), [0, 0.487, 0.41], 0.78, 0.48, screen(false));
  displayPanel(scene, [3.45, 1.112, -2.614], 0.475, 0.27, (g, w, h) => {
    g.fillStyle = '#273831';
    g.fillRect(0, 0, w, h);
    g.fillStyle = '#b9d6b1';
    g.font = '23px monospace';
    ['IBN 5100  /  LAB', 'READY > LIST', '  008   PHONEWAVE'].forEach((v, i) =>
      g.fillText(v, 18, 34 + 41 * i)
    );
  });
  const timer = (armed) => (g, w, h) => {
    g.fillStyle = '#1d302a';
    g.fillRect(0, 0, w, h);
    g.fillStyle = armed ? '#a7f5a0' : '#80af8b';
    g.textAlign = 'center';
    g.font = 'bold 122px monospace';
    g.fillText(armed ? '12:00' : '00:00', w / 2, 126);
  };
  const timerPaint = displayPanel(nodes.get('wave'), [0.305, 0.382, 0.313], 0.11, 0.05, timer(false));
  const glass = new T.Mesh(
    new T.PlaneGeometry(0.51, 0.268),
    new T.MeshPhysicalMaterial({
      color: 0x64756d,
      roughness: 0.16,
      metalness: 0.18,
      transparent: true,
      opacity: 0.24,
      side: T.DoubleSide,
      depthWrite: false
    })
  );
  glass.position.set(0.3, 0.212, 0.013);
  motions.door.add(glass);
  const held = new T.Group(),
    handset = nodes.get('phone').clone(true);
  handset.position.set(0, 0, 0);
  handset.rotation.set(0, 0, 0);
  held.add(handset);
  const handMaterial = new T.MeshStandardMaterial({ color: 0xd0a183, roughness: 0.85 });
  function palm(at, size, angle) {
    const n = new T.Mesh(new T.SphereGeometry(1, 20, 14), handMaterial);
    n.position.set(...at);
    n.scale.set(...size);
    n.rotation.z = angle;
    held.add(n);
  }
  palm([0.038, -0.021, 0.039], [0.052, 0.03, 0.084], -0.16);
  for (let i = 0; i < 3; i++) palm([-0.038, -0.009, 0.032 - i * 0.031], [0.019, 0.013, 0.014], 0.21);
  palm([0.055, 0.029, 0.043], [0.015, 0.039, 0.02], -0.4);
  const sleeve = new T.Mesh(
    new T.CylinderGeometry(0.041, 0.049, 0.17, 20),
    new T.MeshStandardMaterial({ color: 0x515956, roughness: 0.88 })
  );
  sleeve.position.set(0.056, -0.035, 0.134);
  sleeve.rotation.x = -1.25;
  held.add(sleeve);
  const pocketPaint = displayPanel(handset, [0, 0.091, -0.082], 0.071, 0.087, (g, w, h) =>
    g.clearRect(0, 0, w, h)
  );
  held.rotation.set(0.32, -0.23, -0.15);
  camera.add(held);
  scene.add(camera);
  const nav = document.createElement('span');
  nav.className = 'lab-waypoint';
  nav.setAttribute('aria-hidden', 'true');
  host.append(nav);
  const stick = document.createElement('div');
  stick.className = 'lab-stick';
  stick.setAttribute('aria-label', '移动摇杆');
  const knob = document.createElement('i');
  stick.append(knob);
  host.append(stick);
  const keys = new Set(),
    loc = new T.Vector3(),
    toward = new T.Vector3(),
    facing = new T.Vector3(),
    target = new T.Vector3(),
    projected = new T.Vector3();
  let yaw = 0.18,
    pitch = -0.11,
    phase = 0,
    focus = '',
    zone = '',
    state = {},
    lastEpoch,
    joyX = 0,
    joyY = 0,
    drag,
    joyPointer,
    locked = false,
    filteredY = 0,
    lastRange = 0;
  function stop() {
    keys.clear();
    joyX = joyY = 0;
    joyPointer = undefined;
    drag = null;
    knob.style.transform = '';
  }
  function action() {
    if (!state.modal && focus) emit('act', focus);
  }
  function turn(dx, dy, scale) {
    yaw -= dx * scale;
    pitch = clamp(pitch - dy * scale, -1.36, 1.36);
    s.request();
  }
  const on = (node, event, fn, options = {}) => node.addEventListener(event, fn, { ...options, signal });
  on(stick, 'pointerdown', (e) => {
    if (state.modal) return;
    e.preventDefault();
    joyPointer = e.pointerId;
    stick.setPointerCapture(e.pointerId);
    moveStick(e);
  });
  function moveStick(e) {
    if (joyPointer !== e.pointerId) return;
    const box = stick.getBoundingClientRect();
    const x = (e.clientX - box.left - box.width * 0.5) / (box.width * 0.36),
      y = (e.clientY - box.top - box.height * 0.5) / (box.height * 0.36),
      mag = Math.max(1, Math.hypot(x, y));
    joyX = x / mag;
    joyY = -y / mag;
    knob.style.transform = 'translate(' + joyX * 27 + 'px,' + -joyY * 27 + 'px)';
    s.request();
  }
  on(stick, 'pointermove', moveStick);
  for (const end of ['pointerup', 'pointercancel', 'lostpointercapture'])
    on(stick, end, () => {
      joyPointer = undefined;
      joyX = joyY = 0;
      knob.style.transform = '';
    });
  on(canvas, 'pointerdown', (e) => {
    if (state.modal || e.button !== 0) return;
    canvas.focus({ preventScroll: true });
    if (locked) return;
    drag = { id: e.pointerId, x: e.clientX, y: e.clientY, startX: e.clientX, startY: e.clientY };
    canvas.setPointerCapture(e.pointerId);
  });
  on(canvas, 'pointermove', (e) => {
    if (!drag || drag.id !== e.pointerId || locked || state.modal) return;
    turn(e.clientX - drag.x, e.clientY - drag.y, e.pointerType === 'touch' ? 0.0043 : 0.0034);
    drag.x = e.clientX;
    drag.y = e.clientY;
  });
  on(canvas, 'pointerup', (e) => {
    if (state.modal) return;
    if (locked) {
      action();
      return;
    }
    const tap = drag && Math.hypot(e.clientX - drag.startX, e.clientY - drag.startY) < 5;
    drag = null;
    if (tap && e.pointerType === 'mouse') {
      try {
        canvas.requestPointerLock?.()?.catch(() => {});
      } catch {
        /* Embedded previews can still drag to look. */
      }
    }
    if (tap && e.pointerType === 'touch' && focus) action();
  });
  on(canvas, 'pointercancel', () => {
    drag = null;
  });
  on(document, 'pointerlockchange', () => {
    locked = document.pointerLockElement === canvas;
    drag = null;
    if (!locked) keys.clear();
  });
  on(document, 'mousemove', (e) => {
    if (locked && !state.modal) turn(e.movementX, e.movementY, 0.0024);
  });
  const walking = [
    'KeyW',
    'KeyS',
    'KeyA',
    'KeyD',
    'ArrowLeft',
    'ArrowRight',
    'ArrowUp',
    'ArrowDown',
    'ShiftLeft',
    'ShiftRight'
  ];
  on(window, 'keydown', (e) => {
    if (e.ctrlKey || e.metaKey || e.altKey || e.target?.matches('input,textarea,select')) return;
    if (e.code === 'Escape' && state.modal) {
      emit('close');
      return;
    }
    if (state.modal) return;
    if (walking.includes(e.code)) {
      e.preventDefault();
      keys.add(e.code);
      s.request();
    }
    if (!e.repeat && e.code === 'KeyE') {
      e.preventDefault();
      action();
    }
    if (!e.repeat && e.code === 'KeyM') {
      e.preventDefault();
      emit('phone');
    }
  });
  on(window, 'keyup', (e) => keys.delete(e.code));
  on(window, 'blur', stop);
  on(document, 'visibilitychange', () => {
    if (document.hidden) stop();
  });
  const setTarget = (id, node, out) => {
    node.getWorldPosition(out);
    out.y += targetHeights[id] ?? (id === 'daru' ? 1.04 : 1.28);
    return out;
  };
  function survey() {
    facing.set(0, 0, -1).applyQuaternion(camera.quaternion);
    let best = '',
      score = -Infinity;
    for (const [id, n] of nodes) {
      if (!n.visible) continue;
      setTarget(id, n, target);
      toward.copy(target).sub(camera.position);
      const dist = toward.length();
      if (dist > 2.22 || Math.abs(camera.position.y - target.y) > 1.65 || dist < 0.02) continue;
      toward.divideScalar(dist);
      const dot = facing.dot(toward);
      if (dot < 0.51 || blocked(camera.position, toward, dist, root.solids)) continue;
      const rank = dot * 2.25 - dist * 0.27 + (id === state.objective ? 0.13 : 0);
      if (rank > score) {
        best = id;
        score = rank;
      }
    }
    if (best !== focus) {
      focus = best;
      emit('focus', best);
    }
  }
  function marker() {
    const id = state.objective,
      node = nodes.get(id);
    if (!node || !node.visible || state.modal) {
      nav.hidden = true;
      return;
    }
    setTarget(id, node, target);
    const dest = guide(loc, target);
    projected.copy(dest || target);
    projected.y += dest ? 0.12 : 0.24;
    const meters = loc.distanceTo(projected);
    projected.project(camera);
    const visible =
      projected.z >= -1 &&
      projected.z <= 1 &&
      Math.abs(projected.x) < 0.91 &&
      Math.abs(projected.y) < 0.79;
    nav.hidden = false;
    nav.classList.toggle('offscreen', !visible);
    const width = host.clientWidth,
      height = host.clientHeight;
    const dx = (dest || target).x - loc.x,
      dz = (dest || target).z - loc.z;
    const angle = Math.atan2(-dx, -dz) - yaw,
      side = Math.sin(angle) > 0 ? '←' : '→';
    nav.textContent =
      (visible ? '◇ ' : side + ' ') +
      (dest ? '楼梯' : targetNames[id] || '目标') +
      ' · ' +
      Math.max(1, Math.round(meters)) +
      ' m';
    nav.style.left = (visible ? (projected.x * 0.5 + 0.5) * width : width * 0.5) + 'px';
    nav.style.top = (visible ? (0.5 - projected.y * 0.5) * height : height * 0.22) + 'px';
  }
  function update(dt, reduced) {
    const fov = camera.aspect < 0.8 ? 84 : 67;
    if (camera.fov !== fov) {
      camera.fov = fov;
      camera.updateProjectionMatrix();
    }
    if (!state.modal) {
      const f = (keys.has('KeyW') ? 1 : 0) - (keys.has('KeyS') ? 1 : 0) + joyY,
        r = (keys.has('KeyD') ? 1 : 0) - (keys.has('KeyA') ? 1 : 0) + joyX;
      yaw += ((keys.has('ArrowLeft') ? 1 : 0) - (keys.has('ArrowRight') ? 1 : 0)) * dt * 1.65;
      pitch = clamp(
        pitch + ((keys.has('ArrowUp') ? 1 : 0) - (keys.has('ArrowDown') ? 1 : 0)) * dt * 1.2,
        -1.36,
        1.36
      );
      if (f || r)
        walker.step(f, r, yaw, dt * (keys.has('ShiftLeft') || keys.has('ShiftRight') ? 3.65 : 2.58));
      phase += dt * (f || r ? 6.5 : 1);
    }
    loc.set(walker.x, walker.y, walker.z);
    filteredY = T.MathUtils.lerp(filteredY, loc.y, Math.min(1, dt * 16));
    camera.position.set(loc.x, filteredY + 1.58 + (reduced ? 0 : Math.sin(phase) * 0.009), loc.z);
    camera.rotation.set(pitch, yaw, 0, 'YXZ');
    const halfHeight = 0.47 * Math.tan(T.MathUtils.degToRad(camera.fov * 0.5));
    held.position.set(
      Math.min(0.26, halfHeight * camera.aspect * 0.67),
      -halfHeight * 0.91 + (reduced ? 0 : Math.sin(phase) * 0.002),
      -0.47
    );
    camera.updateMatrixWorld();
    const currentZone =
      loc.x > 5.46
        ? '外楼梯'
        : loc.y > 1.66
          ? '屋顶'
          : loc.y < -1.66
            ? '一楼 · 布朗管工房'
            : '二楼 · 未来道具研究所';
    if (currentZone !== zone) {
      zone = currentZone;
      emit('zone', zone);
    }
    if (!reduced) {
      motions.fan.rotation.z += dt * 3.5;
      if (state.armed) motions.plate.rotation.y += dt * 2.1;
      for (const j of joints) {
        if (j.node.userData.joint === 'head') {
          j.node.rotation.y = j.base.y + Math.sin(phase * 0.73 + j.phase) * 0.085;
          j.node.rotation.x = j.base.x + Math.sin(phase * 0.51 + j.phase) * 0.025;
        } else j.node.rotation.z = j.base.z + Math.sin(phase * 0.71 + j.phase) * 0.022;
      }
    }
    const angle = state.armed ? 0 : -0.93;
    motions.door.rotation.y = reduced
      ? angle
      : T.MathUtils.lerp(motions.door.rotation.y, angle, Math.min(1, dt * 6));
    if (!state.modal) survey();
    lastRange += dt;
    if (lastRange > 0.065 || reduced || dt === 0) {
      marker();
      lastRange = 0;
    }
    return !state.modal && (keys.size > 0 || joyX !== 0 || joyY !== 0);
  }
  function sync(raw) {
    const next = JSON.parse(raw);
    if (next.modal && !state.modal) {
      stop();
      if (document.pointerLockElement === canvas) document.exitPointerLock();
    }
    if (next.epoch !== lastEpoch) {
      walker.reset();
      filteredY = 0;
      yaw = 0.18;
      pitch = -0.11;
      lastEpoch = next.epoch;
      stop();
    }
    if (state.line !== next.line || state.armed !== next.armed) room.set(next.line, next.armed);
    if (state.number !== next.number || state.armed !== next.armed) {
      meterPaint(divergence(next.number));
      timerPaint(timer(next.armed));
      pocketPaint((g, w, h) => {
        g.fillStyle = '#93ab98';
        g.fillRect(0, 0, w, h);
        g.fillStyle = '#273b32';
        g.font = 'bold 41px monospace';
        g.fillText('MADO  ▰▰', 20, 50);
        g.font = '62px monospace';
        g.fillText(next.armed ? 'D-MAIL' : 'INBOX', 20, 128);
      });
    }
    held.visible = next.phone && !next.modal;
    if (state.crt !== next.crt) crtPaint(screen(next.crt));
    if (state.line !== next.line || state.epoch !== next.epoch) {
      s.engine.shadowMap.needsUpdate = true;
      const boxes = root.solids.slice();
      for (const a of actors) {
        a.node.visible = !(
          (a.id === 'mayuri' && next.line === 'echo') ||
          (a.id === 'daru' && next.line === 'beta')
        );
        a.shadow.visible = a.node.visible;
        if (a.node.visible && a.id !== 'daru') {
          const [x, y, z] = root.spawns[a.id];
          boxes.push(x - 0.11, x + 0.11, z - 0.11, z + 0.11, y, y + 1.58);
        }
      }
      walker.load(new Float32Array(root.floors), new Float32Array(boxes));
      const minute = clockMins[next.line];
      motions.minute.rotation.z = (-minute * Math.PI) / 30;
      motions.hour.rotation.z = (-(5 + minute / 60) * Math.PI) / 6;
    }
    nodes.get('phone').visible = !next.phone;
    nodes.get('upa').visible = !next.upa;
    state = next;
    nav.hidden = next.modal;
    stick.hidden = next.modal;
    s.request();
    lastRange = 1;
  }
  s.start(update);
  return {
    sync,
    dispose() {
      if (document.pointerLockElement === canvas) document.exitPointerLock();
      nav.remove();
      stick.remove();
      s.dispose();
    }
  };
}
