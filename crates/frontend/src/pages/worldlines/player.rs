use serde::{Deserialize, Serialize};

use super::{
    catalog::*,
    terrain::{World, HEIGHT},
};

const TICK: f32 = 1. / 120.;
pub const EYE: f32 = 1.58;
const RADIUS: f32 = 0.29;
const BODY: f32 = 1.77;
pub const REACH: f32 = 5.3;
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub pitch: f32,
    #[serde(skip)]
    vy: f32,
    #[serde(skip)]
    pub grounded: bool,
    #[serde(skip)]
    remainder: f32,
    #[serde(skip)]
    jump_queued: bool,
    #[serde(skip)]
    previous_eye: Option<[f32; 3]>,
    #[serde(skip)]
    step_offset: f32,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            x: -0.95,
            y: 17.,
            z: 4.38,
            yaw: -0.235,
            pitch: -0.105,
            vy: 0.,
            grounded: false,
            remainder: 0.,
            jump_queued: false,
            previous_eye: None,
            step_offset: 0.,
        }
    }
}
impl Player {
    pub fn valid(&self) -> bool {
        [self.x, self.y, self.z, self.yaw, self.pitch]
            .iter()
            .all(|n| n.is_finite())
            && self.x.abs() < 1_000_000.
            && self.z.abs() < 1_000_000.
            && (-2.0..=(HEIGHT as f32 + 2.)).contains(&self.y)
            && self.pitch.abs() <= 1.56
    }
    pub fn look(&mut self, yaw: f32, pitch: f32) {
        if yaw.is_finite() && pitch.is_finite() {
            self.yaw = (self.yaw + yaw) % std::f32::consts::TAU;
            self.pitch = (self.pitch + pitch).clamp(-1.54, 1.54);
        }
    }
    pub fn direction(&self) -> [f32; 3] {
        let (s, c) = self.yaw.sin_cos();
        let (v, h) = self.pitch.sin_cos();
        [-s * h, v, -c * h]
    }
    pub fn pose(&self) -> Vec<f32> {
        let [x, y, z] = self.view_position();
        vec![x, y, z, self.yaw, self.pitch, f32::from(u8::from(self.grounded))]
    }
    pub fn view_position(&self) -> [f32; 3] {
        let current = [self.x, self.y + self.step_offset, self.z];
        let previous = self.previous_eye.unwrap_or(current);
        let alpha = self.remainder / TICK;
        std::array::from_fn(|i| previous[i] + (current[i] - previous[i]) * alpha)
    }
    pub fn eye(&self) -> [f32; 3] {
        let [x, y, z] = self.view_position();
        [x, y + EYE, z]
    }
    fn collides(w: &mut World, x: f32, y: f32, z: f32) -> bool {
        for by in y.floor() as i32..=(y + BODY - 0.001).floor() as i32 {
            for bz in (z - RADIUS).floor() as i32..=(z + RADIUS - 0.001).floor() as i32 {
                for bx in (x - RADIUS).floor() as i32..=(x + RADIUS - 0.001).floor() as i32 {
                    let h = ITEMS[w.get(bx, by, bz) as usize].height;
                    if h > 0. && y < by as f32 + h - 0.0001 {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub fn overlaps(&self, p: [i32; 3], height: f32) -> bool {
        let [x, y, z] = p.map(|v| v as f32);
        height > 0.
            && self.x + RADIUS > x
            && self.x - RADIUS < x + 1.
            && self.z + RADIUS > z
            && self.z - RADIUS < z + 1.
            && self.y + BODY > y
            && self.y < y + height
    }
    fn slide(&mut self, w: &mut World, x: f32, z: f32) {
        if !Self::collides(w, x, self.y, z) {
            self.x = x;
            self.z = z;
        } else if self.grounded
            && !Self::collides(w, self.x, self.y + 0.5, self.z)
            && !Self::collides(w, x, self.y + 0.5, z)
        {
            self.x = x;
            self.y += 0.5;
            self.z = z;
        }
    }
    pub fn advance(
        &mut self,
        w: &mut World,
        dt: f32,
        side: f32,
        forward: f32,
        jump: bool,
        sprint: bool,
    ) {
        self.remainder += dt.clamp(0., 0.2);
        self.jump_queued |= jump;
        while self.remainder >= TICK {
            self.remainder -= TICK;
            self.previous_eye = Some([self.x, self.y + self.step_offset, self.z]);
            let jump = std::mem::take(&mut self.jump_queued);
            self.tick(w, side, forward, jump, sprint);
        }
    }
    fn tick(&mut self, w: &mut World, side: f32, forward: f32, jump: bool, sprint: bool) {
        let (s, c) = self.yaw.sin_cos();
        let length = (side * side + forward * forward).sqrt().max(1.);
        let swimming =
            w.get(self.x.floor() as i32, (self.y + 0.35).floor() as i32, self.z.floor() as i32)
                == WATER;
        let speed = if swimming {
            2.9
        } else if sprint {
            6.4
        } else {
            4.5
        };
        let dx = (side * c - forward * s) * speed / length;
        let dz = (-forward * c - side * s) * speed / length;
        let t = TICK;
        if jump && (self.grounded || swimming) {
            self.vy = if swimming { 4.4 } else { 8.0 };
            self.grounded = false;
        }
        let was_grounded = self.grounded;
        let floor_y = self.y;
        if dx != 0. {
            self.slide(w, self.x + dx * t, self.z);
        }
        if dz != 0. {
            self.slide(w, self.x, self.z + dz * t);
        }
        // Follow half-height treads in both directions. Full ledges still start a real
        // fall.
        if was_grounded && !swimming {
            if let Some(top) = self.support(w) {
                self.y = top;
                self.step_offset = (self.step_offset - (top - floor_y)).clamp(-0.6, 0.6);
                self.step_offset *= 0.88;
                self.vy = 0.;
                return;
            }
        }
        self.step_offset *= 0.88;
        self.vy = (self.vy - if swimming { 8. } else { 22. } * t).max(-28.);
        let ny = self.y + self.vy * t;
        if Self::collides(w, self.x, ny, self.z) {
            // Resolve to the last free point, also for partial blocks and ceilings.
            let (mut ok, mut blocked) = (self.y, ny);
            for _ in 0..10 {
                let mid = (ok + blocked) * 0.5;
                if Self::collides(w, self.x, mid, self.z) {
                    blocked = mid
                } else {
                    ok = mid
                }
            }
            self.y = ok;
            self.grounded = self.vy <= 0.;
            self.vy = 0.;
        } else {
            self.y = ny;
            self.grounded = false;
        }
    }
    fn support(&self, w: &mut World) -> Option<f32> {
        let mut top: Option<f32> = None;
        for by in (self.y - 0.501).floor() as i32..=(self.y + 0.001).floor() as i32 {
            for bz in (self.z - RADIUS).floor() as i32..=(self.z + RADIUS - 0.001).floor() as i32 {
                for bx in
                    (self.x - RADIUS).floor() as i32..=(self.x + RADIUS - 0.001).floor() as i32
                {
                    let h = ITEMS[w.get(bx, by, bz) as usize].height;
                    let y = by as f32 + h;
                    if h > 0. && (self.y - 0.501..=self.y + 0.001).contains(&y) {
                        top = Some(top.map_or(y, |prev| prev.max(y)));
                    }
                }
            }
        }
        top
    }
    pub fn pick(&self, w: &mut World) -> Option<Hit> {
        let origin = self.eye();
        let dir = self.direction();
        // Exact grid traversal; partial-height objects leave the air above them
        // reachable.
        let mut pos = origin.map(|v| v.floor() as i32);
        let step = dir.map(|d| if d >= 0. { 1 } else { -1 });
        let delta = dir.map(|d| if d.abs() < 1.0e-6 { f32::INFINITY } else { 1. / d.abs() });
        let mut next = [0.; 3];
        for i in 0..3 {
            next[i] = if dir[i].abs() < 1.0e-6 {
                f32::INFINITY
            } else {
                ((pos[i] + i32::from(step[i] > 0)) as f32 - origin[i]) / dir[i]
            };
        }
        let mut distance = 0.;
        while distance <= REACH {
            let id = w.get(pos[0], pos[1], pos[2]);
            if id != AIR && id != WATER {
                let [lo, hi] = bounds(id).map(|p| std::array::from_fn(|i| pos[i] as f32 + p[i]));
                if let Some((distance, normal)) = ray_box(origin, dir, lo, hi, REACH) {
                    return Some(Hit {
                        pos,
                        id,
                        distance,
                        normal,
                    });
                }
            }
            let axis = if next[0] < next[1] && next[0] < next[2] {
                0
            } else if next[1] < next[2] {
                1
            } else {
                2
            };
            distance = next[axis];
            next[axis] += delta[axis];
            pos[axis] += step[axis];
        }
        None
    }
    pub fn return_home(&mut self, w: &mut World) {
        let mut home = Self::default();
        // A player's buildings can occupy the original spawn. Find nearby headroom and
        // footing.
        for (dx, dz) in [(0, 0), (1, 0), (-1, 0), (0, -1), (0, 1), (2, 0), (-2, 0)] {
            let x = home.x + dx as f32;
            let z = home.z + dz as f32;
            for y in (1..HEIGHT).filter(|y| *y >= 17).chain((1..17).rev()) {
                if !Self::collides(w, x, y as f32, z) && Self::collides(w, x, y as f32 - 0.05, z) {
                    home.x = x;
                    home.y = y as f32;
                    home.z = z;
                    *self = home;
                    return;
                }
            }
        }
        home.y = HEIGHT as f32;
        *self = home;
    }
}
pub fn ray_box(
    origin: [f32; 3],
    dir: [f32; 3],
    lo: [f32; 3],
    hi: [f32; 3],
    limit: f32,
) -> Option<(f32, [i32; 3])> {
    let mut near = 0.0f32;
    let mut far = limit;
    let mut normal = [0; 3];
    for i in 0..3 {
        if dir[i].abs() < 1.0e-6 {
            if origin[i] < lo[i] || origin[i] > hi[i] {
                return None;
            }
        } else {
            let a = (lo[i] - origin[i]) / dir[i];
            let b = (hi[i] - origin[i]) / dir[i];
            if a.min(b) > near {
                near = a.min(b);
                normal = [0; 3];
                normal[i] = if dir[i] > 0. { -1 } else { 1 };
            }
            far = far.min(a.max(b));
            if near > far {
                return None;
            }
        }
    }
    Some((near, normal))
}
pub struct Hit {
    pub pos: [i32; 3],
    pub normal: [i32; 3],
    pub id: u8,
    pub distance: f32,
}
