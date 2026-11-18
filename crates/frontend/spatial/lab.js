import { makeAtlas } from './atlas.js';
import { worldScene } from './scene.js';
import { makeHud } from './hud.js';

const SAVE = 'staticflow.lab.blocks.v2';
const LEGACY = 'staticflow.lab.worldlines.v1';
export function mount(host, game, signal, report) {
  let stored = null,
    lockedSave = false,
    saveWarning = '';
  try {
    stored = localStorage.getItem(SAVE);
    if (stored !== null && !game.restore(stored)) {
      localStorage.setItem(SAVE + '.unreadable', stored);
      saveWarning = '旧存档无法读取，已保留原始副本。';
    } else if (stored === null) {
      const old = localStorage.getItem(LEGACY);
      if (old && game.import_story(old)) saveWarning = '已接续原来的世界线记录。';
    }
  } catch {
    saveWarning = '浏览器未允许本地存档。';
  }
  const cat = JSON.parse(game.catalog()),
    { texture, icons, instruments } = makeAtlas(cat.items);
  const world = worldScene(host, game, cat, texture, signal, report),
    canvas = world.canvas;
  const hud = makeHud(host, cat, icons, action, signal),
    keys = new Set();
  let state,
    clock = 0,
    saveTime = 0,
    alive = true,
    swing = 0,
    engaged = false,
    jump = false,
    holding = false,
    mining = 0,
    targetKey = '',
    miningDenied = false;
  let stickId = null,
    side = 0,
    forward = 0,
    drag = null,
    lastTouchTap = 0,
    shiftTimer = 0;
  const on = (element, type, fn, options = {}) =>
    element.addEventListener(type, fn, { signal, ...options });
  function save() {
    if (!alive || lockedSave) return false;
    try {
      if (localStorage.getItem(SAVE) !== stored) {
        lockedSave = true;
        hud.storage('另一标签页更新了存档。重新载入以继续保存。');
        return false;
      }
      const next = game.save();
      localStorage.setItem(SAVE, next);
      stored = next;
      hud.storage('');
      saveTime = 0;
      return true;
    } catch {
      hud.storage('存档写入失败，请检查浏览器存储空间或权限。');
      return false;
    }
  }
  function refresh(write = false) {
    state = JSON.parse(game.snapshot());
    clock = state.seconds;
    instruments(state);
    world.flush();
    world.state(state);
    hud.state(state);
    world.request();
    if (write) save();
  }
  function release() {
    keys.clear();
    holding = false;
    jump = false;
    side = forward = 0;
    stickId = null;
    drag = null;
    mining = 0;
    targetKey = '';
    miningDenied = false;
    hud.stick.firstElementChild.style.transform = 'translate(-50%, -50%)';
  }
  function unlock() {
    if (document.pointerLockElement === canvas) document.exitPointerLock();
  }
  function focusCanvas() {
    canvas.focus({ preventScroll: true });
  }
  function interactPanel(mode) {
    release();
    unlock();
    refresh();
    hud.panel(mode);
  }
  function closePanel() {
    hud.panel('');
    game.close();
    release();
    refresh();
    focusCanvas();
  }
  function flash() {
    host.classList.remove('voxel-shift');
    void host.offsetWidth;
    host.classList.add('voxel-shift');
    clearTimeout(shiftTimer);
    shiftTimer = setTimeout(() => host.classList.remove('voxel-shift'), 650);
  }
  function action(key, id) {
    switch (key) {
      case 'slot':
        game.select(id);
        refresh(true);
        if (!hud.mode) focusCanvas();
        break;
      case 'equip':
        game.equip(id);
        refresh(true);
        hud.toast('已拿起' + cat.items[id].name);
        break;
      case 'craft': {
        const desc = game.craft(id);
        refresh(true);
        hud.toast(desc);
        break;
      }
      case 'close':
        closePanel();
        break;
      case 'bag':
      case 'journal':
      case 'help':
      case 'reset':
        interactPanel(key);
        break;
      case 'new': {
        const previous = game.save();
        game.restart(Date.now() >>> 0);
        if (save()) location.reload();
        else {
          game.restore(previous);
          closePanel();
          hud.toast('新世界保存失败，已恢复当前世界。');
        }
        break;
      }
      case 'home':
        closePanel();
        game.home();
        refresh(true);
        hud.toast('已回到研究所');
        break;
      case 'use': {
        if (cat.items[state.bar[state.slot]].key === 'apple' && state.bag[state.bar[state.slot]]) {
          const msg = game.eat();
          refresh(true);
          hud.toast(msg);
          break;
        }
        const res = game.interact();
        refresh(true);
        if (state.panel) {
          interactPanel(state.panel.type);
        } else if (['bench', 'furnace', 'bag'].includes(res)) {
          interactPanel('bag');
        } else if (res) hud.toast(res);
        break;
      }
      case 'phone':
        if (game.open_phone()) interactPanel('phone');
        else hud.toast('先去二楼茶几拿手机');
        break;
      case 'send':
        if (game.send(id)) {
          closePanel();
          refresh(true);
          flash();
          hud.toast('D-Mail 已抵达过去 · ' + state.number + ' %');
        }
        break;
      case 'leap':
        if (game.leap()) {
          closePanel();
          game.home();
          refresh(true);
          flash();
          hud.toast('记忆已回到 17:43');
        }
        break;
      case 'place': {
        const msg = game.place();
        swing = 1;
        refresh(true);
        hud.toast(msg);
        break;
      }
    }
  }
  function requestLook() {
    focusCanvas();
    engaged = true;
    hud.controls(true);
    try {
      canvas.requestPointerLock?.()?.catch(() => {
        hud.controls(false);
      });
    } catch {
      hud.controls(false);
    }
  }
  on(canvas, 'contextmenu', (e) => e.preventDefault());
  on(canvas, 'pointerdown', (e) => {
    if (hud.mode) return;
    e.preventDefault();
    focusCanvas();
    if (e.pointerType === 'touch') {
      if (drag) return;
      drag = { id: e.pointerId, x: e.clientX, y: e.clientY, startX: e.clientX, startY: e.clientY };
      canvas.setPointerCapture(e.pointerId);
      return;
    }
    if (e.button === 2) {
      action('place');
      return;
    }
    if (e.button !== 0) return;
    if (document.pointerLockElement !== canvas) {
      requestLook();
      drag = { id: e.pointerId, x: e.clientX, y: e.clientY };
    }
    holding = true;
    mining = 0;
    miningDenied = false;
  });
  function turn(dx, dy, amount) {
    game.look(-dx * amount, -dy * amount);
  }
  on(document, 'pointermove', (e) => {
    if (hud.mode) return;
    if (document.pointerLockElement === canvas) {
      turn(e.movementX, e.movementY, 0.0024);
      return;
    }
    if (drag?.id === e.pointerId) {
      turn(e.clientX - drag.x, e.clientY - drag.y, e.pointerType === 'touch' ? 0.005 : 0.003);
      drag.x = e.clientX;
      drag.y = e.clientY;
    }
  });
  on(document, 'pointerup', (e) => {
    if (
      e.pointerType === 'touch' &&
      drag?.id === e.pointerId &&
      Math.hypot(e.clientX - drag.startX, e.clientY - drag.startY) < 8
    ) {
      const now = performance.now();
      if (now - lastTouchTap < 320) action('use');
      lastTouchTap = now;
    }
    if (e.button === 0 && e.pointerType !== 'touch') holding = false;
    if (drag?.id === e.pointerId) drag = null;
  });
  on(canvas, 'pointercancel', () => {
    drag = null;
    holding = false;
  });
  on(document, 'pointerlockchange', () => {
    if (document.pointerLockElement !== canvas) {
      holding = false;
      engaged = false;
    } else {
      engaged = true;
      drag = null;
    }
    hud.controls(engaged);
  });
  on(
    canvas,
    'wheel',
    (e) => {
      if (!hud.mode && e.deltaY) {
        e.preventDefault();
        action('slot', (state.slot + (e.deltaY > 0 ? 1 : 8)) % 9);
      }
    },
    { passive: false }
  );
  const motion = new Set([
    'KeyW',
    'KeyA',
    'KeyS',
    'KeyD',
    'ArrowLeft',
    'ArrowRight',
    'ArrowUp',
    'ArrowDown',
    'PageUp',
    'PageDown',
    'ShiftLeft',
    'ShiftRight',
    'Space',
    'KeyF'
  ]);
  on(document, 'keydown', (e) => {
    if (e.altKey || e.ctrlKey || e.metaKey || e.isComposing) return;
    const k = e.code;
    if (hud.mode) {
      if (
        (k === 'KeyB' && hud.mode === 'bag') ||
        (k === 'KeyM' && hud.mode === 'phone') ||
        (k === 'KeyJ' && hud.mode === 'journal')
      ) {
        if (!e.repeat) {
          e.preventDefault();
          action('close');
        }
      }
      return;
    }
    if (k === 'Tab' || k === 'Enter') return;
    if (motion.has(k)) {
      e.preventDefault();
      keys.add(k);
      if (k === 'Space' && !e.repeat) jump = true;
      if (k === 'KeyF' && !e.repeat) {
        holding = true;
        miningDenied = false;
      }
      return;
    }
    if (e.repeat) return;
    if (/^Digit[1-9]$/.test(k)) {
      e.preventDefault();
      action('slot', Number(k.slice(-1)) - 1);
      return;
    }
    const mapped = {
      KeyB: 'bag',
      KeyJ: 'journal',
      KeyM: 'phone',
      KeyE: 'use',
      KeyR: 'place',
      F1: 'help'
    }[k];
    if (mapped) {
      e.preventDefault();
      action(mapped);
    }
  });
  on(document, 'keyup', (e) => {
    keys.delete(e.code);
    if (e.code === 'KeyF') holding = false;
  });
  function joystick(e) {
    const rect = hud.stick.getBoundingClientRect(),
      dx = e.clientX - rect.left - rect.width / 2,
      dy = e.clientY - rect.top - rect.height / 2;
    const radius = rect.width * 0.36,
      mag = Math.max(radius, Math.hypot(dx, dy));
    side = dx / mag;
    forward = -dy / mag;
    hud.stick.firstElementChild.style.transform =
      'translate(calc(-50% + ' + side * radius + 'px),calc(-50% + ' + -forward * radius + 'px))';
  }
  on(hud.stick, 'pointerdown', (e) => {
    e.preventDefault();
    stickId = e.pointerId;
    hud.stick.setPointerCapture(e.pointerId);
    joystick(e);
  });
  on(hud.stick, 'pointermove', (e) => {
    if (e.pointerId === stickId) joystick(e);
  });
  function endStick(e) {
    if (e.pointerId === stickId) {
      stickId = null;
      side = forward = 0;
      hud.stick.firstElementChild.style.transform = 'translate(-50%, -50%)';
    }
  }
  on(hud.stick, 'pointerup', endStick);
  on(hud.stick, 'pointercancel', endStick);
  on(hud.touch, 'pointerdown', (e) => {
    const b = e.target.closest('button');
    if (!b) return;
    if (b.dataset.action === 'jump') {
      e.preventDefault();
      jump = true;
    }
    if (b.dataset.action === 'mine') {
      e.preventDefault();
      b.setPointerCapture(e.pointerId);
      holding = true;
      mining = 0;
      miningDenied = false;
    }
  });
  on(hud.touch, 'pointerup', () => {
    holding = false;
  });
  on(hud.touch, 'pointercancel', () => {
    holding = false;
  });
  on(window, 'blur', () => {
    release();
    save();
  });
  on(window, 'pagehide', save);
  on(document, 'visibilitychange', () => {
    if (document.hidden) {
      release();
      save();
    }
  });
  refresh();
  hud.toast(saveWarning || (!state.phone ? '茶几上的手机在响　·　走近后调查' : '进度已接续'));
  world.start((dt, reduced) => {
    if (!alive) return false;
    if (!hud.mode) {
      const check = (k) => Number(keys.has(k));
      game.look(
        (check('ArrowLeft') - check('ArrowRight')) * dt * 1.85,
        (check('PageUp') - check('PageDown')) * dt * 1.4
      );
      const dirty = game.advance(
        dt,
        side + check('KeyD') - check('KeyA'),
        forward + check('KeyW') + check('ArrowUp') - check('KeyS') - check('ArrowDown'),
        jump,
        keys.has('ShiftLeft') || keys.has('ShiftRight')
      );
      jump = false;
      clock += dt;
      if (dirty) {
        const hp = state.health;
        refresh(true);
        if (state.health < hp) hud.toast('摔伤了，吃苹果可以恢复');
      }
      const hit = game.aim(),
        key = hit.length > 2 ? hit.slice(0, 4).join(',') : '';
      if (key !== targetKey || !holding) {
        mining = 0;
        miningDenied = false;
        targetKey = key;
      }
      if (holding && key && !miningDenied) {
        const required = game.break_time(hit[0]);
        if (required < 0) {
          hud.toast(game.mine());
          miningDenied = true;
        } else if ((mining += dt) >= required) {
          const message = game.mine();
          refresh(true);
          hud.toast(message);
          mining = 0;
          swing = 1;
        }
      }
      const pct =
        mining && hit.length ? Math.min(1, mining / Math.max(0.05, game.break_time(hit[0]))) : 0;
      world.focus(hit, pct);
      hud.target(hit, pct);
      saveTime += dt;
      if (saveTime >= 5) {
        saveTime = 0;
        save();
      }
    }
    const p = game.pose();
    swing = Math.max(0, swing - dt * 3.7);
    world.frame(dt, p, clock, reduced, swing);
    hud.position(p);
    return !hud.mode;
  });
  return {
    dispose() {
      if (!alive) return;
      save();
      alive = false;
      clearTimeout(shiftTimer);
      unlock();
      world.dispose();
      hud.dispose();
    }
  };
}
