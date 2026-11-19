import {
  BackSide,
  BoxGeometry,
  BufferAttribute,
  BufferGeometry,
  Color,
  DoubleSide,
  DynamicDrawUsage,
  Float32BufferAttribute,
  Fog,
  InstancedMesh,
  LineSegments,
  Mesh,
  MeshBasicMaterial,
  Object3D,
  Points,
  PointsMaterial,
  ShaderMaterial,
  SphereGeometry,
  Vector3
} from 'three';
import { seeded } from './stage.js';

const TAU = Math.PI * 2;
const clamp = (n) => Math.max(0, Math.min(1, n));
function smooth(lo, hi, n) {
  const t = clamp((n - lo) / (hi - lo));
  return t * t * (3 - 2 * t);
}
function solarDirection(angle, target) {
  const height = Math.sin(angle);
  return target.set(Math.cos(angle), height * 0.8, -height * 0.6);
}

// Each chunk fills a fixed slice of one draw call. Only entering or edited chunks read roof heights
// from Wasm. Individual falling streaks move on the GPU and remain inside their own block column.
function precipitation(scene, game, seed) {
  const vertices = 9 * 16 * 16 * 2;
  const geometry = new BufferGeometry();
  const positions = new BufferAttribute(new Float32Array(vertices * 3), 3).setUsage(DynamicDrawUsage);
  const drops = new BufferAttribute(new Float32Array(vertices * 4), 4).setUsage(DynamicDrawUsage);
  geometry.setAttribute('position', positions);
  geometry.setAttribute('drop', drops);
  const uniforms = {
    eye: { value: new Vector3() },
    clock: { value: 0 },
    density: { value: 0 },
    radius: { value: 16 },
    openView: { value: 0 },
    tint: { value: new Color() }
  };
  const material = new ShaderMaterial({
    transparent: true,
    depthWrite: false,
    uniforms,
    vertexShader: `
      attribute vec4 drop;
      uniform vec3 eye;
      uniform float clock,density,radius,openView;
      varying float alpha;
      void main() {
        float low=max(drop.x+.025,mix(eye.y-12.,0.,openView));
        float high=min(76.,eye.y+15.);
        float span=high-low;
        float distanceFromEye=length(position.xz-eye.xz);
        if(drop.x<0. || span<.25 || drop.w>density || distanceFromEye>=radius) {
          alpha=0.; gl_Position=vec4(2.,2.,2.,1.); return;
        }
        float fall=fract(drop.y-clock*(.5+floor(drop.z*4.)*.125));
        float head=low+fall*span;
        float lengthOfDrop=(.31+.28*drop.z)*(1.+.4*density);
        vec3 p=vec3(position.x-.13*position.y,mix(head,min(high,head+lengthOfDrop),position.y),
                    position.z+.045*position.y);
        alpha=(.34+.31*density)*(1.-smoothstep(radius*.62,radius,distanceFromEye))
          *smoothstep(0.,.11,density-drop.w)*smoothstep(0.,.45,head-low)*smoothstep(0.,.65,high-head);
        gl_Position=projectionMatrix*modelViewMatrix*vec4(p,1.);
      }`,
    fragmentShader: `
      uniform vec3 tint; varying float alpha;
      void main() {
        gl_FragColor=vec4(tint,alpha);
        #include <colorspace_fragment>
      }`
  });
  const lines = new LineSegments(geometry, material);
  lines.frustumCulled = false;
  lines.visible = false;
  lines.renderOrder = 4;
  scene.add(lines);
  const day = new Color('#bdd6e9'),
    night = new Color('#6c86ae');
  let width = 0,
    stride = 1,
    count = 0,
    centerX,
    centerZ;
  let slots = [],
    pending = [];
  function slotIndex(cx, cz) {
    return (((cz % width) + width) % width) * width + (((cx % width) + width) % width);
  }
  function markHidden(slot) {
    const offset = slot * count * 2 * 4;
    drops.array.fill(-1, offset, offset + count * 2 * 4);
    drops.addUpdateRange(offset, count * 2 * 4);
    drops.needsUpdate = true;
  }
  function cover(cx, cz, wide) {
    const nextWidth = wide ? 5 : 3;
    if (width === nextWidth && centerX === cx && centerZ === cz) return;
    if (width !== nextWidth) {
      width = nextWidth;
      stride = wide ? 2 : 1;
      count = (16 / stride) ** 2;
      slots = new Array(width * width);
      geometry.setDrawRange(0, slots.length * count * 2);
    }
    centerX = cx;
    centerZ = cz;
    pending = [];
    const reach = (width - 1) / 2;
    for (let z = cz - reach; z <= cz + reach; z++) {
      for (let x = cx - reach; x <= cx + reach; x++) {
        const i = slotIndex(x, z);
        if (slots[i]?.x !== x || slots[i].z !== z) {
          slots[i] = { x, z, slot: i, heights: null };
          markHidden(i);
        }
        if (!slots[i].heights) pending.push(slots[i]);
      }
    }
    pending.sort((a, b) => Math.hypot(b.x - cx, b.z - cz) - Math.hypot(a.x - cx, a.z - cz));
  }
  function load(entry) {
    const { x: cx, z: cz, slot } = entry;
    const heights = game.rain_heights(cx, cz);
    entry.heights = heights;
    const random = seeded(seed ^ Math.imul(cx, 0x85ebca6b) ^ Math.imul(cz, 0xc2b2ae35));
    const start = slot * count * 2;
    let vertex = start;
    for (let z = 0; z < 16; z += stride) {
      for (let x = 0; x < 16; x += stride) {
        const lx = x + Math.floor(random() * stride),
          lz = z + Math.floor(random() * stride);
        const wx = cx * 16 + lx + 0.18 + random() * 0.64;
        const wz = cz * 16 + lz + 0.14 + random() * 0.68;
        const height = heights[lz * 16 + lx],
          phase = random(),
          size = random(),
          density = random();
        for (let tip = 0; tip < 2; tip++, vertex++) {
          const p = vertex * 3,
            d = vertex * 4;
          positions.array[p] = wx;
          positions.array[p + 1] = tip;
          positions.array[p + 2] = wz;
          drops.array[d] = height;
          drops.array[d + 1] = phase;
          drops.array[d + 2] = size;
          drops.array[d + 3] = density;
        }
      }
    }
    positions.addUpdateRange(start * 3, count * 2 * 3);
    drops.addUpdateRange(start * 4, count * 2 * 4);
    positions.needsUpdate = drops.needsUpdate = true;
  }
  function invalidate(changes) {
    if (!width || !changes) return;
    for (let i = 0; i < changes.length; i += 4) {
      const x = Math.floor(changes[i] / 16),
        z = Math.floor(changes[i + 2] / 16);
      const entry = slots[slotIndex(x, z)];
      if (entry?.x === x && entry.z === z && entry.heights) {
        entry.heights = null;
        markHidden(entry.slot);
        pending.push(entry);
      }
    }
  }
  function heightAt(x, z) {
    const cx = Math.floor(x / 16),
      cz = Math.floor(z / 16);
    const entry = slots[slotIndex(cx, cz)];
    const heights =
      entry?.x === cx && entry.z === cz && entry.heights ? entry.heights : game.rain_heights(cx, cz);
    return heights[(z - cz * 16) * 16 + x - cx * 16];
  }
  function frame(eye, time, reduced, wet, daylight, survey) {
    lines.visible = wet > 0.015;
    if (!lines.visible) return;
    const wide = survey && eye.y > 38;
    cover(Math.floor(eye.x / 16), Math.floor(eye.z / 16), wide);
    // Bound main-thread terrain generation even when a flying observer crosses several chunks.
    if (pending.length) load(pending.pop());
    uniforms.eye.value.copy(eye);
    uniforms.clock.value = reduced ? 0 : time % 4096;
    uniforms.density.value = wet;
    uniforms.radius.value = wide ? 32 : 16;
    uniforms.openView.value = wide ? 1 : 0;
    uniforms.tint.value.copy(night).lerp(day, daylight);
  }
  return { frame, invalidate, heightAt };
}

