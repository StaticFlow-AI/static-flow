export function controls(canvas, h, actions, signal) {
  const keys = new Set(),
    held = {
      aim: false,
      board: false,
      fire: false,
      jump: false,
      tap: false,
      forward: 0,
      side: 0,
      yaw: 0,
      pitch: 0
    };
  const add = (el, name, fn, opts = {}) => el.addEventListener(name, fn, { ...opts, signal });
  let captured = false,
    drag = null,
    stick = null;
  function reset() {
    keys.clear();
    held.fire = held.tap = held.aim = held.jump = held.board = false;
    held.forward = held.side = 0;
    drag = null;
    stick = null;
    const knob = h.root.querySelector('.deck-stick i');
    knob.style.transform = '';
  }
  function capture() {
    canvas.focus({ preventScroll: true });
    if (!h.touch && !document.pointerLockElement) {
      try {
        canvas.requestPointerLock()?.catch(() => {});
      } catch {}
    }
  }
  function release() {
    reset();
    if (document.pointerLockElement === canvas) document.exitPointerLock();
  }
  function look(dx, dy, rate = 1) {
    if (!actions.active()) return;
    const k = (held.aim ? 0.00105 : 0.00215) * rate;
    held.yaw -= dx * k;
    held.pitch = Math.max(-1.44, Math.min(1.44, held.pitch - dy * k));
  }
  add(document, 'pointerlockchange', () => {
    const lock = document.pointerLockElement === canvas;
    if (!lock && captured && actions.active()) actions.pause();
    captured = lock;
  });
  add(document, 'mousemove', (e) => {
    if (document.pointerLockElement === canvas) look(e.movementX, e.movementY);
  });
  add(canvas, 'contextmenu', (e) => e.preventDefault());
  add(canvas, 'pointerdown', (e) => {
    if (!actions.active()) return;
    e.preventDefault();
    if (document.pointerLockElement !== canvas) {
      drag = { id: e.pointerId, x: e.clientX, y: e.clientY };
      canvas.setPointerCapture(e.pointerId);
      if (e.pointerType === 'mouse') capture();
    }
    if (e.pointerType === 'mouse') {
      if (e.button === 0) held.fire = held.tap = true;
      if (e.button === 2) held.aim = true;
    }
  });
  add(canvas, 'pointermove', (e) => {
    if (!drag || drag.id !== e.pointerId || document.pointerLockElement === canvas) return;
    look(e.clientX - drag.x, e.clientY - drag.y, e.pointerType === 'touch' ? 1.95 : 1);
    drag.x = e.clientX;
    drag.y = e.clientY;
  });
  add(window, 'pointerup', (e) => {
    if (drag?.id === e.pointerId) drag = null;
    if (e.pointerType === 'mouse') {
      if (e.button === 0) held.fire = false;
      if (e.button === 2) held.aim = false;
    }
  });
  add(canvas, 'pointercancel', () => {
    drag = null;
    held.fire = held.aim = false;
  });
  add(
    canvas,
    'wheel',
    (e) => {
      if (actions.active()) {
        e.preventDefault();
        actions.select();
      }
    },
    { passive: false }
  );
  add(window, 'keydown', (e) => {
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    if (e.code === 'Escape' && !e.repeat) {
      if (actions.active()) actions.pause();
      else if (h.panel() === 'settings') actions.back();
      else if (h.panel() === 'pause') actions.resume();
      return;
    }
    if (e.code === 'F1') {
      e.preventDefault();
      if (!e.repeat) actions.settings();
      return;
    }
    if (!actions.active()) return;
    if (
      [
        'Space',
        'Tab',
        'ArrowUp',
        'ArrowDown',
        'ArrowLeft',
        'ArrowRight',
        'ShiftLeft',
        'ControlLeft'
      ].includes(e.code)
    )
      e.preventDefault();
    keys.add(e.code);
    if (e.repeat) return;
    if (e.code === 'KeyR') actions.reload();
    if (e.code === 'Digit1' || e.code === 'Digit2') actions.select(e.code === 'Digit1' ? 0 : 1);
    if (e.code === 'Space') held.jump = true;
    if (e.code === 'Tab') held.board = true;
    if (e.code === 'KeyM') actions.sound();
  });
  add(window, 'keyup', (e) => {
    keys.delete(e.code);
    if (e.code === 'Tab') {
      e.preventDefault();
      held.board = false;
    }
  });
  const pad = h.root.querySelector('.deck-stick'),
    knob = pad.firstElementChild;
  function moveStick(e) {
    const b = pad.getBoundingClientRect(),
      dx = (e.clientX - b.left - b.width / 2) / (b.width * 0.4),
      dy = (e.clientY - b.top - b.height / 2) / (b.height * 0.4),
      len = Math.max(1, Math.hypot(dx, dy));
    held.side = dx / len;
    held.forward = -dy / len;
    knob.style.transform = 'translate(' + held.side * 22 + 'px,' + -held.forward * 22 + 'px)';
  }
  add(pad, 'pointerdown', (e) => {
    if (stick !== null) return;
    e.preventDefault();
    stick = e.pointerId;
    pad.setPointerCapture(stick);
    moveStick(e);
  });
  add(pad, 'pointermove', (e) => {
    if (stick === e.pointerId) moveStick(e);
  });
  const stopStick = (e) => {
    if (stick === e.pointerId) {
      stick = null;
      held.side = held.forward = 0;
      knob.style.transform = '';
    }
  };
  add(pad, 'pointerup', stopStick);
  add(pad, 'pointercancel', stopStick);
  for (const b of h.root.querySelectorAll('[data-touch]')) {
    const act = b.dataset.touch;
    add(b, 'pointerdown', (e) => {
      e.preventDefault();
      b.setPointerCapture(e.pointerId);
      if (act === 'fire') held.fire = held.tap = true;
      if (act === 'jump') held.jump = true;
      if (act === 'aim') held.aim = !held.aim;
      if (act === 'reload') actions.reload();
      if (act === 'switch') actions.select();
      if (act === 'pause') actions.pause();
    });
    const end = () => {
      if (act === 'fire') held.fire = false;
    };
    add(b, 'pointercancel', end);
    add(b, 'pointerup', end);
  }
  function step(sim, dt) {
    const k = (a, b) => Number(keys.has(a) || keys.has(b));
    const forward = held.forward + k('KeyW', 'ArrowUp') - k('KeyS', 'ArrowDown'),
      side = held.side + k('KeyD', 'ArrowRight') - k('KeyA', 'ArrowLeft');
    const mask =
      Number(held.fire || held.tap) |
      (Number(held.jump) << 1) |
      (Number(keys.has('KeyC') || keys.has('ControlLeft')) << 2) |
      (Number(keys.has('ShiftLeft') || keys.has('ShiftRight')) << 3) |
      (Number(held.aim) << 4);
    sim.advance(dt, forward, side, held.yaw, held.pitch, mask);
    held.tap = false;
    held.jump = false;
  }
  return { held, step, reset, capture, release };
}

