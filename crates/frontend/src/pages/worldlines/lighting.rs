//! The mesh worker owns a second deterministic terrain cache. Only committed
//! block edits cross threads.
use std::collections::VecDeque;

use wasm_bindgen::prelude::*;

use super::{
    catalog::{AIR, BEACON, BEDROCK, ITEMS, LAMP, TORCH, TUBE_LIGHT},
    terrain::{World, HEIGHT, SIDE},
};

// Include the full light radius plus one cell for corner sampling, even across
// four-way seams.
const LIGHT: u8 = 9;
const MARGIN: i32 = LIGHT as i32 + 1;
const WIDTH: usize = (SIDE + 2 * MARGIN) as usize;
const PLANE: usize = WIDTH * WIDTH;
const NEIGHBORS: [isize; 6] =
    [1, -1, PLANE as isize, -(PLANE as isize), WIDTH as isize, -(WIDTH as isize)];
const XYZ: [isize; 3] = [1, PLANE as isize, WIDTH as isize];
// Winding matches Batch.face. Each byte indexes the four x/y/z corner bits.
const CORNERS: [[u8; 4]; 6] =
    [[5, 1, 3, 7], [0, 4, 6, 2], [6, 7, 3, 2], [0, 1, 5, 4], [4, 5, 7, 6], [1, 0, 2, 3]];

struct LightVolume {
    blocks: Vec<u8>,
    sky: Vec<u8>,
    lamps: Vec<u8>,
    opaque: [bool; ITEMS.len()],
}
impl LightVolume {
    fn new(world: &mut World, cx: i32, cz: i32) -> Self {
        let mut blocks = vec![AIR; PLANE * (HEIGHT as usize + 2)];
        blocks[..PLANE].fill(BEDROCK);
        blocks[PLANE..PLANE * (HEIGHT as usize + 1)].copy_from_slice(&world.volume(cx, cz, MARGIN));
        let opaque = std::array::from_fn(|i| ITEMS[i].form == "cube");
        let mut sky = vec![0; blocks.len()];
        let mut lamps = vec![0; blocks.len()];
        let mut sources = VecDeque::new();
        for col in 0..PLANE {
            let mut open = true;
            for y in (1..HEIGHT as usize + 2).rev() {
                let index = y * PLANE + col;
                let id = blocks[index];
                open &= !opaque[id as usize];
                if open {
                    sky[index] = LIGHT;
                }
                let glow = match id {
                    TORCH => 7,
                    TUBE_LIGHT | LAMP | BEACON => LIGHT,
                    _ => 0,
                };
                if glow > 0 {
                    lamps[index] = glow;
                    sources.push_back(index);
                }
            }
        }
        Self::spread(&blocks, &opaque, &mut lamps, &mut sources);
        // Seed only the boundary of direct skylight. Empty sky interiors already have
        // their final value.
        for i in PLANE..blocks.len() - PLANE {
            if sky[i] == 0
                && !opaque[blocks[i] as usize]
                && NEIGHBORS
                    .iter()
                    .any(|&d| sky[i.wrapping_add_signed(d)] == LIGHT)
            {
                sky[i] = LIGHT - 1;
                sources.push_back(i);
            }
        }
        Self::spread(&blocks, &opaque, &mut sky, &mut sources);
        Self {
            blocks,
            sky,
            lamps,
            opaque,
        }
    }
    fn spread(
        blocks: &[u8],
        opaque: &[bool; ITEMS.len()],
        values: &mut [u8],
        queue: &mut VecDeque<usize>,
    ) {
        while let Some(i) = queue.pop_front() {
            let level = values[i] - 1;
            if level == 0 {
                continue;
            }
            for d in NEIGHBORS {
                let j = i.wrapping_add_signed(d);
                if j >= blocks.len() || values[j] >= level || opaque[blocks[j] as usize] {
                    continue;
                }
                // The padding is at least one cell wider than the propagation radius.
                values[j] = level;
                queue.push_back(j);
            }
        }
    }
    fn sample(&self, front: usize, face: usize) -> (u32, u32) {
        let normal_axis = face / 2;
        let (u, v) = match normal_axis {
            0 => (1, 2),
            1 => (0, 2),
            _ => (0, 1),
        };
        let mut ao = 0;
        let mut lighting = 0;
        for (corner, &bits) in CORNERS[face].iter().enumerate() {
            let du = if bits & (1 << u) != 0 { XYZ[u] } else { -XYZ[u] };
            let dv = if bits & (1 << v) != 0 { XYZ[v] } else { -XYZ[v] };
            let sides = [
                front.wrapping_add_signed(du),
                front.wrapping_add_signed(dv),
                front.wrapping_add_signed(du + dv),
            ];
            let [a, b, c] = sides.map(|i| self.opaque[self.blocks[i] as usize]);
            let shade = if a && b { 3 } else { u32::from(a) + u32::from(b) + u32::from(c) };
            ao |= shade << (corner * 2);
            let mut day = self.sky[front];
            let mut lamp = self.lamps[front];
            let mut count = 1;
            for (j, cell) in sides.into_iter().enumerate() {
                if !self.opaque[self.blocks[cell] as usize] && !(j == 2 && a && b) {
                    day += self.sky[cell];
                    lamp += self.lamps[cell];
                    count += 1;
                }
            }
            let value = (((day + count / 2) / count) << 4) | ((lamp + count / 2) / count);
            lighting |= u32::from(value) << (corner * 8);
        }
        (ao, lighting)
    }
    fn faces(&self) -> Vec<u32> {
        let mut out = Vec::with_capacity(4096);
        for y in 0..HEIGHT as usize {
            for z in 0..SIDE as usize {
                for x in 0..SIDE as usize {
                    let i = (y + 1) * PLANE + (z + MARGIN as usize) * WIDTH + x + MARGIN as usize;
                    let id = self.blocks[i];
                    if id == AIR {
                        continue;
                    }
                    let form = ITEMS[id as usize].form;
                    let header = (x | z << 4 | y << 8 | usize::from(id) << 17) as u32;
                    if form == "prop" {
                        let value = self.sky[i] << 4 | self.lamps[i];
                        out.extend([header | 6 << 14, u32::from_le_bytes([value; 4])]);
                        continue;
                    }
                    for (face, &d) in NEIGHBORS.iter().enumerate() {
                        let front = i.wrapping_add_signed(d);
                        let neighbor = self.blocks[front];
                        let partial = form == "slab" || form == "liquid";
                        if (self.opaque[neighbor as usize] || (neighbor == id && form != "slab"))
                            && !(face == 2 && partial)
                        {
                            continue;
                        }
                        let (ao, lighting) = self.sample(front, face);
                        out.extend([header | (face as u32) << 14 | ao << 24, lighting]);
                    }
                }
            }
        }
        out
    }
}

