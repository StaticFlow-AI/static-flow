import { surveyBuffers } from './survey.js';
import { chunkBuffers } from './mesh.js';

let mesher, items;
self.onmessage = async ({ data }) => {
  try {
    if (data.type === 'init') {
      const bindings = await import(data.bindingsURL);
      bindings.initSync({ module: data.module });
      mesher = new bindings.ChunkMesher(data.seed);
      mesher.edit(data.edits);
      items = data.items;
      self.postMessage({ type: 'ready' });
    } else if (data.type === 'survey') {
      if (data.edits.length) mesher.edit(data.edits);
      const buffers = surveyBuffers(mesher.survey(data.x, data.z), items);
      self.postMessage(
        { type: 'survey', x: data.x, z: data.z, buffers },
        Object.values(buffers).map((a) => a.buffer)
      );
    } else if (data.type === 'build') {
      // Edits and builds share one ordered channel, including changes made while a job was running.
      if (data.edits.length) mesher.edit(data.edits);
      const buffers = chunkBuffers(mesher.build(data.x, data.z), items);
      self.postMessage(
        { type: 'chunk', ticket: data.ticket, buffers },
        Object.values(buffers)
          .filter(ArrayBuffer.isView)
          .map((a) => a.buffer)
      );
    }
  } catch (error) {
    self.postMessage({ type: 'error', message: String(error?.stack || error) });
  }
};
