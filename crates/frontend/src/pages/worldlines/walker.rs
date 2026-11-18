use wasm_bindgen::prelude::*;

const RADIUS: f32 = 0.18;

/// Walkable quads carry the heights of their two Z edges. The exported room
/// supplies the same coordinates for both the stair treads and the ramps.
#[wasm_bindgen]
pub struct Walker {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    floors: Vec<f32>,
    solids: Vec<f32>,
}
impl Default for Walker {
    fn default() -> Self {
        Self {
            x: 0.05,
            y: 0.0,
            z: 2.55,
            floors: Vec::new(),
            solids: Vec::new(),
        }
    }
}
#[wasm_bindgen]
impl Walker {
    pub fn reset(&mut self) {
        self.x = 0.05;
        self.y = 0.0;
        self.z = 2.55;
    }
    pub fn load(&mut self, floors: Vec<f32>, solids: Vec<f32>) {
        self.floors = floors;
        self.solids = solids;
    }
    pub fn step(&mut self, forward: f32, right: f32, yaw: f32, distance: f32) {
        let magnitude = forward.hypot(right).max(1.0);
        let (sin, cos) = yaw.sin_cos();
        let distance = distance.clamp(0.0, 0.32);
        let steps = (distance / 0.055).ceil().max(1.0);
        let stride = distance / (steps * magnitude);
        let dx = (right * cos - forward * sin) * stride;
        let dz = (-forward * cos - right * sin) * stride;
        for _ in 0..steps as usize {
            self.move_to(self.x + dx, self.z);
            self.move_to(self.x, self.z + dz);
        }
    }
}
impl Walker {
    fn move_to(&mut self, x: f32, z: f32) {
        let height = self
            .floors
            .as_chunks::<6>()
            .0
            .iter()
            .filter(|f| x >= f[0] && x <= f[1] && z >= f[2] && z <= f[3])
            .map(|f| f[4] + (f[5] - f[4]) * (z - f[2]) / (f[3] - f[2]).max(0.001))
            .filter(|h| (h - self.y).abs() < 0.16)
            .min_by(|a, b| (a - self.y).abs().total_cmp(&(b - self.y).abs()));
        let Some(h) = height else { return };
        if self.solids.as_chunks::<6>().0.iter().any(|b| {
            x > b[0] - RADIUS
                && x < b[1] + RADIUS
                && z > b[2] - RADIUS
                && z < b[3] + RADIUS
                && h + 1.58 > b[4]
                && h + 0.10 < b[5]
        }) {
            return;
        }
        self.x = x;
        self.y = h;
        self.z = z;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn connects_slopes_and_blocks_tall_furniture() {
        let mut w = Walker::default();
        w.load(
            vec![-1., 1., 1., 3., 0., 0., -1., 1., -1., 1., 1., 0., -1., 1., -3., -1., 1., 1.],
            vec![-1., 1., -2.2, -1.9, 1., 2.0],
        );
        for _ in 0..40 {
            w.step(1., 0., 0., 0.2);
        }
        assert!((w.y - 1.).abs() < 0.02 && w.z > -1.73 && w.z < -1.63);
        w.reset();
        w.step(1., 1., 0., 0.2);
        assert!(((w.x - 0.05).hypot(w.z - 2.55) - 0.2).abs() < 0.001);
    }
}
