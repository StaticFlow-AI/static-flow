// The router owns both the native match and the WebGL scene for this visit.
export function startFreight(host, base, sim, notify) {
  const controller = new AbortController();
  let view;
  const url = new URL(base + 'sf-space/manifest.json', document.baseURI);
  async function load() {
    const response = await fetch(url, { signal: controller.signal, cache: 'no-cache' });
    if (!response.ok) throw Error('Missing game manifest');
    const { freight } = await response.json();
    if (!freight) throw Error('Missing freight entry');
    const { mount } = await import(new URL(freight, url).href);
    if (controller.signal.aborted) return;
    view = await mount(host, sim, new URL(base + 'freight/', document.baseURI), controller.signal);
    if (controller.signal.aborted) view?.dispose();
    else notify('ready');
  }
  load().catch((error) => {
    if (!controller.signal.aborted) {
      console.error('Freight deck', error);
      notify('error');
    }
  });
  return {
    dispose() {
      controller.abort();
      view?.dispose();
      sim.free();
    }
  };
}
