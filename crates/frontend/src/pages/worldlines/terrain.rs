use std::collections::{BTreeMap, HashMap, HashSet};

use super::catalog::*;

pub const SIDE: i32 = 16;
pub const HEIGHT: i32 = 48;
// The authored town and open cut share a level foundation before the wilderness
// begins.
const TOWN_RADIUS: i32 = 34;
const VOLUME: usize = (SIDE * SIDE * HEIGHT) as usize;
type Chunk = Box<[u8]>;
type Key = (i32, i32);
fn address(x: i32, y: i32, z: i32) -> (Key, usize) {
    (
        (x.div_euclid(SIDE), z.div_euclid(SIDE)),
        ((y * SIDE + z.rem_euclid(SIDE)) * SIDE + x.rem_euclid(SIDE)) as usize,
    )
}
pub fn hash(seed: u32, x: i32, z: i32) -> u32 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x85ebca6b) ^ (z as u32).wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846ca68b);
    h ^ (h >> 16)
}
fn noise(seed: u32, x: i32, z: i32, scale: i32) -> f32 {
    let ix = x.div_euclid(scale);
    let iz = z.div_euclid(scale);
    let smooth = |n: i32| {
        let f = n.rem_euclid(scale) as f32 / scale as f32;
        f * f * (3. - 2. * f)
    };
    let a = smooth(x);
    let b = smooth(z);
    let v = |dx, dz| (hash(seed, ix + dx, iz + dz) & 65535) as f32 / 65535.;
    let mix = |p: f32, q: f32, t: f32| p + (q - p) * t;
    mix(mix(v(0, 0), v(1, 0), a), mix(v(0, 1), v(1, 1), a), b)
}
fn surface(seed: u32, x: i32, z: i32) -> i32 {
    let town = x.abs().max(z.abs());
    let wild =
        (11. + (noise(seed, x, z, 27) - 0.45) * 13. + (noise(seed ^ 71, x, z, 76) - 0.4) * 31.)
            .round();
    (11. + (wild - 11.) * ((town - TOWN_RADIUS) as f32 / 30.).clamp(0., 1.))
        .round()
        .clamp(5., 35.) as i32
}
struct Builder {
    x: i32,
    z: i32,
    blocks: Chunk,
}
impl Builder {
    fn set(&mut self, x: i32, y: i32, z: i32, id: u8) {
        let (lx, lz) = (x - self.x, z - self.z);
        if (0..SIDE).contains(&lx) && (0..SIDE).contains(&lz) && (0..HEIGHT).contains(&y) {
            self.blocks[((y * SIDE + lz) * SIDE + lx) as usize] = id;
        }
    }
    fn fill(&mut self, a: [i32; 3], b: [i32; 3], id: u8) {
        for x in a[0].max(self.x)..=b[0].min(self.x + SIDE - 1) {
            for z in a[2].max(self.z)..=b[2].min(self.z + SIDE - 1) {
                for y in a[1]..=b[1] {
                    self.set(x, y, z, id);
                }
            }
        }
    }
    fn tree(&mut self, x: i32, y: i32, z: i32, high: bool) {
        let h = if high { 6 } else { 4 };
        for dy in h - 2..=h + 1 {
            let r = if dy >= h { 1 } else { 2 };
            for dx in -r..=r {
                for dz in -r..=r {
                    if dx * dx + dz * dz <= r * r + 1 {
                        self.set(x + dx, y + dy, z + dz, if high { PINE } else { LEAVES });
                    }
                }
            }
        }
        self.fill([x, y, z], [x, y + h - 1, z], LOG);
    }
    fn lab(&mut self) {
        self.fill([-7, 11, -5], [7, 11, 5], TILE);
        self.fill([-7, 16, -5], [7, 16, 5], PLANK);
        self.fill([-7, 21, -5], [7, 21, 5], ROOF);
        for y in 12..21 {
            if y == 16 {
                continue;
            }
            for x in -7..=7 {
                for z in [-5, 5] {
                    let window = (13..=14).contains(&y) || (18..=19).contains(&y);
                    let id = if x == -7 || x == 7 {
                        BRICK
                    } else if window && x % 4 != 0 {
                        GLASS
                    } else {
                        PLASTER
                    };
                    self.set(x, y, z, id);
                }
            }
            for z in -4..5 {
                for x in [-7, 7] {
                    let id = if (18..=19).contains(&y) && z % 3 != 0 { GLASS } else { PLASTER };
                    self.set(x, y, z, id);
                }
            }
        }
        // Two open entrances and a switchback staircase connect the shop, lab and roof.
        self.fill([0, 12, 5], [1, 14, 5], AIR);
        self.fill([7, 17, -4], [7, 19, -3], AIR);
        self.fill([8, 16, -5], [11, 16, -4], STEEL);
        for i in 0..10 {
            let id = if i % 2 == 0 { SLAB } else { STEEL };
            for x in 8..=9 {
                self.set(x, 12 + i / 2, 5 - i, id);
            }
            for x in 10..=11 {
                self.set(x, 17 + i / 2, -3 + i, id);
            }
        }
        self.fill([6, 21, 6], [11, 21, 7], STEEL);
        self.fill([6, 21, 4], [7, 21, 6], STEEL);
        // Street-level columns support both landings. The outside rail follows the
        // upper flight.
        self.fill([11, 12, 6], [11, 20, 6], STEEL);
        self.fill([11, 12, -5], [11, 15, -5], STEEL);
        self.fill([12, 17, -5], [12, 17, -4], RAIL);
        for i in 0..10 {
            self.set(12, 17 + (i + 1) / 2, -3 + i, RAIL);
        }
        self.fill([12, 22, 6], [12, 22, 7], RAIL);
        self.fill([-5, 15, 6], [4, 15, 6], AWNING);
        for x in [-3, 3] {
            for z in [-2, 2] {
                self.set(x, 20, z, TUBE_LIGHT);
            }
        }
        self.set(-1, 19, -4, PINBOARD);
        self.set(-5, 18, -4, RADIO);
        self.set(-6, 17, 0, CABINET);
        self.set(-6, 18, 0, FAN);
        self.fill([1, 16, 1], [4, 16, 3], PLANK);
        self.fill([-5, 17, -4], [-1, 17, -4], CABINET);
        self.set(-4, 18, -4, WAVE);
        self.set(-2, 18, -4, NIXIE);
        self.fill([2, 17, -4], [5, 17, -4], CABINET);
        self.set(2, 18, -4, LEAP);
        self.set(3, 18, -4, MONITOR);
        self.set(4, 18, -4, MONITOR);
        self.set(5, 18, -4, BOOKS);
        self.fill([-6, 17, -1], [-6, 18, -1], SHELF);
        self.set(-6, 17, -2, BENCH);
        self.fill([2, 17, 0], [4, 17, 0], SOFA);
        self.set(4, 18, 0, CUSHION);
        self.set(4, 17, 1, UPA);
        self.set(2, 17, 2, TABLE);
        self.set(2, 18, 2, PHONE);
        self.set(3, 17, 2, RUG);
        self.set(3, 17, 3, RUG);
        self.set(4, 17, 3, RUG);
        self.set(4, 17, 2, RUG);
        self.set(6, 17, 3, TABLE);
        self.set(6, 18, 3, CUP);
        self.set(-6, 18, 3, CURTAIN);
        self.set(-6, 17, 3, CURTAIN);
        self.set(0, 19, -4, WATCH);
        self.fill([-5, 12, -4], [-2, 12, -4], CABINET);
        self.set(-4, 13, -4, CRT);
        self.fill([3, 12, -4], [5, 13, -4], SHELF);
        self.set(4, 14, -4, MONITOR);
        self.fill([-6, 12, 1], [-6, 13, 2], SHELF);
        self.set(-6, 14, 1, CRT);
        self.set(1, 16, 6, SIGN);
        self.fill([-6, 22, -4], [-5, 24, -3], BRICK);
        self.fill([-6, 25, -4], [-5, 25, -3], SLAB);
        self.fill([2, 22, -4], [4, 23, -2], STEEL);
        for (x, z) in [(-5, 2), (5, -2)] {
            self.set(x, 22, z, TORCH);
        }
    }
    fn town(&mut self) {
        self.fill([-28, 11, 7], [33, 11, 10], ROAD);
        self.fill([13, 11, -25], [16, 11, 22], ROAD);
        for x in (-27..32).step_by(7) {
            self.fill([x, 11, 8], [x + 2, 11, 8], STRIPE);
        }
        for x in (11..19).step_by(2) {
            self.fill([x, 11, 7], [x, 11, 10], STRIPE);
        }
        for (x, z) in [(-12, 4), (12, 12), (15, -15), (-21, 10)] {
            self.fill([x, 12, z], [x, 15, z], LOG);
            self.set(x, 16, z, LAMP);
        }
        // The neighbourhood leaves the east side open to a terraced excavation.
        for (x, z, w, d, wall) in
            [(-21, -4, 7, 7, BRICK), (-22, 13, 8, 7, PLASTER), (19, 14, 6, 6, BRICK)]
        {
            self.fill([x, 12, z], [x + w, 15, z + d], wall);
            self.fill([x + 1, 12, z + 1], [x + w - 1, 15, z + d - 1], AIR);
            self.fill([x + 1, 13, z], [x + w - 1, 14, z], GLASS);
            self.fill([x + 1, 13, z + d], [x + w - 1, 14, z + d], GLASS);
            self.fill([x + w / 2, 12, z], [x + w / 2, 13, z], AIR);
            for dy in 0..=2 {
                self.fill(
                    [x - 1 + dy, 16 + dy, z - 1],
                    [x + w + 1 - dy, 16 + dy, z + d + 1],
                    if dy == 0 { PLANK } else { BRICK },
                );
            }
        }
        for (x, z) in [(1, 14), (-5, 16), (6, 19), (-11, -10), (3, -13), (21, 8)] {
            self.tree(x, 12, z, false);
        }
        // An open cut exposes each tier. The layered ramp is walkable from the west.
        for x in 18..=32 {
            for z in -8..=4 {
                let depth = ((x - 18).min(32 - x).min(z + 8).min(4 - z) + 1).clamp(0, 5);
                self.fill([x, 12 - depth, z], [x, 12, z], AIR);
                self.set(x, 11 - depth, z, STONE);
            }
        }
        self.fill([22, 6, -6], [29, 8, -4], STONE);
        self.fill([24, 8, -7], [27, 9, -6], STONE);
        self.set(28, 9, -5, STONE);
        for (x, z, id) in [
            (23, -4, COPPER),
            (24, -4, COAL),
            (25, -4, CRYSTAL),
            (26, -4, COPPER),
            (27, -4, COAL),
            (26, -2, CRYSTAL),
        ] {
            for xx in x..=x + 1 {
                for y in 5..=8 {
                    self.set(xx, y, z, id);
                }
            }
        }
        self.set(17, 12, 2, TORCH);
        self.set(33, 12, -7, TORCH);
        self.lab();
    }
}
fn generate(seed: u32, cx: i32, cz: i32) -> Chunk {
    let mut b = Builder {
        x: cx * SIDE,
        z: cz * SIDE,
        blocks: vec![AIR; VOLUME].into_boxed_slice(),
    };
    for z in b.z..b.z + SIDE {
        for x in b.x..b.x + SIDE {
            let h = surface(seed, x, z);
            let town = x.abs().max(z.abs()) <= TOWN_RADIUS;
            for y in 0..=h {
                let r = hash(seed ^ (y as u32).wrapping_mul(15299), x, z) % 521;
                let cave =
                    !town && y > 1 && y < h - 3 && noise(seed ^ (y / 3) as u32, x, z, 9) > 0.76;
                let id = if y == 0 {
                    BEDROCK
                } else if cave {
                    AIR
                } else if y == h {
                    if h <= 8 {
                        SAND
                    } else if h > 26 {
                        SNOW
                    } else {
                        GRASS
                    }
                } else if y > h - 3 {
                    DIRT
                } else {
                    match r {
                        0..=14 => COPPER,
                        15..=46 => COAL,
                        47..=50 if y < 12 => CRYSTAL,
                        _ => STONE,
                    }
                };
                b.set(x, y, z, id);
            }
            for y in h + 1..=8 {
                b.set(x, y, z, WATER);
            }
            if !town && h > 8 && h < 25 {
                match hash(seed ^ 828, x, z) % 53 {
                    0 => b.set(x, h + 1, z, FLOWER),
                    1..=4 => b.set(x, h + 1, z, FERN),
                    _ => {},
                }
            }
        }
    }
    for gz in (b.z - 3).div_euclid(7)..=(b.z + 18).div_euclid(7) {
        for gx in (b.x - 3).div_euclid(7)..=(b.x + 18).div_euclid(7) {
            let n = hash(seed ^ 415, gx, gz);
            let x = gx * 7 + (n % 6) as i32;
            let z = gz * 7 + ((n >> 8) % 6) as i32;
            let h = surface(seed, x, z);
            if x.abs().max(z.abs()) > TOWN_RADIUS && h > 8 && h < 30 && n % 5 < 3 {
                b.tree(x, h + 1, z, h > 19);
            }
        }
    }
    if (-2..=2).contains(&cx) && (-2..=1).contains(&cz) {
        b.town();
    }
    b.blocks
}