function lightning(scene, seed, heightAt) {
  const positions = new BufferAttribute(new Float32Array(9 * 6 * 3), 3).setUsage(DynamicDrawUsage);
  const geometry = new BufferGeometry();
  geometry.setAttribute('position', positions);
  const material = new MeshBasicMaterial({
    color: 0xe4ecff,
    side: DoubleSide,
    transparent: true,
    depthWrite: false,
    fog: false
  });
  const bolt = new Mesh(geometry, material);
  bolt.visible = false;
  bolt.frustumCulled = false;
  scene.add(bolt);
  const direction = new Vector3();
  let interval = -1,
    drawn = -1,
    begins = 0,
    bearing = 0,
    distance = 0;
  function draw(eye) {
    const random = seeded(seed ^ Math.imul(interval, 0x27d4eb2d));
    const x = Math.floor(eye.x + Math.sin(bearing) * distance) + 0.5;
    const z = Math.floor(eye.z + Math.cos(bearing) * distance) + 0.5;
    const ground = heightAt(Math.floor(x), Math.floor(z));
    const height = Math.max(49, ground + 22) - ground;
    const points = [];
    for (let i = 0; i <= 7; i++) {
      const t = 1 - i / 7;
      points.push([x + (random() - 0.55) * 5 * t, ground + height * t, z + (random() - 0.5) * 5 * t]);
    }
    const sideX = Math.cos(bearing),
      sideZ = -Math.sin(bearing);
    let vertex = 0;
    function ribbon(a, b, width) {
      for (const [p, edge] of [
        [a, -1],
        [a, 1],
        [b, -1],
        [b, -1],
        [a, 1],
        [b, 1]
      ]) {
        positions.setXYZ(vertex++, p[0] + sideX * edge * width, p[1], p[2] + sideZ * edge * width);
      }
    }
    for (let i = 0; i < 7; i++) ribbon(points[i], points[i + 1], 0.12 - i * 0.009);
    const [bx, by, bz] = points[3];
    const fork = [bx + sideX * 2.7, by - 2.4, bz + sideZ * 2.7];
    ribbon(points[3], fork, 0.064);
    ribbon(fork, [bx + sideX * 4, by - 6, bz + sideZ * 4], 0.042);
    positions.needsUpdate = true;
    direction.set(x - eye.x, ground + height * 0.65 - eye.y, z - eye.z).normalize();
  }
  function frame(eye, time, reduced, thunder) {
    bolt.visible = false;
    if (reduced || thunder < 0.52) return 0;
    const next = Math.floor(time / 16);
    if (next !== interval) {
      interval = next;
      const random = seeded(seed ^ Math.imul(interval + 1, 0x85ebca6b));
      begins = 2 + random() * 8;
      bearing = random() * TAU;
      distance = 23 + random() * 14;
    }
    const age = time - next * 16 - begins;
    if (age < 0 || age > 0.65) return 0;
    if (drawn !== interval) {
      draw(eye);
      drawn = interval;
    }
    const pulse = smooth(0, 0.08, age) * (1 - smooth(0.08, 0.65, age));
    bolt.visible = true;
    material.opacity = pulse;
    return pulse;
  }
  return { frame, direction };
}

