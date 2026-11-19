use serde::Deserialize;

pub const HALF_X: f32 = 14.65;
pub const HALF_Z: f32 = 28.6;
pub const DECK: &str = include_str!("deck.json");
const COLS: usize = 15;
const ROWS: usize = 29;
const CELLS: usize = COLS * ROWS;

#[derive(Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Container,
    Crate,
    Ramp,
    Platform,
}
#[derive(Deserialize)]
pub struct Cargo {
    pub kind: Kind,
    pub x: f32,
    pub z: f32,
    pub w: f32,
    pub d: f32,
    pub h: f32,
    #[serde(default)]
    pub y: f32,
    #[serde(default)]
    pub paint: u8,
}
impl Cargo {
    fn inside(&self, x: f32, z: f32, pad: f32) -> bool {
        (x - self.x).abs() < self.w * 0.5 + pad && (z - self.z).abs() < self.d * 0.5 + pad
    }
    fn top(&self, z: f32) -> f32 {
        if self.kind == Kind::Ramp {
            let t = ((z - self.z) / self.d + 0.5).clamp(0., 1.);
            self.y + self.h * if self.paint == 1 { t } else { 1. - t }
        } else {
            self.y + self.h
        }
    }
}

pub struct Deck {
    pub cargo: Vec<Cargo>,
    links: [[i16; 4]; CELLS],
    open: [bool; CELLS],
}
impl Deck {
    pub fn new() -> Self {
        let mut map = Self {
            cargo: serde_json::from_str(DECK).expect("valid freight deck"),
            links: [[-1; 4]; CELLS],
            open: [false; CELLS],
        };
        for i in 0..CELLS {
            let (x, z) = position(i);
            map.open[i] = map.clear(x, 0., z, 0.50, 1.7);
        }
        for i in 0..CELLS {
            if !map.open[i] {
                continue;
            }
            let (x, z) = position(i);
            for (k, (dx, dz)) in [(1, 0), (-1, 0), (0, 1), (0, -1)].into_iter().enumerate() {
                let nx = (i % COLS) as i32 + dx;
                let nz = (i / COLS) as i32 + dz;
                if (0..COLS as i32).contains(&nx) && (0..ROWS as i32).contains(&nz) {
                    let j = nz as usize * COLS + nx as usize;
                    if map.open[j] && map.clear(x + dx as f32, 0., z + dz as f32, 0.50, 1.7) {
                        map.links[i][k] = j as i16;
                    }
                }
            }
        }
        map
    }
    pub fn clear(&self, x: f32, y: f32, z: f32, r: f32, height: f32) -> bool {
        x.abs() + r < HALF_X
            && z.abs() + r < HALF_Z
            && !self
                .cargo
                .iter()
                .any(|b| b.inside(x, z, r) && y + height > b.y + 0.02 && b.top(z) > y + 0.11)
    }
    pub fn floor(&self, x: f32, z: f32, ceiling: f32, r: f32) -> f32 {
        self.cargo
            .iter()
            .filter(|b| b.inside(x, z, r))
            .map(|b| b.top(z))
            .filter(|h| *h <= ceiling)
            .fold(0., f32::max)
    }
    pub fn cover(&self, origin: [f32; 3], dir: [f32; 3], limit: f32) -> f32 {
        let mut hit = limit;
        if dir[1] < -0.0001 {
            hit = hit.min((-origin[1] / dir[1]).max(0.));
        }
        for b in &self.cargo {
            let t = ray_box(
                origin,
                dir,
                [b.x - b.w * 0.5, b.y, b.z - b.d * 0.5],
                [b.x + b.w * 0.5, b.y + b.h, b.z + b.d * 0.5],
                hit,
            );
            if let Some(mut enter) = t {
                if b.kind == Kind::Ramp {
                    let slope = if b.paint == 1 { b.h / b.d } else { -b.h / b.d };
                    let side = origin[1] + dir[1] * enter - b.top(origin[2] + dir[2] * enter);
                    if side > 0.001 {
                        let rate = slope * dir[2] - dir[1];
                        if rate <= 0. {
                            continue;
                        }
                        enter += side / rate;
                        let x = origin[0] + dir[0] * enter;
                        let z = origin[2] + dir[2] * enter;
                        if !b.inside(x, z, 0.001) {
                            continue;
                        }
                    }
                }
                hit = hit.min(enter)
            }
        }
        hit
    }
    fn nearest(&self, x: f32, z: f32) -> usize {
        let col = ((x + 14.) * 0.5).round().clamp(0., 14.) as usize;
        let row = ((z + 28.) * 0.5).round().clamp(0., 28.) as usize;
        let i = row * COLS + col;
        if self.open[i] {
            return i;
        }
        (0..CELLS)
            .filter(|&j| self.open[j])
            .min_by(|&a, &b| {
                let dist = |j| {
                    let (px, pz) = position(j);
                    (px - x).powi(2) + (pz - z).powi(2)
                };
                dist(a).total_cmp(&dist(b))
            })
            .unwrap_or(i)
    }
    pub fn next(&self, x: f32, z: f32, gx: f32, gz: f32) -> (f32, f32) {
        let start = self.nearest(x, z);
        let goal = self.nearest(gx, gz);
        if start == goal {
            return position(goal);
        }
        let mut first = [-1i16; CELLS];
        let mut queue = [0usize; CELLS];
        let mut read = 0;
        let mut write = 1;
        queue[0] = start;
        first[start] = start as i16;
        while read < write {
            let i = queue[read];
            read += 1;
            for &j in &self.links[i] {
                if j < 0 || first[j as usize] >= 0 {
                    continue;
                }
                let j = j as usize;
                first[j] = if i == start { j as i16 } else { first[i] };
                if j == goal {
                    return position(first[j] as usize);
                }
                queue[write] = j;
                write += 1;
            }
        }
        position(start)
    }
}
fn position(i: usize) -> (f32, f32) {
    ((i % COLS) as f32 * 2. - 14., (i / COLS) as f32 * 2. - 28.)
}
pub fn ray_box(o: [f32; 3], d: [f32; 3], lo: [f32; 3], hi: [f32; 3], limit: f32) -> Option<f32> {
    let (mut near, mut far) = (0f32, limit);
    for a in 0..3 {
        if d[a].abs() < 0.00001 {
            if o[a] < lo[a] || o[a] > hi[a] {
                return None;
            }
        } else {
            let t = (lo[a] - o[a]) / d[a];
            let u = (hi[a] - o[a]) / d[a];
            near = near.max(t.min(u));
            far = far.min(t.max(u));
            if far < near {
                return None;
            }
        }
    }
    Some(near)
}
