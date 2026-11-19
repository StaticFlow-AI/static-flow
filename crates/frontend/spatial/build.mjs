import { build } from 'esbuild';
import { mkdir, readdir, readFile, unlink, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const out = path.join(root, 'static/sf-space');
const result = await build({
  absWorkingDir: root,
  entryPoints: { lab: 'spatial/lab.js', worker: 'spatial/worker.js' },
  outdir: out,
  entryNames: '[name]-[hash]',
  chunkNames: 'shared-[hash]',
  assetNames: '[name]-[hash]',
  bundle: true,
  format: 'esm',
  splitting: true,
  minify: true,
  target: 'es2020',
  metafile: true,
  write: false,
  legalComments: 'eof'
});
await mkdir(out, { recursive: true });
// Trunk also uses this hook while watching. A byte-identical rebuild must not
// invalidate the files the watcher just consumed.
async function writeChanged(file, data) {
  const bytes = Buffer.from(data);
  try {
    if ((await readFile(file)).equals(bytes)) return;
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;
  }
  await writeFile(file, bytes);
}
const generated = new Set();
for (const file of result.outputFiles) {
  await writeChanged(file.path, file.contents);
  generated.add(path.basename(file.path));
}
const entries = {};
for (const [file, info] of Object.entries(result.metafile.outputs)) {
  if (info.entryPoint) entries[path.basename(info.entryPoint, '.js')] = path.basename(file);
}
await writeChanged(path.join(out, 'manifest.json'), JSON.stringify(entries));
generated.add('manifest.json');
for (const file of await readdir(out)) {
  if (!generated.has(file)) await unlink(path.join(out, file));
}
