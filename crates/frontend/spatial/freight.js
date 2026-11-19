import { field, actor, actorOffset } from './match-frame.js';
import * as T from 'three';
import { shipAssets, buildShip } from './ship.js';
import { equipment } from './equipment.js';
import { hud } from './hud-freight.js';
import { controls, soundscape } from './match-input.js';
import { Batch, surface, disposeScene } from './model.js';

function prefs() {
  try {
    const saved = JSON.parse(localStorage.getItem('staticflow.freight.settings') || '{}');
    return {
      quality: ['native', 'fast'].includes(saved.quality) ? saved.quality : 'auto',
      mute: saved.mute === true
    };
  } catch {
    return { quality: 'auto', mute: false };
  }
}
function effects(scene) {
  const cap = 80,
    life = new Float32Array(cap),
    points = new Float32Array(cap * 6),
    colors = new Float32Array(cap * 6);
  const g = new T.BufferGeometry();
  g.setAttribute('position', new T.BufferAttribute(points, 3).setUsage(T.DynamicDrawUsage));
  g.setAttribute('color', new T.BufferAttribute(colors, 3).setUsage(T.DynamicDrawUsage));
  const mat = new T.LineBasicMaterial({
    vertexColors: true,
    transparent: true,
    opacity: 0.82,
    blending: T.AdditiveBlending,
    depthWrite: false,
    toneMapped: false
  });
  const lines = new T.LineSegments(g, mat);
  lines.frustumCulled = false;
  scene.add(lines);
  let next = 0;
  const marks = new Float32Array(cap * 3),
    glow = new Float32Array(cap * 3),
    pg = new T.BufferGeometry();
  pg.setAttribute('position', new T.BufferAttribute(marks, 3).setUsage(T.DynamicDrawUsage));
  pg.setAttribute('color', new T.BufferAttribute(glow, 3).setUsage(T.DynamicDrawUsage));
  const sparks = new T.Points(
    pg,
    new T.PointsMaterial({
      size: 0.072,
      vertexColors: true,
      transparent: true,
      depthWrite: false,
      blending: T.AdditiveBlending
    })
  );
  sparks.frustumCulled = false;
  scene.add(sparks);
  function shot(e, i) {
    const n = next++ % cap,
      k = n * 6,
      s = n * 3;
    life[n] = 0.13;
    const len = Math.hypot(e[i + 6] - e[i + 3], e[i + 7] - e[i + 4], e[i + 8] - e[i + 5]),
      portion = Math.min(0.65, 7 / Math.max(1, len));
    for (let j = 0; j < 3; j++) {
      const a = e[i + 3 + j],
        b = e[i + 6 + j];
      points[k + j] = b + (a - b) * portion;
      points[k + 3 + j] = b;
      marks[s + j] = b;
    }
    g.attributes.position.needsUpdate = pg.attributes.position.needsUpdate = true;
  }
  function update(dt) {
    for (let n = 0; n < cap; n++) {
      life[n] = Math.max(0, life[n] - dt);
      const t = life[n] / 0.13,
        k = n * 6,
        s = n * 3;
      colors[k] = t * 0.18;
      colors[k + 1] = t * 0.11;
      colors[k + 2] = t * 0.05;
      colors[k + 3] = t * 1.6;
      colors[k + 4] = t;
      colors[k + 5] = t * 0.47;
      glow[s] = t * 2;
      glow[s + 1] = t * 1.3;
      glow[s + 2] = t * 0.59;
    }
    g.attributes.color.needsUpdate = pg.attributes.color.needsUpdate = true;
  }
  return { shot, update };
}
function lights(scene, shadows) {
  const hemi = new T.HemisphereLight(0xd9eaff, 0x99846b, 1.25);
  scene.add(hemi);
  const sun = new T.DirectionalLight(0xffe2bd, 2.45);
  sun.position.set(-31, 52, 23);
  scene.add(sun, sun.target);
  if (shadows) {
    sun.castShadow = true;
    const c = sun.shadow.camera;
    c.left = -42;
    c.right = 42;
    c.top = 39;
    c.bottom = -39;
    c.near = 0.5;
    c.far = 126;
    sun.shadow.mapSize.set(2048, 2048);
    sun.shadow.bias = -0.00024;
    sun.shadow.normalBias = 0.025;
    sun.shadow.radius = 1.3;
  }
  return sun;
}
export async function mount(host, sim, assetBase, signal) {
  const settings = prefs(),
    sound = soundscape(settings.mute);
  let canvas = document.createElement('canvas');
  canvas.tabIndex = 0;
  canvas.setAttribute('aria-label', '运输船第一人称竞技场');
  // Software rasterizers pay the full multisampling cost on every covered pixel. Reuse the
  // capability probe as their drawing surface, and request hardware MSAA on accelerated devices.
  const probe = document.createElement('canvas');
  probe.width = probe.height = 1;
  const sample = probe.getContext('webgl2', {
    antialias: false,
    powerPreference: 'high-performance',
    stencil: false
  });
  const info = sample?.getExtension('WEBGL_debug_renderer_info');
  const software = !!(
    info &&
    /swiftshader|llvmpipe|softpipe|software rasterizer|microsoft basic render/i.test(
      sample.getParameter(info.UNMASKED_RENDERER_WEBGL)
    )
  );
  let renderer;
  if (software) {
    canvas = probe;
    renderer = new T.WebGLRenderer({ canvas, context: sample });
    canvas.tabIndex = 0;
    canvas.setAttribute('aria-label', '运输船第一人称竞技场');
  } else {
    sample?.getExtension('WEBGL_lose_context')?.loseContext();
    renderer = new T.WebGLRenderer({
      canvas,
      antialias: true,
      powerPreference: 'high-performance',
      stencil: false
    });
  }
  host.append(canvas);
  const scene = new T.Scene(),
    camera = new T.PerspectiveCamera(68, 1, 0.055, 650),
    weaponScene = new T.Scene(),
    weaponCamera = new T.PerspectiveCamera(59, 1, 0.02, 3);
  camera.rotation.order = 'YXZ';
  renderer.toneMapping = T.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.02;
  renderer.autoClear = false;
  renderer.shadowMap.type = T.PCFShadowMap;
  renderer.shadowMap.autoUpdate = false;
  let constrained = software,
    slow = 0,
    precise,
    paused = false,
    disposed = false,
    lost = false,
    warmed = false,
    raf = 0,
    last = 0,
    elapsed = 0,
    lastStep = 0,
    lastPhase = 0,
    assets,
    ui,
    input,
    f,
    visible = true,
    lowMaterials = new Map(),
    originals = new Map(),
    resizeObserver;
  const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
  function save() {
    try {
      localStorage.setItem('staticflow.freight.settings', JSON.stringify(settings));
    } catch {}
  }
  function dispose() {
    if (disposed) return;
    disposed = true;
    cancelAnimationFrame(raf);
    resizeObserver?.disconnect();
    input?.release();
    sound.dispose();
    for (const [node, mat] of originals) node.material = mat;
    disposeScene(scene);
    disposeScene(weaponScene);
    for (const m of lowMaterials.values()) m.dispose();
    if (assets)
      for (const tex of Object.values(assets)) {
        if (tex.isTexture) tex.dispose();
        else if (tex.isObject3D) disposeScene(tex);
      }
    sun.shadow.dispose();
    renderer.dispose();
    renderer.forceContextLoss();
    canvas.remove();
    ui?.root.remove();
  }
  signal.addEventListener('abort', dispose, { once: true });
  const sun = lights(scene, true);
  lights(weaponScene, false);
  scene.fog = new T.FogExp2(0xa9b6bd, 0.0053);
  try {
    assets = await shipAssets(assetBase, signal);
    if (signal.aborted) {
      for (const t of Object.values(assets)) {
        if (t.isTexture) t.dispose();
        else if (t.isObject3D) disposeScene(t);
      }
      return { dispose };
    }
    scene.background = assets.sky;
    scene.backgroundRotation.y = 0.96;
    scene.environmentRotation.y = 0.96;
    scene.environmentIntensity = 0.54;
    weaponScene.environmentIntensity = 0.7;
    const sea = buildShip(scene, assets),
      gear = equipment({ scene, camera, weaponScene, weaponCamera, precise: () => precise }, assets),
      fx = effects(scene);
    const straps = new Batch(),
      web = surface(0x45483c, 0.89, 0.04);
    for (const sign of [-1, 1]) {
      straps.beam(web, [sign * 0.33, -0.25, -0.4], [sign * 0.59, 0.94, -0.62], 0.008);
      straps.beam(web, [sign * 0.33, -0.25, -0.4], [sign * 0.83, 1.04, -0.32], 0.006);
    }
    const canopy = straps.make(weaponScene, false);
    for (const s of [scene, weaponScene])
      s.traverse((n) => {
        if (n.material?.isMeshStandardMaterial) originals.set(n, n.material);
      });
    function resize() {
      const w = host.clientWidth || 1,
        h = host.clientHeight || 1;
      const dpr = devicePixelRatio || 1;
      const ratio = Math.min(
        renderer.capabilities.maxTextureSize / Math.max(w, h),
        precise ? dpr : Math.min(dpr, 1.15, Math.sqrt((software ? 360000 : 1400000) / (w * h)))
      );
      renderer.setDrawingBufferSize(w, h, ratio);
      camera.aspect = weaponCamera.aspect = w / h;
      camera.updateProjectionMatrix();
      weaponCamera.updateProjectionMatrix();
    }
    function quality() {
      precise = settings.quality === 'native' || (settings.quality === 'auto' && !constrained);
      const filter = precise ? Math.min(8, renderer.capabilities.getMaxAnisotropy()) : 1;
      for (const texture of Object.values(assets)) {
        if (texture.wrapS === T.RepeatWrapping && texture.anisotropy !== filter) {
          texture.anisotropy = filter;
          texture.needsUpdate = true;
        }
      }
      renderer.shadowMap.enabled = precise;
      renderer.shadowMap.needsUpdate = true;
      scene.environment = weaponScene.environment = precise ? assets.hdr : null;
      for (const [node, orig] of originals) {
        if (!precise && !lowMaterials.has(orig)) {
          const low = new T.MeshLambertMaterial({ color: orig.color, map: orig.map, side: orig.side });
          low.onBeforeCompile = orig.onBeforeCompile;
          lowMaterials.set(orig, low);
        }
        node.material = precise ? orig : lowMaterials.get(orig);
      }
      resize();
    }
    const actions = {
      active: () => !paused && !lost && f && f[field.phase] > 0 && f[field.phase] !== 4,
      prefs: () => settings,
      rules(v) {
        sim.choose_elimination(v);
        wake();
      },
      start() {
        sim.start();
        input.reset();
        input.held.yaw = input.held.pitch = 0;
        paused = false;
        last = 0;
        input.capture();
        sound.arm();
        sound.play('land', 0.4);
        wake();
      },
      resume() {
        paused = false;
        last = 0;
        ui.open('');
        if (f?.[field.phase] > 0 && f[field.phase] !== 4) input.capture();
        sound.arm();
        wake();
      },
      pause(show = true) {
        if (!f || f[field.phase] === 4) return;
        paused = true;
        input.release();
        if (show && f[field.phase] !== 0) ui.open('pause');
        wake();
      },
      settings() {
        if (f?.[field.phase] === 4) return;
        actions.pause(false);
        ui.open('settings');
      },
      back() {
        if (f?.[field.phase] === 0) {
          paused = false;
          ui.open('');
        } else ui.open('pause');
        wake();
      },
      reload() {
        if (!paused) {
          sim.reload();
          if (f?.[field.magazine] < [30, 12][f[field.weapon]]) sound.play('metal', 0.25);
        }
      },
      select(slot) {
        if (!paused) sim.select(slot ?? (f?.[field.weapon] === 0 ? 1 : 0));
      },
      sound() {
        settings.mute = !settings.mute;
        save();
        sound.mute(settings.mute);
      },
      quality(k) {
        settings.quality = k;
        save();
        quality();
        wake();
      }
    };
    ui = hud(host, actions, signal, new URL('credits.html', assetBase));
    input = controls(canvas, ui, actions, signal);
    quality();
    resizeObserver = new ResizeObserver(() => {
      resize();
      wake();
    });
    resizeObserver.observe(host);
    function cameraPose(state) {
      const phase = state[field.phase],
        t = phase === 0 ? 0 : phase === 1 ? Math.min(1, state[field.stageTime] / 3.1) : 1;
      // Cubic easing settles on the exact simulated eye without a landing camera bump.
      const remain = (1 - t) ** 2 * (1 + 2 * t),
        x = state[field.x],
        y = state[field.eyeY],
        z = state[field.z];
      camera.position.set(x + 9.5 * remain, y + 22.8 * remain + (phase === 3 ? 0.36 : 0), z + 13.0 * remain);
      camera.rotation.set(state[field.pitch] - 0.56 * remain, state[field.yaw] + 0.27 * remain, 0, 'YXZ');
      if (phase === 3 && state[field.elimination]) {
        for (let id = 1; id < 4; id++) {
          const i = actorOffset(id);
          if (state[i + actor.health] > 0) {
            camera.position.set(state[i], state[i + actor.y] + 1.76, state[i + actor.z]);
            camera.rotation.set(0, state[i + actor.yaw], 0, 'YXZ');
            break;
          }
        }
      }
      canopy.visible = phase === 1 && t < 0.87;
      sea.value = elapsed;
      return remain;
    }
    function lens(ads, remain) {
      const fov = 68 - ads * 13 + remain * 5;
      if (Math.abs(camera.fov - fov) > 0.016) {
        camera.fov = fov;
        camera.updateProjectionMatrix();
      }
    }
    function frame(now) {
      raf = 0;
      if (disposed || lost || !visible || document.hidden) return;
      const raw = last ? (now - last) / 1000 : 0,
        dt = Math.min(0.12, raw);
      last = now;
      if (!paused) elapsed += dt;
      if (!paused) input.step(sim, dt);
      f = sim.frame();
      const phase = f[field.phase];
      if (phase !== lastPhase) {
        if (phase === 2 && lastPhase === 1) sound.play('land', 0.9);
        if (phase === 2 && (lastPhase === 3 || lastPhase === 5)) {
          input.reset();
          input.held.yaw = lastPhase === 5 ? f[field.yaw] : 0;
          input.held.pitch = 0;
        }
        if (phase === 4) {
          input.release();
          sound.play('confirm', 0.7);
        }
        lastPhase = phase;
      }
      if (sim.has_events()) {
        const ev = sim.events();
        for (let i = 0; i < ev.length; i += 9) {
          const who = ev[i + 1],
            target = ev[i + 2];
          if (ev[i] === 1) {
            fx.shot(ev, i);
            const dx = ev[i + 3] - f[field.x],
              dz = ev[i + 5] - f[field.z];
            sound.play(
              'shot',
              who === 0 ? 0.98 : Math.min(0.65, 3.5 / (Math.hypot(dx, dz) + 2)),
              who === 0 ? 0 : (dx * Math.cos(f[field.yaw]) - dz * Math.sin(f[field.yaw])) / 18
            );
            if (!who && target >= 0) sound.play('confirm', 0.63);
          } else ui.elimination(who, target);
        }
      }
      if (!paused && phase === 2 && f[field.speed] > 1.3 && elapsed - lastStep > 1.82 / f[field.speed]) {
        lastStep = elapsed;
        sound.play('step', 0.18);
      }
      const remain = cameraPose(f);
      const ads = gear.draw(f, paused ? 0 : dt, input.held.aim, elapsed, reduced);
      lens(ads, remain);
      fx.update(paused ? 0 : dt);
      ui.update(f, dt, elapsed, input.held);
      renderer.clear();
      renderer.render(scene, camera);
      if (phase === 1 || phase === 2) {
        renderer.clearDepth();
        renderer.render(weaponScene, weaponCamera);
      }
      if (!paused && phase === 2 && settings.quality === 'auto' && !constrained && raw > 0 && raw < 0.2) {
        slow = raw > 0.031 ? slow + raw : Math.max(0, slow - dt * 2);
        if (slow > 3) {
          constrained = true;
          quality();
        }
      }
      if (!paused && phase !== 0 && phase !== 4) wake();
      else last = 0;
    }
    function wake() {
      if (warmed && !raf && !disposed && !lost && visible && !document.hidden)
        raf = requestAnimationFrame(frame);
    }
    canvas.addEventListener(
      'webglcontextlost',
      (e) => {
        e.preventDefault();
        lost = true;
        actions.pause(false);
        if (f?.[field.phase] > 0 && f[field.phase] !== 4) ui.open('pause');
        cancelAnimationFrame(raf);
        raf = 0;
      },
      { signal }
    );
    canvas.addEventListener(
      'webglcontextrestored',
      () => {
        lost = false;
        renderer.shadowMap.needsUpdate = true;
        last = 0;
        wake();
      },
      { signal }
    );
    document.addEventListener(
      'visibilitychange',
      () => {
        visible = !document.hidden;
        last = 0;
        if (!visible && actions.active()) actions.pause();
        if (visible) wake();
      },
      { signal }
    );
    window.addEventListener(
      'blur',
      () => {
        if (actions.active()) actions.pause();
      },
      { signal }
    );
    f = sim.frame();
    lens(0, cameraPose(f));
    gear.draw(f, 0, false, 0, reduced);
    {
      await renderer.compileAsync(scene, camera);
      if (!signal.aborted) await renderer.compileAsync(weaponScene, weaponCamera);
    }
    if (!signal.aborted) {
      gear.prewarm(renderer, weaponCamera);
      gear.draw(f, 0, false, 0, reduced);
      warmed = true;
      frame(performance.now());
    }
    return { dispose };
  } catch (error) {
    dispose();
    throw error;
  }
}
