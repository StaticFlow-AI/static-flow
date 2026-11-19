import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createReadStream } from 'node:fs';
import { cp, mkdir, mkdtemp, rename, rm, stat, symlink } from 'node:fs/promises';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import adaptUI from './adapt-ui.mjs';
import adaptContent from './adapt-content.mjs';
import adaptShared from './adapt-shared.mjs';

const worldDir = dirname(fileURLToPath(import.meta.url));
const revision = '70764a94756cc4de564e493c8ba46c4b5fe03ed1';
const archiveDigest = '7d43219dbb3ee8d7ebec9e7140d9994f8a7e4c841abc7ab4ea445fae65ed3631';
const cache = join(process.env.XDG_CACHE_HOME || join(homedir(), '.cache'), 'staticflow', 'world');
const upstream = join(cache, [revision, process.platform, process.arch, process.versions.modules].join('-'));

async function prepareUpstream() {
  if (await stat(upstream).then(s => s.isDirectory(), () => false)) return;
  const pending = await mkdtemp(join(cache, 'upstream-'));
  try {
    const archive = join(pending, 'source.tar.gz');
    execFileSync('curl', ['--fail', '--location', '--silent', '--show-error', '--retry', '3',
      '-o', archive, `https://codeload.github.com/brunosimon/folio-2025/tar.gz/${revision}`], { stdio: 'inherit' });
    const hash = createHash('sha256');
    for await (const chunk of createReadStream(archive)) hash.update(chunk);
    if (hash.digest('hex') !== archiveDigest) throw new Error('Folio upstream archive SHA-256 mismatch');

    const names = ['sources', 'static', 'license.md', 'package.json', 'package-lock.json'];
    execFileSync('tar', ['-xzf', archive, '--strip-components=1', '--exclude=*.wav', '-C', pending,
      ...names.map(name => `folio-2025-${revision}/${name}`)], { stdio: 'inherit' });
    await rm(archive);
    execFileSync('npm', ['ci', '--ignore-scripts', '--no-audit', '--no-fund'], {
      cwd: pending, stdio: 'inherit', shell: process.platform === 'win32',
    });
    // Publish only complete inputs. A simultaneous build can win this rename.
    try {
      await rename(pending, upstream);
    } catch (error) {
      if (!['EEXIST', 'ENOTEMPTY'].includes(error.code)) throw error;
    }
  } finally {
    await rm(pending, { recursive: true, force: true });
  }
}

// Trunk passes its private staging directory. A manual build can use an empty
// directory to serve and exercise the world independently from the rest of the site.
const distribution = process.argv[2] || process.env.TRUNK_STAGING_DIR;
if (!distribution) throw new Error('Expected a distribution directory');
await mkdir(cache, { recursive: true });
await prepareUpstream();
const work = await mkdtemp(join(cache, 'build-'));
try {
  for (const dir of ['sources', 'static']) await cp(join(upstream, dir), join(work, dir), { recursive: true });
  await cp(join(upstream, 'license.md'), join(work, 'static', 'folio-license.txt'));
  await cp(join(upstream, 'package.json'), join(work, 'package.json'));
  await symlink(join(upstream, 'node_modules'), join(work, 'node_modules'), 'junction');
  await cp(join(worldDir, 'overlay'), work, { recursive: true });
  await adaptUI(work);
  await adaptContent(work);
  await adaptShared(work);
  for (const dir of ['projects/images', 'lab/images', 'career', 'timeMachine']) {
    await rm(join(work, 'static', dir), { recursive: true });
  }

  // The world has a public, fixed configuration independent of the shell
  // running Trunk. In particular no operator VITE_* values enter this bundle.
  const env = Object.fromEntries(Object.entries(process.env).filter(([key]) => !key.startsWith('VITE_')));
  execFileSync(process.execPath, [join(work, 'node_modules/vite/bin/vite.js'), 'build'], {
    cwd: work, stdio: 'inherit',
    env: { ...env, VITE_COMPRESSED: '1', VITE_WHISPERS_COUNT: '30', VITE_MUSIC: '1',
      VITE_STATICFLOW_API_BASE: process.env.STATICFLOW_API_BASE || '/api' },
  });
  await cp(join(work, 'dist'), join(resolve(distribution), 'static', 'world'), { recursive: true });
} finally {
  await rm(work, { recursive: true, force: true });
}
