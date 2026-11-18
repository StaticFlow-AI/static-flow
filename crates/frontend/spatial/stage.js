import { WebGLRenderer, Scene, PerspectiveCamera, SRGBColorSpace } from 'three';

export function stage(host, signal, notify) {
  const canvas = document.createElement('canvas');
  canvas.tabIndex = 0;
  const engine = new WebGLRenderer({
    canvas,
    antialias: true,
    alpha: false,
    powerPreference: 'high-performance',
    stencil: false
  });
  engine.outputColorSpace = SRGBColorSpace;
  const gl = engine.getContext();
  const info = gl.getExtension('WEBGL_debug_renderer_info');
  const software =
    info &&
    /swiftshader|llvmpipe|softpipe|software rasterizer|microsoft basic render/i.test(
      gl.getParameter(info.UNMASKED_RENDERER_WEBGL)
    );
  let setting = 'auto';
  try {
    const saved = localStorage.getItem('staticflow.lab.graphics');
    if (['native', 'fast'].includes(saved)) setting = saved;
  } catch {}
  let constrained = Boolean(software),
    slowTime = 0,
    slowFrames = 0,
    qualityChanged;
  const precise = () => setting === 'native' || (setting === 'auto' && !constrained);
  host.append(canvas);
  const scene = new Scene(),
    camera = new PerspectiveCamera(72, 1, 0.12, 150);
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  let frame = 0,
    previous = 0,
    seen = true,
    lost = false,
    stopped = false,
    paint,
    preparing = true,
    generation = 0,
    onPresented;
  function request() {
    if (!frame && !stopped && !lost && seen && !document.hidden && paint)
      frame = requestAnimationFrame(draw);
  }
  function draw(now) {
    frame = 0;
    if (stopped || lost || !seen || document.hidden) return;
    const dt = previous ? Math.min((now - previous) / 1000, 0.2) : 0;
    previous = now;
    const moving = paint(dt, reduced.matches);
    if (!preparing) {
      engine.render(scene, camera);
      if (onPresented) {
        onPresented();
        onPresented = null;
      }
    }
    // Select a stable preset once when sustained frame times show this GPU cannot keep up.
    // A background-tab gap, a shader compile or an individual dropped frame cannot trigger it.
    if (!preparing && moving && setting === 'auto' && !constrained && dt > 0 && dt < 0.15) {
      if (dt > 0.029) {
        slowTime += dt;
        slowFrames++;
      } else {
        slowTime = Math.max(0, slowTime - dt * 2);
        slowFrames = Math.max(0, slowFrames - 2);
      }
      if (slowTime >= 2.5 && slowFrames >= 45) {
        constrained = true;
        resize();
        qualityChanged?.(false);
      }
    }
    if (moving) request();
    else previous = 0;
  }
  function resize() {
    const w = Math.max(1, host.clientWidth),
      h = Math.max(1, host.clientHeight);
    const limit = engine.capabilities.maxTextureSize;
    const ratio = Math.min(
      devicePixelRatio || 1,
      limit / Math.max(w, h),
      precise() ? Infinity : Math.min(1, Math.sqrt(2073600 / (w * h)))
    );
    engine.setDrawingBufferSize(w, h, ratio);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    request();
  }
  function visibility() {
    cancelAnimationFrame(frame);
    frame = 0;
    previous = 0;
    slowTime = slowFrames = 0;
    request();
  }
  let density;
  function watchDensity() {
    density?.removeEventListener('change', watchDensity);
    density = matchMedia('(resolution: ' + devicePixelRatio + 'dppx)');
    density.addEventListener('change', watchDensity, { once: true, signal });
    resize();
  }
  watchDensity();
  const size = new ResizeObserver(resize);
  size.observe(host);
  const intersection = new IntersectionObserver((entries) => {
    seen = entries[0].isIntersecting;
    visibility();
  });
  intersection.observe(host);
  document.addEventListener('visibilitychange', visibility, { signal });
  reduced.addEventListener('change', request, { signal });
  canvas.addEventListener(
    'webglcontextlost',
    (e) => {
      e.preventDefault();
      lost = true;
      visibility();
      notify('error');
    },
    { signal }
  );
  canvas.addEventListener(
    'webglcontextrestored',
    () => {
      lost = false;
      notify('ready');
      resize();
    },
    { signal }
  );
  function dispose() {
    if (stopped) return;
    stopped = true;
    cancelAnimationFrame(frame);
    size.disconnect();
    intersection.disconnect();
    const assets = new Set();
    scene.traverse((o) => {
      if (o.shadow) o.shadow.dispose();
      if (o.geometry) assets.add(o.geometry);
      for (const m of [].concat(o.material || [])) {
        assets.add(m);
        for (const v of Object.values(m)) if (v?.isTexture) assets.add(v);
      }
    });
    for (const asset of assets) asset.dispose();
    scene.clear();
    engine.dispose();
    engine.forceContextLoss();
    canvas.remove();
  }
  resize();
  signal.addEventListener('abort', dispose, { once: true });
  return {
    scene,
    get playable() {
      return !preparing && !lost;
    },
    prepare() {
      generation++;
      preparing = true;
      onPresented = null;
    },
    reveal(done) {
      const version = ++generation;
      engine
        .compileAsync(scene, camera)
        .then(() => {
          if (stopped || generation !== version) return;
          preparing = false;
          previous = 0;
          onPresented = done;
          request();
        })
        .catch((error) => {
          if (!stopped) {
            console.error('Voxel shader compilation', error);
            notify('error');
          }
        });
    },
    get setting() {
      return setting;
    },
    onQuality(callback) {
      qualityChanged = callback;
      callback(precise());
    },
    setQuality(next) {
      const was = precise();
      setting = next;
      slowTime = slowFrames = 0;
      if (next === 'auto') constrained = Boolean(software);
      try {
        localStorage.setItem('staticflow.lab.graphics', setting);
      } catch {}
      resize();
      if (was !== precise()) qualityChanged?.(precise());
    },
    camera,
    canvas,
    engine,
    request,
    dispose,
    start(callback) {
      paint = callback;
      request();
    }
  };
}
export function seeded(seed) {
  return () => {
    seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
    return seed / 4294967296;
  };
}