#[wasm_bindgen]
pub struct ChunkMesher {
    world: World,
}
#[wasm_bindgen]
impl ChunkMesher {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        Self {
            world: World::new(seed),
        }
    }
    pub fn edit(&mut self, edits: &[i32]) {
        for &[x, y, z, id] in edits.as_chunks::<4>().0 {
            if (0..HEIGHT).contains(&y) && (0..ITEMS.len() as i32).contains(&id) {
                self.world.set([x, y, z], id as u8);
            }
        }
    }
    pub fn build(&mut self, cx: i32, cz: i32) -> Vec<u32> {
        self.world.retain(cx, cz);
        LightVolume::new(&mut self.world, cx, cz).faces()
    }
}

#[cfg(test)]
mod tests {
    use super::{super::catalog::STONE, *};
    #[test]
    fn lighting_matches_at_chunk_seams_and_a_new_wall_blocks_the_lamp() {
        let mut w = World::new(6);
        // Enclose a tunnel crossing a chunk boundary. Its midpoint must receive no
        // skylight.
        for x in 3..=28 {
            for y in 30..=34 {
                for z in -2..=2 {
                    let wall = x == 3 || x == 28 || y == 30 || y == 34 || z == -2 || z == 2;
                    w.set([x, y, z], if wall { STONE } else { AIR });
                }
            }
        }
        w.set([13, 32, 0], TORCH);
        let light_at = |w: &mut World, cx: i32, x: i32| {
            let volume = LightVolume::new(w, cx, 0);
            let i = 33 * PLANE + MARGIN as usize * WIDTH + (x - cx * SIDE + MARGIN) as usize;
            (volume.sky[i], volume.lamps[i])
        };
        assert_eq!(light_at(&mut w, 0, 16), (0, 4));
        assert_eq!(light_at(&mut w, 1, 16), (0, 4));
        for y in 31..=33 {
            for z in -1..=1 {
                w.set([15, y, z], STONE);
            }
        }
        assert_eq!(light_at(&mut w, 1, 16), (0, 0));
    }
}