pub struct World {
    pub seed: u32,
    pub waves: HashSet<[i32; 3]>,
    chunks: HashMap<Key, Chunk>,
    changes: HashMap<Key, BTreeMap<usize, u8>>,
}
impl World {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            waves: HashSet::from([[-4, 18, -4]]),
            chunks: HashMap::new(),
            changes: HashMap::new(),
        }
    }
    fn load(&mut self, key: Key) -> &mut Chunk {
        self.chunks.entry(key).or_insert_with(|| {
            let mut chunk = generate(self.seed, key.0, key.1);
            if let Some(changes) = self.changes.get(&key) {
                for (&i, &id) in changes {
                    chunk[i] = id;
                }
            }
            chunk
        })
    }
    pub fn get(&mut self, x: i32, y: i32, z: i32) -> u8 {
        if y < 0 {
            return BEDROCK;
        }
        if y >= HEIGHT {
            return AIR;
        }
        let (key, i) = address(x, y, z);
        self.load(key)[i]
    }
    pub fn set(&mut self, p: [i32; 3], id: u8) {
        let [x, y, z] = p;
        if !(0..HEIGHT).contains(&y) {
            return;
        }
        let (key, i) = address(x, y, z);
        if let Some(chunk) = self.chunks.get_mut(&key) {
            chunk[i] = id;
        }
        self.restore_edit(p, id);
    }
    pub fn restore_edit(&mut self, p: [i32; 3], id: u8) {
        let (key, i) = address(p[0], p[1], p[2]);
        self.changes.entry(key).or_default().insert(i, id);
        if id == WAVE {
            self.waves.insert(p);
        } else {
            self.waves.remove(&p);
        }
    }
    pub fn edits(&self) -> Vec<[i32; 4]> {
        let mut edits = Vec::new();
        for (&(x, z), chunk) in &self.changes {
            for (&i, &id) in chunk {
                let i = i as i32;
                edits.push([
                    x * SIDE + i % SIDE,
                    i / (SIDE * SIDE),
                    z * SIDE + i / SIDE % SIDE,
                    i32::from(id),
                ]);
            }
        }
        edits
    }
    pub fn retain(&mut self, cx: i32, cz: i32) {
        self.chunks
            .retain(|&(x, z), _| (cx - x).abs() <= 7 && (cz - z).abs() <= 7);
    }
    // Include the adjacent ring so visibility and ambient occlusion agree at chunk
    // seams.
    pub fn region(&mut self, cx: i32, cz: i32) -> Vec<u8> {
        self.volume(cx, cz, 1)
    }
    pub fn volume(&mut self, cx: i32, cz: i32, margin: i32) -> Vec<u8> {
        let width = SIDE + 2 * margin;
        let (x0, z0) = (cx * SIDE - margin, cz * SIDE - margin);
        let (x1, z1) = (x0 + width, z0 + width);
        let mut out = vec![0; (width * width * HEIGHT) as usize];
        for bz in z0.div_euclid(SIDE)..=(z1 - 1).div_euclid(SIDE) {
            for bx in x0.div_euclid(SIDE)..=(x1 - 1).div_euclid(SIDE) {
                let blocks = self.load((bx, bz));
                let x = x0.max(bx * SIDE);
                let n = (x1.min((bx + 1) * SIDE) - x) as usize;
                for z in z0.max(bz * SIDE)..z1.min((bz + 1) * SIDE) {
                    let mut src = (z.rem_euclid(SIDE) * SIDE + x.rem_euclid(SIDE)) as usize;
                    let mut dst = ((z - z0) * width + x - x0) as usize;
                    for _ in 0..HEIGHT {
                        out[dst..dst + n].copy_from_slice(&blocks[src..src + n]);
                        dst += (width * width) as usize;
                        src += (SIDE * SIDE) as usize;
                    }
                }
            }
        }
        out
    }
}
