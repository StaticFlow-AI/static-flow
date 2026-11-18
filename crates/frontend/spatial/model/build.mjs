import { mkdtemp, mkdir, writeFile, rm } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { GLTFExporter } from 'three/addons/exporters/GLTFExporter.js';
import { createLab } from './laboratory.mjs';
import { createMembers } from './members.mjs';
import { bake } from './kit.mjs';

// The exporter uses the browser FileReader API for its binary container.
globalThis.FileReader = class {
  readAsArrayBuffer(blob) {
    blob.arrayBuffer().then((result) => {
      this.result = result;
      this.onloadend?.();
    });
  }
};
const dest = fileURLToPath(new URL('../models/', import.meta.url));
const pack = fileURLToPath(new URL('../../node_modules/gltfpack/cli.js', import.meta.url));
const temp = await mkdtemp(join(tmpdir(), 'staticflow-gltf-'));
try {
  await mkdir(dest, { recursive: true });
  for (const [name, make] of [
    ['future-gadget-lab', createLab],
    ['lab-members', createMembers]
  ]) {
    const scene = make();
    bake(scene);
    const input = join(temp, name + '.glb');
    await writeFile(input, new Uint8Array(await new GLTFExporter().parseAsync(scene, { binary: true })));
    execFileSync(
      process.execPath,
      [
        pack,
        '-i',
        input,
        '-o',
        join(dest, name + '.glb'),
        '-cc',
        '-vp',
        '16',
        ...(name === 'future-gadget-lab' ? ['-kv', '-vtf'] : []),
        '-kn',
        '-km',
        '-ke'
      ],
      { stdio: 'inherit' }
    );
  }
} finally {
  await rm(temp, { recursive: true });
}
