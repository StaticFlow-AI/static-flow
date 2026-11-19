use serde::{Deserialize, Serialize};

use super::{
    catalog::{is_water, AIR, GRASS, ITEMS, LEAVES, PINE, SAND, SNOW, WATER},
    player::Player,
    terrain::{surface, tree_at, World, HEIGHT, SIDE},
};

// The camera has its own position so that surveying a mountain never moves the
// survivor.
#[derive(Clone, Copy, Serialize, Deserialize)]
pub(super) struct SurveyCamera {
    x: f32,
    y: f32,
    z: f32,
    yaw: f32,
    pitch: f32,
}
impl SurveyCamera {
    pub fn above(player: &Player) -> Self {
        let (s, c) = player.yaw.sin_cos();
        Self {
            x: (player.x + s * 28.).clamp(-999_900., 999_900.),
            y: (player.y + 33.).clamp(43., 96.),
            z: (player.z + c * 28.).clamp(-999_900., 999_900.),
            yaw: player.yaw,
            pitch: -0.88,
        }
    }
    pub fn valid(&self) -> bool {
        self.x.abs() <= 999_900.
            && self.z.abs() <= 999_900.
            && (1.0..=96.).contains(&self.y)
            && self.yaw.is_finite()
            && self.pitch.abs() <= 1.56
    }
    pub fn pose(&self) -> Vec<f32> {
        vec![self.x, self.y, self.z, self.yaw, self.pitch, 0.]
    }
    pub fn look(&mut self, yaw: f32, pitch: f32) {
        if yaw.is_finite() && pitch.is_finite() {
            self.yaw = (self.yaw + yaw) % std::f32::consts::TAU;
            self.pitch = (self.pitch + pitch).clamp(-1.55, 1.55);
        }
    }
    pub fn fly(&mut self, dt: f32, side: f32, forward: f32, lift: f32, fast: bool) {
        let distance = dt.clamp(0., 0.2) * if fast { 31. } else { 13. };
        let (s, c) = self.yaw.sin_cos();
        let n = side.hypot(forward).hypot(lift).max(1.);
        self.x = (self.x + (side * c - forward * s) * distance / n).clamp(-999_900., 999_900.);
        self.z = (self.z - (forward * c + side * s) * distance / n).clamp(-999_900., 999_900.);
        self.y = (self.y + lift * distance / n).clamp(1., 96.);
    }
    pub fn landmark(&mut self, id: u8) {
        let (x, z) = match id {
            0 => (0., 0.),
            1 => (-15., 10.),
            2 => (-11., 14.),
            3 => (25., -3.),
            _ => return,
        };
        self.x = x + 9.;
        self.y = if id == 0 { 46. } else { 31. };
        self.z = z + if id == 0 { 23. } else { 12. };
        self.yaw = if id == 0 { 0.38 } else { 0.64 };
        self.pitch = -0.92;
    }
}

// The overview is sampled in the same terrain worker as detailed chunks. Town
// buildings and any modified chunks use their actual blocks. Untouched
// wilderness needs only the generator's surface and tree positions, so crossing
// the horizon doesn't instantiate hundreds of underground chunks.
pub(super) fn landscape(world: &mut World, cx: i32, cz: i32) -> Vec<i16> {
    const WIDTH: usize = 97;
    const STRIDE: i32 = 4;
    let ox = cx - WIDTH as i32 / 2 * STRIDE;
    let oz = cz - WIDTH as i32 / 2 * STRIDE;
    let mut data = vec![0i16; 2 + WIDTH * WIDTH * 2];
    data[..2].copy_from_slice(&[WIDTH as i16, STRIDE as i16]);
    for iz in 0..WIDTH {
        for ix in 0..WIDTH {
            let x = ox + ix as i32 * STRIDE;
            let z = oz + iz as i32 * STRIDE;
            let i = iz * WIDTH + ix;
            let key = (x.div_euclid(SIDE), z.div_euclid(SIDE));
            let authored = (-2..=2).contains(&key.0) && (-2..=1).contains(&key.1);
            let (height, id) = if authored || world.has_edits(key) {
                (0..HEIGHT)
                    .rev()
                    .find_map(|y| {
                        let id = world.get(x, y, z);
                        (id != AIR && (is_water(id) || ITEMS[id as usize].height > 0.))
                            .then_some((y + 1, id))
                    })
                    .unwrap_or((0, SAND))
            } else {
                let h = surface(world.seed, x, z);
                if h < 8 {
                    (9, WATER)
                } else {
                    (
                        h + 1,
                        if h == 8 {
                            SAND
                        } else if h > 26 {
                            SNOW
                        } else {
                            GRASS
                        },
                    )
                }
            };
            data[2 + i * 2..4 + i * 2].copy_from_slice(&[height as i16, i16::from(id)]);
        }
    }
    let end = WIDTH as i32 * STRIDE;
    for gz in (oz - 3).div_euclid(7)..=(oz + end).div_euclid(7) {
        for gx in (ox - 3).div_euclid(7)..=(ox + end).div_euclid(7) {
            let Some((x, y, z, high)) = tree_at(world.seed, gx, gz) else { continue };
            if x < ox - 2 || z < oz - 2 || x >= ox + end || z >= oz + end {
                continue;
            }
            if !world.has_edits((x.div_euclid(SIDE), z.div_euclid(SIDE))) {
                data.extend([
                    (x - cx) as i16,
                    y as i16,
                    (z - cz) as i16,
                    i16::from(if high { PINE } else { LEAVES }),
                ]);
            }
        }
    }
    data
}