export function soundscape(muted) {
  let ctx, noise, master;
  function arm() {
    if (muted) return;
    try {
      if (!ctx) {
        ctx = new AudioContext();
        master = ctx.createGain();
        master.gain.value = 0.31;
        master.connect(ctx.destination);
        noise = ctx.createBuffer(1, ctx.sampleRate * 0.33, ctx.sampleRate);
        const data = noise.getChannelData(0);
        for (let i = 0; i < data.length; i++) data[i] = Math.random() * 2 - 1;
      }
      if (ctx.state === 'suspended') ctx.resume().catch(() => {});
    } catch {}
  }
  function play(kind, volume = 1, pan = 0) {
    if (!ctx || muted || ctx.state !== 'running') return;
    const shot = kind === 'shot',
      tick = kind === 'confirm',
      step = kind === 'step',
      low = kind === 'land';
    const now = ctx.currentTime,
      t = shot ? 0.135 : tick ? 0.065 : step ? 0.065 : low ? 0.18 : 0.09;
    const p = ctx.createStereoPanner(),
      env = ctx.createGain(),
      f = ctx.createBiquadFilter();
    p.pan.value = Math.max(-1, Math.min(1, pan));
    p.connect(master);
    env.connect(p);
    f.connect(env);
    const src = tick ? ctx.createOscillator() : ctx.createBufferSource();
    if (tick) {
      src.frequency.setValueAtTime(1180, now);
      src.frequency.exponentialRampToValueAtTime(670, now + t);
      src.type = 'sine';
    } else src.buffer = noise;
    f.type = step || low ? 'lowpass' : 'bandpass';
    f.frequency.setValueAtTime(shot ? 2250 : tick ? 980 : low ? 350 : step ? 420 : 1500, now);
    f.frequency.exponentialRampToValueAtTime(shot ? 410 : tick ? 780 : step ? 180 : 280, now + t);
    env.gain.setValueAtTime(Math.max(0.005, volume) * (tick ? 0.2 : 1), now);
    env.gain.exponentialRampToValueAtTime(0.001, now + t);
    src.connect(f);
    src.start();
    src.stop(now + t);
    src.onended = () => {
      src.disconnect();
      f.disconnect();
      env.disconnect();
      p.disconnect();
    };
  }
  return {
    arm,
    play,
    mute(v) {
      muted = v;
      if (master) master.gain.value = v ? 0 : 0.31;
      if (!v) arm();
    },
    dispose() {
      ctx?.close().catch(() => {});
    }
  };
}