function cloudLayer(dome, geometry, material, banks, random, storm) {
  const mesh = new InstancedMesh(geometry, material, banks * 3);
  const cloud = new Object3D();
  for (let bank = 0; bank < banks; bank++) {
    const a = ((bank + random() * 0.6) / banks) * TAU;
    const d = storm ? 12 + random() * 51 : 66 + random() * 24;
    const x = Math.cos(a) * d,
      z = Math.sin(a) * d;
    const y = (storm ? 29 : 17) + random() * 12;
    const w = (storm ? 9 : 6) + random() * 8;
    for (let layer = 0; layer < 3; layer++) {
      cloud.position.set(x + (layer - 1) * w * 0.52, y + (layer === 1 ? 1.2 : 0), z + random() * 1.7);
      cloud.scale.set(w * (0.64 + random() * 0.6), 0.8 + random() * 1.3, 3 + random() * (storm ? 8 : 4));
      cloud.updateMatrix();
      mesh.setMatrixAt(bank * 3 + layer, cloud.matrix);
    }
  }
  dome.add(mesh);
  return mesh;
}

export function worldWeather(scene, game, initial) {
  const gradient = new ShaderMaterial({
    side: BackSide,
    depthWrite: false,
    fog: false,
    uniforms: {
      top: { value: new Color() },
      horizon: { value: new Color() },
      sunDir: { value: new Vector3() },
      sunGlow: { value: 1 },
      boltDir: { value: new Vector3() },
      flash: { value: 0 }
    },
    vertexShader:
      'varying vec3 ray; void main(){ray=position;gl_Position=projectionMatrix*modelViewMatrix*vec4(position,1.);}',
    fragmentShader: `varying vec3 ray; uniform vec3 top,horizon,sunDir,boltDir; uniform float sunGlow,flash;
      void main() {
        vec3 d=normalize(ray);
        vec3 sky=mix(horizon,top,pow(clamp(d.y,0.,1.),.55));
        float facing=max(0.,dot(d,sunDir));
        sky+=sunGlow*vec3(.43,.27,.12)*(pow(facing,22.)+.55*pow(facing,146.));
        sky+=flash*vec3(.26,.29,.36)*(.1+.8*pow(max(0.,dot(d,boltDir)),8.));
        gl_FragColor=vec4(sky,1.);
        #include <colorspace_fragment>
      }`
  });
  const dome = new Mesh(new SphereGeometry(127, 24, 14), gradient);
  dome.renderOrder = -4;
  scene.add(dome);
  const random = seeded(initial.seed);
  const cloudBox = new BoxGeometry();
  const normal = cloudBox.attributes.normal,
    tones = [];
  for (let i = 0; i < normal.count; i++) {
    const light = normal.getY(i) > 0.5 ? 1 : normal.getY(i) < -0.5 ? 0.68 : normal.getX(i) > 0.5 ? 0.87 : 0.8;
    tones.push(light * 0.99, light, Math.min(1, light * 1.045));
  }
  cloudBox.setAttribute('color', new Float32BufferAttribute(tones, 3));
  const clouds = cloudLayer(
    dome,
    cloudBox,
    new MeshBasicMaterial({ vertexColors: true, fog: false }),
    16,
    random,
    false
  );
  const bank = cloudLayer(
    dome,
    cloudBox,
    new MeshBasicMaterial({ vertexColors: true, fog: false, transparent: true, depthWrite: false }),
    18,
    random,
    true
  );
  const cube = new BoxGeometry();
  function orb(size, color) {
    const body = new Mesh(
      cube,
      new MeshBasicMaterial({ color, fog: false, transparent: true, depthWrite: false })
    );
    body.scale.set(size, size, 1);
    dome.add(body);
    return body;
  }
  const sun = orb(5.8, 0xffdfa3),
    moon = orb(4.1, 0xdde7fc);
  const stars = [];
  for (let i = 0; i < 210; i++) {
    const a = random() * TAU,
      b = random() * 1.5;
    stars.push(Math.cos(a) * Math.cos(b) * 105, Math.sin(b) * 105, Math.sin(a) * Math.cos(b) * 105);
  }
  const starsGeometry = new BufferGeometry();
  starsGeometry.setAttribute('position', new Float32BufferAttribute(stars, 3));
  const starMat = new PointsMaterial({
    size: 0.28,
    color: 0xe1e7f6,
    fog: false,
    transparent: true,
    depthWrite: false
  });
  dome.add(new Points(starsGeometry, starMat));
  const rain = precipitation(scene, game, initial.seed);
  const strike = lightning(scene, initial.seed, rain.heightAt);
  scene.fog = new Fog(0xb9cbd0, 41, 71);
  const noon = new Color('#5899c6'),
    dusk = new Color('#bb8b99'),
    night = new Color('#15243c');
  const haze = new Color('#d3e1df'),
    sunset = new Color('#ffc08f'),
    navy = new Color('#414f6c');
  const rainTop = new Color('#768796'),
    rainHorizon = new Color('#aab8bb');
  const stormTop = new Color('#444f64'),
    stormHorizon = new Color('#7b8a98');
  const rainNight = new Color('#252f43'),
    stormNight = new Color('#1a2233');
  const cloudDay = new Color('#fbf7ea'),
    cloudNight = new Color('#67768d'),
    cloudStorm = new Color('#83909c');
  const sunlight = new Color('#ffddaa'),
    moonlight = new Color('#b1caf6');
  const forward = new Vector3(0, 0, 1),
    stormTint = new Color(),
    horizonTint = new Color();
  const light = { step: -1, direction: new Vector3(), color: new Color(), direct: 0, ambient: 0 };
  let sample,
    sampledAt,
    survey,
    line,
    wet = 0,
    thunder = 0;
  function state(s) {
    // A deliberate choice takes effect even when the clock is paused inside the controls.
    if (!sample || !s.sky.automatic || s.sky.automatic !== sample.automatic) {
      wet = s.sky.weather === 'thunder' ? 1 : s.sky.weather === 'rain' ? 0.6 : 0;
      thunder = s.sky.weather === 'thunder' ? 1 : 0;
    }
    sample = s.sky;
    sampledAt = s.seconds;
    survey = s.survey;
    line = s.line;
  }
  state(initial);
  function frame(camera, time, dt, reduced) {
    const phase = (((sample.phase + (time - sampledAt) / sample.daySeconds) % 1) + 1) % 1;
    const angle = TAU * phase - Math.PI / 2;
    const altitude = Math.sin(angle),
      daylight = smooth(-0.16, 0.62, altitude);
    const twilight = (1 - smooth(0.04, 0.6, Math.abs(altitude))) * smooth(-0.3, -0.05, altitude);
    const overcast = sample.weather === 'thunder' ? 1 : sample.weather === 'rain' ? 0.6 : 0;
    const blend = reduced ? 1 : 1 - Math.exp(-dt * 0.8);
    wet += (overcast - wet) * blend;
    thunder += ((sample.weather === 'thunder' ? 1 : 0) - thunder) * blend;
    rain.frame(camera.position, time, reduced, wet, daylight, survey);
    const flash = strike.frame(camera.position, time, reduced, thunder);
    dome.position.copy(camera.position);
    dome.scale.setScalar(survey ? Math.max(1, Math.min(4.6, (camera.far * 0.9) / 127)) : 1);
    const sky = gradient.uniforms;
    sky.top.value
      .copy(night)
      .lerp(noon, daylight)
      .lerp(dusk, twilight * 0.55);
    sky.horizon.value
      .copy(navy)
      .lerp(haze, daylight)
      .lerp(sunset, twilight * 0.88);
    if (line === 'echo') {
      sky.top.value.lerp(night, 0.42);
      sky.horizon.value.lerp(navy, 0.35);
    } else if (line === 'beta') {
      sky.top.value.lerp(dusk, 0.3 * daylight);
      sky.horizon.value.lerp(sunset, 0.4 * daylight);
    }
    stormTint
      .copy(rainNight)
      .lerp(rainTop, daylight)
      .lerp(stormNight, thunder * (1 - daylight))
      .lerp(stormTop, thunder * daylight);
    horizonTint
      .copy(navy)
      .lerp(rainHorizon, daylight)
      .lerp(stormNight, thunder * (1 - daylight))
      .lerp(stormHorizon, thunder * daylight);
    const cloudCover = smooth(0, 1, wet * 1.4);
    sky.top.value.lerp(stormTint, cloudCover);
    sky.horizon.value.lerp(horizonTint, cloudCover);
    sky.sunGlow.value = daylight * (1 - cloudCover);
    sky.flash.value = flash;
    sky.boltDir.value.copy(strike.direction);
    scene.fog.color.copy(sky.horizon.value);
    scene.fog.near = survey ? 115 - 42 * wet : 41 - 23 * wet;
    scene.fog.far = survey ? 275 - 70 * wet : 71 - 22 * wet;
    solarDirection(angle, sky.sunDir.value);
    sun.position.copy(sky.sunDir.value).multiplyScalar(100);
    moon.position.copy(sun.position).negate();
    sun.quaternion.setFromUnitVectors(forward, sky.sunDir.value);
    moon.quaternion.copy(sun.quaternion);
    sun.material.opacity = smooth(-0.075, 0.03, altitude) * (1 - cloudCover * 0.98);
    moon.material.opacity = smooth(-0.075, 0.03, -altitude) * (1 - cloudCover);
    starMat.opacity = (1 - smooth(-0.3, 0.01, altitude)) * (1 - cloudCover);
    clouds.material.color
      .copy(cloudNight)
      .lerp(cloudDay, daylight)
      .lerp(stormTint, cloudCover * 0.7);
    bank.material.color.copy(cloudNight).lerp(cloudStorm, daylight).lerp(stormTint, 0.38);
    bank.material.opacity = cloudCover * 0.96;
    bank.visible = cloudCover > 0.01;
    if (!reduced) {
      clouds.rotation.y = (time * 0.00045) % TAU;
      bank.rotation.y = (time * 0.0013) % TAU;
    }
    const step = Math.floor(phase * 640);
    if (step !== light.step) {
      light.step = step;
      solarDirection((step / 640) * TAU - Math.PI / 2, light.direction);
      if (light.direction.y < 0) light.direction.negate();
      light.direction.y = Math.max(0.08, light.direction.y);
      light.direction.normalize();
    }
    light.color.copy(moonlight).lerp(sunlight, daylight);
    light.direct = (0.06 + daylight * 1.02) * (1 - cloudCover * 0.92);
    light.ambient = (0.17 + daylight * 0.65) * (1 - wet * 0.35) + flash * 0.085;
    return light;
  }
  return { frame, state, invalidate: rain.invalidate };
}
