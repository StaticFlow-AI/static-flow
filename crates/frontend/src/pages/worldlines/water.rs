use std::collections::{HashSet, VecDeque};

use super::{
    catalog::{is_water, AIR, FALLING_WATER, ITEMS, WATER, WATER_1, WATER_7, WATER_LEVELS},
    terrain::{World, HEIGHT},
};

const STEP: f32 = 0.20;
const CELLS_PER_STEP: usize = 384;
const SIDES: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
type Pos = [i32; 3];

#[derive(Default)]
pub struct WaterFlow {
    pending: VecDeque<Pos>,
    queued: HashSet<Pos>,
    elapsed: f32,
}

impl WaterFlow {
    fn enqueue(&mut self, p: Pos) {
        if (1..HEIGHT).contains(&p[1])
            && p[0].unsigned_abs() < 1_000_000
            && p[2].unsigned_abs() < 1_000_000
            && self.queued.insert(p)
        {
            self.pending.push_back(p);
        }
    }

    pub fn pending(&self) -> Vec<Pos> {
        self.pending.iter().copied().collect()
    }

    pub fn resume(&mut self, cells: Vec<Pos>) {
        self.pending.clear();
        self.queued.clear();
        self.elapsed = 0.;
        for p in cells {
            self.enqueue(p);
        }
    }

    pub fn wake(&mut self, [x, y, z]: Pos) {
        self.enqueue([x, y, z]);
        self.enqueue([x, y - 1, z]);
        self.enqueue([x, y + 1, z]);
        for [dx, dz] in SIDES {
            self.enqueue([x + dx, y, z + dz]);
            // Adding/removing support lets the water directly above this edit
            // start/stop feeding its horizontal neighbors.
            self.enqueue([x + dx, y + 1, z + dz]);
        }
    }

    pub fn advance(&mut self, world: &mut World, dt: f32, dirty: &mut Vec<Pos>) {
        if self.pending.is_empty() {
            self.elapsed = 0.;
            return;
        }
        self.elapsed += dt.clamp(0., STEP);
        if self.elapsed < STEP {
            return;
        }
        self.elapsed -= STEP;
        let mut updates = Vec::new();
        for _ in 0..self.pending.len().min(CELLS_PER_STEP) {
            let Some(p) = self.pending.pop_front() else { break };
            self.queued.remove(&p);
            let [x, y, z] = p;
            let old = world.get(x, y, z);
            if old != AIR && (old == WATER || !is_water(old)) {
                continue;
            }
            let new = incoming(world, p);
            if new != old {
                updates.push((p, new));
            }
        }
        // Evaluate the whole frontier first so queue order cannot turn one tick
        // into an instantaneous waterfall or favor a compass direction.
        for (p, id) in updates {
            world.set(p, id);
            dirty.push(p);
            self.wake(p);
        }
    }
}

fn incoming(world: &mut World, [x, y, z]: Pos) -> u8 {
    let neighbors = SIDES.map(|[dx, dz]| world.get(x + dx, y, z + dz));
    if neighbors.iter().filter(|&&id| id == WATER).count() >= 2 {
        let floor = world.get(x, y - 1, z);
        if floor == WATER || matches!(ITEMS[floor as usize].form, "cube" | "cutout" | "slab") {
            return WATER;
        }
    }
    if is_water(world.get(x, y + 1, z)) {
        return FALLING_WATER;
    }
    let mut nearest = WATER_LEVELS.len();
    for ([dx, dz], neighbor) in SIDES.into_iter().zip(neighbors) {
        let (nx, nz) = (x + dx, z + dz);
        let distance = match neighbor {
            WATER | FALLING_WATER => 0,
            level @ WATER_1..=WATER_7 => usize::from(level - WATER_1) + 1,
            _ => continue,
        };
        if distance >= nearest {
            continue;
        }
        let below = world.get(nx, y - 1, nz);
        if below == AIR || (is_water(below) && below != WATER) {
            continue;
        }
        nearest = distance;
    }
    WATER_LEVELS.get(nearest).copied().unwrap_or(AIR)
}

#[cfg(test)]
#[path = "water_tests.rs"]
mod tests;
