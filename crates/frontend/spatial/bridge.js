// A route may unmount while its 3D module or GLB is still in flight.
export function startSpace(host, base, kind, emit, walker, initial) {
  const controller = new AbortController();
  let view,
    state = initial;
  const notify = (type, value = '') => {
    if (!controller.signal.aborted) emit(type, String(value));
  };
  const load = async () => {
    const asset = window.sfSpatialEntries?.[kind];
    if (!asset) throw new Error('Spatial entry is missing');
    const { mount } = await import(new URL(base + asset, document.baseURI).href);
    if (controller.signal.aborted) return;
    const mounted = await mount(host, notify, walker, controller.signal);
    if (controller.signal.aborted) {
      mounted?.dispose();
      return;
    }
    view = mounted;
    view.sync(state);
    notify('ready');
  };
  load().catch((error) => {
    if (!controller.signal.aborted) {
      console.warn('StaticFlow 3D', error);
      notify('error');
    }
  });
  return {
    sync(next) {
      state = next;
      view?.sync(next);
    },
    dispose() {
      controller.abort();
      view?.dispose();
      view = null;
      walker?.free();
    }
  };
}
