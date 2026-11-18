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
  host.append(canvas);
  const scene = new Scene(),
    camera = new PerspectiveCamera(72, 1, 0.045, 150);
  const overlay = new Scene(),
    handCamera = new PerspectiveCamera();
  const reduced = matchMedia('(prefers-reduced-motion: reduce)');
  let frame = 0,
    previous = 0,
    seen = true,
    lost = false,
    stopped = false,
    paint;
  function request() {
    if (!frame && !stopped && !lost && seen && !document.hidden && paint)
      frame = requestAnimationFrame(draw);
  }
  function draw(now) {
    frame = 0;
    if (stopped || lost || !seen || document.hidden) return;
    const dt = previous ? Math.min((now - previous) / 1000, 0.065) : 0;
    previous = now;
    const moving = paint(dt, reduced.matches);
    engine.render(scene, camera);
    // The hand has its own depth buffer, including correct occlusion between the fingers and tool.
    handCamera.projectionMatrix.copy(camera.projectionMatrix);
    engine.autoClear = false;
    engine.clearDepth();
    engine.render(overlay, handCamera);
    engine.autoClear = true;
    if (moving) request();
  }
  function resize() {
    const w = Math.max(1, host.clientWidth),
      h = Math.max(1, host.clientHeight);
    engine.setPixelRatio(Math.min(devicePixelRatio || 1, 1.5, Math.sqrt(1850000 / (w * h))));
    engine.setSize(w, h, false);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    request();
  }
  function visibility() {
    cancelAnimationFrame(frame);
    frame = 0;
    previous = 0;
    request();
  }
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
    for (const layer of [scene, overlay])
      layer.traverse((o) => {
        if (o.shadow) o.shadow.dispose();
        if (o.geometry) assets.add(o.geometry);
        for (const m of [].concat(o.material || [])) {
          assets.add(m);
          for (const v of Object.values(m)) if (v?.isTexture) assets.add(v);
        }
      });
    for (const asset of assets) asset.dispose();
    scene.clear();
    overlay.clear();
    engine.dispose();
    engine.forceContextLoss();
    canvas.remove();
  }
  resize();
  signal.addEventListener('abort', dispose, { once: true });
  return {
    scene,
    overlay,
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
