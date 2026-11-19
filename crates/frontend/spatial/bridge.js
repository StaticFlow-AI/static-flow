// The Yew route owns the WASM simulation for the lifetime of a mounted game.
export function startGame(host, base, game, module, bindingsURL, notify) {
  const controller = new AbortController();
  let view;
  host.classList.add('voxel-loading');
  const report = (status) => {
    if (controller.signal.aborted) return;
    host.classList.toggle('voxel-loading', status === 'loading');
    notify(status);
  };
  async function load() {
    const manifest = new URL(base + 'manifest.json', document.baseURI);
    const response = await fetch(manifest, { signal: controller.signal, cache: 'no-cache' });
    if (!response.ok) throw new Error('Game manifest unavailable');
    const { lab: entry, worker } = await response.json();
    if (!entry || !worker) throw new Error('Game bundle missing');
    const { mount } = await import(new URL(entry, manifest).href);
    if (controller.signal.aborted) return;
    view = mount(host, game, controller.signal, report, {
      workerURL: new URL(worker, manifest),
      module,
      bindingsURL
    });
  }
  load().catch((error) => {
    if (!controller.signal.aborted) {
      console.error('Block laboratory', error);
      report('error');
    }
  });
  return {
    dispose() {
      view?.dispose();
      controller.abort();
      game.free();
    }
  };
}
