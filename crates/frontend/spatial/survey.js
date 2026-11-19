import {
  BufferAttribute,
  BufferGeometry,
  DataTexture,
  InstancedMesh,
  Matrix4,
  Mesh,
  NearestFilter,
  RedFormat,
  Vector2
} from 'three';
import { faceTiles } from './atlas.js';
import { Batch } from './mesh.js';

const skylight = 0x90909090;

// The terrain worker produces a stepped surface and compact tree instances from the same seed as
// the full chunks. Only the player-edited/hand-built chunks require reading actual voxel columns.
export function surveyBuffers(samples, items) {
  const [width, stride] = samples,
    batch = new Batch();
  const water = items.findIndex((item) => item.key === 'water');
  const tiles = items.map((item, id) => faceTiles(item.form === 'liquid' ? water : id));
  for (let iz = 0; iz < width; iz++) {
    for (let ix = 0; ix < width; ix++) {
      const i = 2 + (iz * width + ix) * 2,
        h = samples[i],
        id = samples[i + 1];
      const x = (ix - width / 2) * stride,
        z = (iz - width / 2) * stride;
      const y = items[id].form === 'liquid' ? h - 0.125 : h;
      batch.face(x, y, z, stride, 0, stride, 2, tiles[id][2], null, 1, skylight);
      if (ix + 1 < width && h !== samples[i + 2]) {
        const next = samples[i + 2],
          high = h > next,
          face = high ? 0 : 1;
        batch.face(
          x + stride,
          Math.min(h, next),
          z,
          0,
          Math.abs(h - next),
          stride,
          face,
          tiles[high ? id : samples[i + 3]][face],
          null,
          1,
          skylight
        );
      }
      if (iz + 1 < width && h !== samples[i + width * 2]) {
        const next = samples[i + width * 2],
          high = h > next,
          face = high ? 4 : 5;
        batch.face(
          x,
          Math.min(h, next),
          z + stride,
          stride,
          Math.abs(h - next),
          0,
          face,
          tiles[high ? id : samples[i + width * 2 + 1]][face],
          null,
          1,
          skylight
        );
      }
    }
  }
  const pack = (values) => Uint8Array.from(values, (n) => Math.round(Math.min(1, n) * 255));
  return {
    position: Float32Array.from(batch.position),
    normal: Int8Array.from(batch.normal, (n) => n * 127),
    color: pack(batch.color),
    uv: Uint16Array.from(batch.uv, (n) => Math.round(n * 65535)),
    irradiance: pack(batch.irradiance),
    index: Uint32Array.from(batch.triangles),
    trees: samples.slice(2 + width * width * 2)
  };
}
function geometry(buffers) {
  const g = new BufferGeometry();
  for (const [key, size] of [
    ['position', 3],
    ['normal', 3],
    ['color', 3],
    ['uv', 2],
    ['irradiance', 3]
  ])
    g.setAttribute(key, new BufferAttribute(buffers[key], size, key !== 'position'));
  g.setIndex(new BufferAttribute(buffers.index, 1));
  return g;
}
function treeGeometry(log, leaf, tall) {
  const b = new Batch(),
    h = tall ? 6 : 4;
  function box(x, y, z, w, height, d, id, light) {
    const tiles = faceTiles(id);
    for (let face = 0; face < 6; face++)
      b.face(x, y, z, w, height, d, face, tiles[face], null, light, skylight);
  }
  box(0, 0, 0, 1, h - 2, 1, log, 0.85);
  // Lower canopy has a five-block cross section with clipped corners. Top leaves sit two blocks higher.
  box(-2, h - 2, -1, 5, 2, 3, leaf, 0.91);
  box(-1, h - 2, -2, 3, 2, 1, leaf, 0.91);
  box(-1, h - 2, 2, 3, 2, 1, leaf, 0.91);
  box(-1, h, -1, 3, 2, 3, leaf, 1);
  const g = b.geometry();
  g.setAttribute('irradiance', new BufferAttribute(Float32Array.from(b.irradiance), 3));
  return g;
}
export function overviewLayer(scene, foreground, items) {
  const size = 16,
    mask = new Uint8Array(size * size),
    texture = new DataTexture(mask, size, size, RedFormat);
  texture.magFilter = texture.minFilter = NearestFilter;
  const base = new Vector2(),
    origin = new Vector2(),
    matrix = new Matrix4();
  const material = foreground.clone();
  material.defines = {};
  material.onBeforeCompile = (shader) => {
    foreground.onBeforeCompile(shader);
    shader.uniforms.surveyMask = { value: texture };
    shader.uniforms.surveyBase = { value: base };
    shader.uniforms.surveyOrigin = { value: origin };
    const varyings = 'varying vec3 surveyPos, surveyNormal;\n';
    shader.vertexShader =
      varyings +
      'uniform vec2 surveyOrigin;\n' +
      shader.vertexShader.replace(
        '#include <begin_vertex>',
        `#include <begin_vertex>
      vec4 local = vec4(position,1.0);
      #ifdef USE_INSTANCING
        local = instanceMatrix * local;
      #endif
      surveyPos = local.xyz + vec3(surveyOrigin.x,0.0,surveyOrigin.y); surveyNormal = normal;`
      );
    shader.fragmentShader =
      varyings +
      'uniform sampler2D surveyMask; uniform vec2 surveyBase;\n' +
      shader.fragmentShader.replace(
        '#include <map_fragment>',
        `
      vec2 cell=floor(surveyPos.xz / 16.0) - surveyBase;
      if (all(greaterThanEqual(cell,vec2(0.0))) && all(lessThan(cell,vec2(16.0))) && texture2D(surveyMask,(cell+0.5)/16.0).r>0.5) discard;
      vec2 plane = abs(surveyNormal.y)>.5 ? vec2(surveyPos.x,-surveyPos.z) : abs(surveyNormal.x)>.5 ? surveyPos.zy : surveyPos.xy;
      vec2 uv=(floor(vMapUv*16.0)*64.0+16.0+32.0*fract(plane))/1024.0;
      diffuseColor *= textureGrad(map,uv,dFdx(plane)/32.0,dFdy(plane)/32.0);`
      );
  };
  const log = items.findIndex((i) => i.key === 'log');
  const forest = ['leaves', 'pine'].map((key, i) => {
    const id = items.findIndex((item) => item.key === key);
    return { id, geometry: treeGeometry(log, id, Boolean(i)), mesh: null };
  });
  let mesh;
  return {
    show(x, z, b) {
      const g = geometry(b);
      if (mesh) {
        mesh.geometry.dispose();
        mesh.geometry = g;
      } else {
        mesh = new Mesh(g, material);
        scene.add(mesh);
      }
      origin.set(x, z);
      mesh.position.set(x, 0, z);
      for (const type of forest) {
        let count = 0;
        for (let i = 3; i < b.trees.length; i += 4) if (b.trees[i] === type.id) count++;
        if (!type.mesh || type.mesh.instanceMatrix.count < count) {
          if (type.mesh) {
            scene.remove(type.mesh);
            type.mesh.dispose();
          }
          type.mesh = new InstancedMesh(type.geometry, material, Math.ceil((count + 1) / 64) * 64);
          scene.add(type.mesh);
        }
        type.mesh.count = count;
        type.mesh.position.set(x, 0, z);
        let slot = 0;
        for (let i = 0; i < b.trees.length; i += 4)
          if (b.trees[i + 3] === type.id) {
            matrix.makeTranslation(b.trees[i], b.trees[i + 1], b.trees[i + 2]);
            type.mesh.setMatrixAt(slot++, matrix);
          }
        type.mesh.instanceMatrix.needsUpdate = true;
        type.mesh.computeBoundingSphere();
      }
    },
    cover(cx, cz, entries) {
      base.set(cx - size / 2, cz - size / 2);
      mask.fill(0);
      for (const e of entries)
        if (e.mesh) {
          const x = e.x - base.x,
            z = e.z - base.y;
          if (x >= 0 && x < size && z >= 0 && z < size) mask[z * size + x] = 255;
        }
      texture.needsUpdate = true;
    },
    clear() {
      if (mesh) {
        scene.remove(mesh);
        mesh.geometry.dispose();
        mesh = null;
      }
      for (const f of forest)
        if (f.mesh) {
          scene.remove(f.mesh);
          f.mesh.dispose();
          f.mesh = null;
        }
    },
    dispose() {
      this.clear();
      for (const f of forest) f.geometry.dispose();
      texture.dispose();
      material.dispose();
    }
  };
}
