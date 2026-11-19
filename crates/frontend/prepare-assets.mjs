import { execFileSync } from 'node:child_process';
import { cpSync, existsSync, lstatSync, readdirSync, readFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';

const removeDirectory = { recursive: true, force: true };

function pruneRemoved(source, output) {
  if (!existsSync(source)) {
    rmSync(output, removeDirectory);
  } else if (lstatSync(output).isDirectory()) {
    for (const name of readdirSync(output)) {
      pruneRemoved(join(source, name), join(output, name));
    }
  }
}

switch (process.argv[2]) {
  case 'snippets': {
    const profile = process.env.TRUNK_PROFILE;
    if (profile !== 'debug' && profile !== 'release') throw new Error('Missing Trunk build profile');
    const metadata = execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
      cwd: process.env.TRUNK_SOURCE_DIR,
      encoding: 'utf8',
    });
    // Trunk copies and preloads every snippet it finds in this shared build cache.
    // wasm-bindgen regenerates the current module's snippets on each invocation.
    rmSync(join(JSON.parse(metadata).target_directory, 'wasm-bindgen', profile, 'snippets'), removeDirectory);
    break;
  }
  case 'distribution': {
    const [source, output] = process.argv.slice(3);
    const dist = output ?? process.env.TRUNK_STAGING_DIR;
    const previousStyles = new Set();
    if (output) {
      const index = join(output, 'index.html');
      if (existsSync(index)) {
        const html = readFileSync(index, 'utf8');
        if (/static-flow-frontend-[\da-f]{1,16}\.js/.test(html)) {
          for (const name of html.match(/\bstyles-[\da-f]{1,16}\.css\b/g) ?? []) previousStyles.add(name);
        }
      }
      cpSync(source, output, { recursive: true });
    }
    // Older checkouts can still have this generated directory underneath static/.
    rmSync(join(dist, 'static', 'sf-space'), removeDirectory);
    if (!output) break;

    // A custom output can also contain bundles from the previous release.
    const bundle = /^static-flow-frontend-[\da-f]{1,16}(\.js|_bg\.wasm)$/;
    for (const entry of readdirSync(output, { withFileTypes: true })) {
      const name = entry.name.replace(/\.(br|gz)$/, '');
      if (entry.isFile() && (bundle.test(name) || previousStyles.has(name))) {
        pruneRemoved(join(source, entry.name), join(output, entry.name));
      }
    }
    const snippets = join(output, 'snippets');
    if (existsSync(snippets)) {
      for (const name of readdirSync(snippets)) {
        if (/^static-flow-frontend-[\da-f]{1,16}$/.test(name)) {
          pruneRemoved(join(source, 'snippets', name), join(snippets, name));
        }
      }
    }
    break;
  }
  default:
    throw new Error('Expected snippets or distribution');
}
