// The Yew route owns the WASM simulation for the lifetime of a mounted game.
export function startGame(host, base, game, notify) {
  const controller = new AbortController();
  let view;
  const report = (status) => {
    if (!controller.signal.aborted) notify(status);
  };
  async function load() {
    const manifest = new URL(base + 'manifest.json', document.baseURI);
    const response = await fetch(manifest, { signal: controller.signal, cache: 'no-cache' });
    if (!response.ok) throw new Error('Game manifest unavailable');
    const { lab: entry } = await response.json();
    if (!entry) throw new Error('Game bundle missing');
    const { mount } = await import(new URL(entry, manifest).href);
    if (controller.signal.aborted) return;
    view = mount(host, game, controller.signal, report);
    report('ready');
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
