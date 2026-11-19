import { Mesh } from 'three';
import { overviewLayer } from './survey.js';
import { uploadedGeometry } from './mesh.js';

// One outstanding job bounds memory and lets a nearby edit preempt background exploration.
export function streamChunks(view, game, catalog, material, assets, signal, report) {
  const worker = new Worker(assets.workerURL, { type: 'module', name: 'voxel-terrain' });
  const entries = new Map();
  const overview = overviewLayer(view.scene, material[0], catalog.items);
  let flying = false,
    surveyPending = false,
    surveyX,
    surveyZ,
    surveyDirty = false,
    lastSurvey = 0;
  const radius = matchMedia('(pointer:coarse)').matches ? 3 : 4;
  let cx,
    cz,
    active,
    completed,
    ready = false,
    stopped = false,
    ticket = 0,
    initial = new Set(),
    edits = [];
  const key = (x, z) => x + ',' + z;
  function fail(message) {
    if (stopped || signal.aborted) return;
    stopped = true;
    worker.terminate();
    console.error('Voxel terrain worker', message);
    report('error');
  }
  worker.onerror = (e) => fail(e.message);
  worker.onmessage = ({ data }) => {
    if (stopped || signal.aborted) return;
    if (data.type === 'ready') ready = true;
    else if (data.type === 'error') return fail(data.message);
    else if (data.type === 'survey') {
      surveyPending = false;
      if (flying) {
        surveyX = data.x;
        surveyZ = data.z;
        overview.show(data.x, data.z, data.buffers);
        overview.cover(cx, cz, entries.values());
      }
    } else if (active && data.ticket === active.ticket) completed = data;
    view.request();
  };
  signal.addEventListener(
    'abort',
    () => {
      stopped = true;
      worker.terminate();
      completed = null;
      overview.dispose();
    },
    { once: true }
  );
  const initialEdits = game.world_edits();
  worker.postMessage(
    {
      type: 'init',
      module: assets.module,
      bindingsURL: assets.bindingsURL,
      seed: JSON.parse(game.snapshot()).seed,
      edits: initialEdits,
      items: catalog.items
    },
    [initialEdits.buffer]
  );
  function center(x, z) {
    const nx = Math.floor(x / 16),
      nz = Math.floor(z / 16);
    if (nx === cx && nz === cz) return;
    const first = cx === undefined || Math.max(Math.abs(cx - nx), Math.abs(cz - nz)) > radius;
    cx = nx;
    cz = nz;
    game.trim(cx, cz);
    for (const [k, e] of entries) {
      if (Math.max(Math.abs(e.x - cx), Math.abs(e.z - cz)) > radius) {
        if (e.mesh) {
          view.scene.remove(e.mesh);
          e.mesh.geometry.dispose();
        }
        entries.delete(k);
        initial.delete(k);
      }
    }
    if (flying) overview.cover(cx, cz, entries.values());
    if (first) {
      initial = new Set();
      for (const a of [Math.floor((x - 5) / 16), Math.floor((x + 5) / 16)])
        for (const b of [Math.floor((z - 5) / 16), Math.floor((z + 5) / 16)])
          if (!entries.get(key(a, b))?.mesh) initial.add(key(a, b));
      if (initial.size) {
        view.prepare();
        report('loading');
      }
    }
    for (let dx = -radius; dx <= radius; dx++)
      for (let dz = -radius; dz <= radius; dz++) {
        if (dx * dx + dz * dz > (radius + 0.45) ** 2) continue;
        const x = cx + dx,
          z = cz + dz,
          k = key(x, z);
        if (!entries.has(k)) entries.set(k, { k, x, z, dirty: true, mesh: null });
      }
  }
  function flush() {
    const changes = game.changes();
    if (!changes.length) return;
    surveyDirty = true;
    for (let i = 0; i < changes.length; i += 4) {
      const [x, y, z, id] = changes.subarray(i, i + 4);
      edits.push(x, y, z, id);
      // A wall or lamp alters flood lighting up to nine cells away, including the corner samples.
      for (let bx = Math.floor((x - 10) / 16); bx <= Math.floor((x + 10) / 16); bx++)
        for (let bz = Math.floor((z - 10) / 16); bz <= Math.floor((z + 10) / 16); bz++) {
          const e = entries.get(key(bx, bz));
          if (e) {
            e.dirty = true;
            e.priority = bx === Math.floor(x / 16) && bz === Math.floor(z / 16) ? -200 : -100;
          }
        }
    }
    view.request();
    return changes;
  }
  function frame(x, z, inSurvey = false) {
    if (flying !== inSurvey) {
      flying = inSurvey;
      if (!flying) {
        overview.clear();
        surveyX = surveyZ = undefined;
      }
    }
    center(x, z);
    if (stopped || !ready) return;
    if (completed) {
      const { entry } = active;
      // Present completed work even if more water changed mid-build. A dirty entry stays in
      // the queue for the newer revision, instead of starving a continuously flowing channel.
      if (entries.get(entry.k) === entry) {
        const g = uploadedGeometry(completed.buffers);
        if (entry.mesh) {
          entry.mesh.geometry.dispose();
          entry.mesh.geometry = g;
        } else {
          entry.mesh = new Mesh(g, material);
          entry.mesh.position.set(entry.x * 16, 0, entry.z * 16);
          entry.mesh.castShadow = entry.mesh.receiveShadow = true;
          entry.mesh.updateMatrix();
          entry.mesh.matrixAutoUpdate = false;
          view.scene.add(entry.mesh);
          if (flying) overview.cover(cx, cz, entries.values());
        }
        if (Math.abs(entry.x - cx) <= 1 && Math.abs(entry.z - cz) <= 1)
          view.engine.shadowMap.needsUpdate = true;
        if (initial.delete(entry.k) && initial.size === 0) view.reveal(() => report('ready'));
      }
      active = null;
      completed = null;
    }
    if (flying && !surveyPending) {
      const x0 = Math.round(x / 32) * 32,
        z0 = Math.round(z / 32) * 32;
      const now = performance.now();
      if (x0 !== surveyX || z0 !== surveyZ || (surveyDirty && now - lastSurvey > 1600)) {
        surveyDirty = false;
        surveyPending = true;
        lastSurvey = now;
        const changes = Int32Array.from(edits);
        edits = [];
        worker.postMessage({ type: 'survey', x: x0, z: z0, edits: changes }, [changes.buffer]);
      }
    }
    if (active) return;
    let next,
      score = Infinity;
    for (const e of entries.values()) {
      if (!e.dirty) continue;
      const rank =
        (e.priority ?? 0) +
        (initial.has(e.k) ? -300 : 0) +
        (e.x + 0.5 - x / 16) ** 2 +
        (e.z + 0.5 - z / 16) ** 2;
      if (rank < score) {
        next = e;
        score = rank;
      }
    }
    if (!next) return;
    next.dirty = false;
    next.priority = 0;
    active = { entry: next, ticket: ++ticket };
    const changes = Int32Array.from(edits);
    edits = [];
    worker.postMessage({ type: 'build', ticket, x: next.x, z: next.z, edits: changes }, [changes.buffer]);
  }
  return { frame, flush };
}
