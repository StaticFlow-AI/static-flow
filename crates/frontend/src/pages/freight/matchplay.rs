use std::f32::consts::PI;

use wasm_bindgen::prelude::*;

use super::map::{ray_box, Deck};

const COUNT: usize = 8;
const STEP: f32 = 1. / 120.;
const ROUND: f32 = 240.;
const LIMIT: u16 = 20;
const SPAWNS: [[f32; 2]; 4] = [[2.65, 26.0], [-10.0, 24.4], [5.4, 23.8], [8.7, 22.7]];
const CAP: [i16; 2] = [30, 12];
const LOAD: [f32; 2] = [1.65, 1.1];

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum Phase {
    Ready,
    Drop,
    Live,
    Respawn,
    Over,
    Interval,
}
#[derive(Clone, Copy, Default)]
struct Fighter {
    x: f32,
    y: f32,
    z: f32,
    prev: [f32; 3],
    yaw: f32,
    hp: i16,
    spawn: f32,
    shield: f32,
    cooldown: f32,
    kills: u16,
    deaths: u16,
    moving: f32,
    flash: f32,
    sense: f32,
    path: f32,
    way: [f32; 2],
    target: Option<usize>,
}
impl Fighter {
    fn born(id: usize, point: usize) -> Self {
        let [x, z] = SPAWNS[point % 4];
        let sign = if id < 4 { 1. } else { -1. };
        Self {
            x: x * sign,
            z: z * sign,
            prev: [x * sign, 0., z * sign],
            yaw: if id < 4 { 0. } else { PI },
            hp: 100,
            shield: 1.65,
            way: [x * sign, z * sign],
            sense: id as f32 * 0.023,
            ..Self::default()
        }
    }
    fn pos(self, t: f32) -> [f32; 3] {
        let now = [self.x, self.y, self.z];
        std::array::from_fn(|i| self.prev[i] + (now[i] - self.prev[i]) * t)
    }
}
#[wasm_bindgen]
pub struct FreightMatch {
    deck: Deck,
    people: [Fighter; COUNT],
    seed: u32,
    phase: Phase,
    stage: f32,
    time: f32,
    elimination: bool,
    round: u16,
    last_winner: i8,
    scores: [u16; 2],
    remain: f32,
    yaw: f32,
    pitch: f32,
    vy: f32,
    grounded: bool,
    crouch: f32,
    weapon: usize,
    ammo: [i16; 2],
    reserve: [i16; 2],
    loading: f32,
    jump: bool,
    tap: bool,
    events: Vec<f32>,
    damage: f32,
    confirm: f32,
    shots: u16,
}
#[wasm_bindgen]
impl FreightMatch {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u32) -> Self {
        Self {
            deck: Deck::new(),
            people: std::array::from_fn(|i| Fighter::born(i, i)),
            seed: seed.max(1),
            phase: Phase::Ready,
            stage: 0.,
            time: ROUND,
            elimination: false,
            round: 1,
            last_winner: -1,
            scores: [0; 2],
            remain: 0.,
            yaw: 0.,
            pitch: 0.,
            vy: 0.,
            grounded: true,
            crouch: 0.,
            weapon: 0,
            ammo: CAP,
            reserve: [120, 60],
            loading: 0.,
            jump: false,
            tap: false,
            events: Vec::with_capacity(96),
            damage: 0.,
            confirm: 0.,
            shots: 0,
        }
    }
    pub fn choose_elimination(&mut self, enabled: bool) {
        if matches!(self.phase, Phase::Ready | Phase::Over) {
            self.elimination = enabled;
        }
    }
    pub fn start(&mut self) {
        if !matches!(self.phase, Phase::Ready | Phase::Over) {
            return;
        }
        self.people = std::array::from_fn(|i| Fighter::born(i, i));
        self.phase = Phase::Drop;
        self.stage = 0.;
        self.time = if self.elimination { 60. } else { ROUND };
        self.round = 1;
        self.last_winner = -1;
        self.scores = [0; 2];
        self.yaw = 0.;
        self.pitch = 0.;
        self.remain = 0.;
        self.events.clear();
        self.shots = 0;
        self.reset_loadout();
    }
    pub fn reload(&mut self) {
        if self.phase == Phase::Live
            && self.loading == 0.
            && self.ammo[self.weapon] < CAP[self.weapon]
            && self.reserve[self.weapon] > 0
        {
            self.loading = LOAD[self.weapon];
        }
    }
    pub fn select(&mut self, slot: usize) {
        if self.phase == Phase::Live && slot < 2 && self.weapon != slot {
            self.weapon = slot;
            self.loading = 0.;
            self.people[0].cooldown = 0.18;
            self.tap = false;
        }
    }
    pub fn advance(
        &mut self,
        dt: f32,
        forward: f32,
        side: f32,
        yaw: f32,
        pitch: f32,
        buttons: u32,
    ) {
        self.yaw = yaw;
        self.pitch = pitch.clamp(-1.45, 1.45);
        if matches!(self.phase, Phase::Ready | Phase::Over) {
            return;
        }
        self.remain += dt.clamp(0., 0.15);
        self.jump |= buttons & 2 != 0;
        self.tap |= buttons & 1 != 0;
        while self.remain >= STEP {
            self.remain -= STEP;
            if self.phase == Phase::Over {
                self.remain = 0.;
                break;
            }
            self.tick(forward, side, buttons);
        }
    }
    /// Header followed by eight fixed-size actor records. One transfer per
    /// animation frame.
    pub fn frame(&self) -> Vec<f32> {
        let p = self.people[0];
        let t = (self.remain / STEP).clamp(0., 1.);
        let [x, y, z] = p.pos(t);
        let mut v = Vec::with_capacity(29 + COUNT * 11);
        v.extend([
            self.phase as u8 as f32,
            self.stage,
            self.time,
            self.scores[0] as f32,
            self.scores[1] as f32,
            x,
            y + 1.62 - self.crouch * 0.55,
            z,
            self.yaw,
            self.pitch,
            p.hp as f32,
            self.weapon as f32,
            self.ammo[self.weapon] as f32,
            self.reserve[self.weapon] as f32,
            if self.loading > 0. { 1. - self.loading / LOAD[self.weapon] } else { -1. },
            self.damage,
            self.confirm,
            p.moving,
            self.crouch,
            p.spawn,
            self.shots as f32,
            if self.elimination { 3. } else { LIMIT as f32 },
            p.shield,
            p.cooldown,
            p.kills as f32,
            p.deaths as f32,
        ]);
        for a in self.people {
            let [x, y, z] = a.pos(t);
            v.extend([
                x,
                y,
                z,
                a.yaw,
                a.hp as f32,
                a.moving,
                a.flash,
                a.shield,
                a.kills as f32,
                a.deaths as f32,
                a.spawn,
            ]);
        }
        v.extend([u8::from(self.elimination) as f32, self.round as f32, self.last_winner as f32]);
        v
    }
    pub fn events(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.events)
    }
    pub fn has_events(&self) -> bool {
        !self.events.is_empty()
    }
}

impl FreightMatch {
    fn random(&mut self) -> f32 {
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        (x >> 8) as f32 / 16_777_216.
    }
    fn reset_loadout(&mut self) {
        self.ammo = CAP;
        self.reserve = [120, 60];
        self.weapon = 0;
        self.loading = 0.;
        self.vy = 0.;
        self.grounded = true;
        self.crouch = 0.;
        self.damage = 0.;
        self.confirm = 0.;
        self.jump = false;
        self.tap = false;
    }
    fn finish_round(&mut self, winner: Option<usize>) {
        self.last_winner = winner.map_or(-1, |team| team as i8);
        if let Some(team) = winner {
            self.scores[team] += 1;
        }
        self.stage = 0.;
        self.phase = if self.scores.contains(&3) { Phase::Over } else { Phase::Interval };
    }
    fn next_round(&mut self) {
        self.round += 1;
        self.time = 60.;
        self.stage = 0.;
        self.phase = Phase::Live;
        for (id, a) in self.people.iter_mut().enumerate() {
            let old = *a;
            *a = Fighter::born(id, id);
            if self.round.is_multiple_of(2) {
                a.x = -a.x;
                a.z = -a.z;
                a.yaw += PI;
                a.prev = [a.x, 0., a.z];
                a.way = [a.x, a.z];
            }
            a.kills = old.kills;
            a.deaths = old.deaths;
        }
        self.yaw = self.people[0].yaw;
        self.pitch = 0.;
        self.reset_loadout();
    }
    fn tick(&mut self, forward: f32, side: f32, buttons: u32) {
        self.stage += STEP;
        if self.phase == Phase::Interval {
            if self.stage >= 2.8 {
                self.next_round();
            }
            return;
        }
        if self.phase == Phase::Drop {
            if self.stage >= 3.1 {
                self.phase = Phase::Live;
                self.stage = 0.;
                self.tap = false;
                self.jump = false;
            }
            return;
        }
        self.time = (self.time - STEP).max(0.);
        if self.time == 0. {
            if self.elimination {
                let tally = |team: usize| {
                    let crew = &self.people[team * 4..team * 4 + 4];
                    (
                        crew.iter().filter(|a| a.hp > 0).count(),
                        crew.iter().map(|a| i32::from(a.hp)).sum::<i32>(),
                    )
                };
                self.finish_round(match tally(0).cmp(&tally(1)) {
                    std::cmp::Ordering::Greater => Some(0),
                    std::cmp::Ordering::Less => Some(1),
                    std::cmp::Ordering::Equal => None,
                });
            } else {
                self.phase = Phase::Over;
            }
            return;
        }
        self.damage = (self.damage - STEP * 2.8).max(0.);
        self.confirm = (self.confirm - STEP * 3.).max(0.);
        for i in 0..COUNT {
            let a = &mut self.people[i];
            a.prev = [a.x, a.y, a.z];
            a.flash = (a.flash - STEP).max(0.);
            a.cooldown = (a.cooldown - STEP).max(0.);
            a.shield = (a.shield - STEP).max(0.);
            if a.hp <= 0 && !self.elimination {
                a.spawn -= STEP;
                if a.spawn <= 0. {
                    self.respawn(i)
                }
            }
        }
        if self.phase == Phase::Live {
            self.walk(forward, side, buttons);
            if self.loading > 0. {
                self.loading = (self.loading - STEP).max(0.);
                if self.loading == 0. {
                    let n =
                        (CAP[self.weapon] - self.ammo[self.weapon]).min(self.reserve[self.weapon]);
                    self.ammo[self.weapon] += n;
                    self.reserve[self.weapon] -= n;
                }
            } else if (self.tap || buttons & 1 != 0) && self.people[0].cooldown <= 0. {
                if self.ammo[self.weapon] > 0 {
                    self.ammo[self.weapon] -= 1;
                    self.shots = self.shots.wrapping_add(1);
                    self.people[0].cooldown = [0.105, 0.225][self.weapon];
                    let dir = direction(self.yaw, self.pitch);
                    self.fire(0, dir, if self.weapon == 0 { 34 } else { 29 });
                } else {
                    self.reload()
                }
            }
        }
        self.tap = false;
        self.jump = false;
        for i in 1..COUNT {
            if self.people[i].hp > 0 && matches!(self.phase, Phase::Live | Phase::Respawn) {
                self.bot(i)
            }
        }
    }
    fn walk(&mut self, forward: f32, side: f32, buttons: u32) {
        let p = &mut self.people[0];
        let low = buttons & 4 != 0;
        let want = if low { 1. } else { 0. };
        self.crouch += (want - self.crouch) * (STEP * 14.).min(1.);
        if self.jump && self.grounded {
            self.vy = 6.1;
            self.grounded = false
        }
        let speed = if low {
            2.1
        } else if buttons & 16 != 0 {
            2.7
        } else if buttons & 8 != 0 {
            6.7
        } else {
            4.65
        };
        let length = forward.hypot(side).max(1.);
        let (s, c) = self.yaw.sin_cos();
        let dx = (side * c - forward * s) * speed / length * STEP;
        let dz = (-forward * c - side * s) * speed / length * STEP;
        let height = 1.77 - self.crouch * 0.55;
        for (sx, sz) in [(dx, 0.), (0., dz)] {
            let (x, z) = (p.x + sx, p.z + sz);
            let foot = self
                .deck
                .floor(x, z, p.y + if self.grounded { 0.16 } else { 0.025 }, 0.29);
            if self.deck.clear(x, p.y.max(foot), z, 0.30, height) {
                p.x = x;
                p.z = z;
                if self.grounded && p.y - foot <= 0.16 {
                    p.y = foot
                }
            }
        }
        self.vy -= 16. * STEP;
        let support = self.deck.floor(p.x, p.z, p.y + 0.045, 0.29);
        p.y += self.vy * STEP;
        if p.y <= support {
            p.y = support;
            self.vy = 0.;
            self.grounded = true
        } else {
            self.grounded = false
        }
        p.yaw = self.yaw;
        p.moving = (dx.hypot(dz) / STEP).min(speed);
    }
    fn respawn(&mut self, id: usize) {
        let start = (self.random() * 4.) as usize;
        let old = self.people[id];
        let mut best = Fighter::born(id, start);
        let mut safest = -1.;
        for n in 0..4 {
            let p = Fighter::born(id, n + start);
            let score = self
                .people
                .iter()
                .enumerate()
                .filter(|(i, a)| *i != id && a.hp > 0)
                .map(|(i, a)| {
                    ((a.x - p.x).hypot(a.z - p.z)) * if (i < 4) == (id < 4) { 3.5 } else { 1. }
                })
                .fold(80., f32::min);
            if score > safest {
                safest = score;
                best = p
            }
        }
        best.kills = old.kills;
        best.deaths = old.deaths;
        self.people[id] = best;
        if id == 0 {
            self.phase = Phase::Live;
            self.stage = 0.;
            self.reset_loadout();
        }
    }
    fn bot(&mut self, id: usize) {
        let mut b = self.people[id];
        b.sense -= STEP;
        b.path -= STEP;
        if b.sense <= 0. {
            b.sense = 0.16 + self.random() * 0.08;
            let mut best = None;
            let mut range = 36.;
            for j in 0..COUNT {
                let other = self.people[j];
                if (j < 4) == (id < 4) || other.hp <= 0 || other.shield > 0. {
                    continue;
                }
                let dy = other.y - b.y - if j == 0 { self.crouch * 0.50 } else { 0. };
                let v = [other.x - b.x, dy - 0.1, other.z - b.z];
                let len = length(v);
                if len < range
                    && self
                        .deck
                        .cover([b.x, b.y + 1.4, b.z], scale(v, 1. / len.max(0.01)), len)
                        >= len - 0.05
                {
                    best = Some(j);
                    range = len;
                }
            }
            if best != b.target && best.is_some() {
                b.cooldown = b.cooldown.max(0.28 + self.random() * 0.48);
            }
            b.target = best;
        }
        let target = b.target.filter(|&j| self.people[j].hp > 0);
        let foreign = if id < 4 { 4 } else { 0 };
        let opponent = target.unwrap_or_else(|| {
            (0..4)
                .map(|n| foreign + (id + 1 + n) % 4)
                .find(|&j| self.people[j].hp > 0)
                .unwrap_or(foreign)
        });
        let other = self.people[opponent];
        if b.path <= 0. {
            b.path = 0.3 + self.random() * 0.18;
            let (gx, gz) = if target.is_some() {
                (other.x, other.z)
            } else {
                let flank = [-9.2, -1.4, 8.8][id % 3];
                if b.z.abs() > 12. && (b.z > 0.) == (id < 4) {
                    (flank, if id < 4 { -4. } else { 4. })
                } else {
                    (other.x, other.z)
                }
            };
            let (x, z) = self.deck.next(b.x, b.z, gx, gz);
            b.way = [x, z];
        }
        let walk = target.is_none() || (b.x - other.x).hypot(b.z - other.z) > 17.;
        let (dx, dz) = (b.way[0] - b.x, b.way[1] - b.z);
        let dist = dx.hypot(dz).max(0.06);
        b.moving = 0.;
        if walk && dist > 0.09 {
            let speed = if target.is_some() { 2.5 } else { 3.65 };
            let d = (speed * STEP).min(dist) / dist;
            let nx = b.x + dx * d;
            let nz = b.z + dz * d;
            if self.deck.clear(nx, 0., b.z, 0.32, 1.7) {
                b.x = nx;
            }
            if self.deck.clear(b.x, 0., nz, 0.32, 1.7) {
                b.z = nz;
            }
            b.moving = (b.x - b.prev[0]).hypot(b.z - b.prev[2]) / STEP;
        }
        let desired = if target.is_some() {
            (-(other.x - b.x)).atan2(-(other.z - b.z))
        } else if dist > 0.2 {
            (-dx).atan2(-dz)
        } else {
            b.yaw
        };
        b.yaw += wrap(desired - b.yaw).clamp(-STEP * 7., STEP * 7.);
        self.people[id] = b;
        if let Some(j) = target.filter(|_| b.cooldown <= 0. && wrap(desired - b.yaw).abs() < 0.25) {
            let origin = [b.x, b.y + 1.4, b.z];
            let swing = if j == 0 { 1.18 } else { 1.38 };
            let v = [
                other.x - origin[0] + (self.random() - 0.5) * swing,
                other.y + 1.0 - if j == 0 { self.crouch * 0.4 } else { 0. } - origin[1]
                    + (self.random() - 0.5) * 1.1,
                other.z - origin[2] + (self.random() - 0.5) * swing,
            ];
            self.people[id].cooldown = 0.24 + self.random() * 0.44;
            self.fire(id, scale(v, 1. / length(v).max(0.01)), 19);
        }
    }
    fn fire(&mut self, id: usize, dir: [f32; 3], power: i16) {
        let a = self.people[id];
        let o = [a.x, a.y + if id == 0 { 1.62 - self.crouch * 0.55 } else { 1.4 }, a.z];
        self.people[id].flash = 0.075;
        self.people[id].shield = 0.;
        let mut distance = self.deck.cover(o, dir, 82.);
        let mut victim = None;
        let mut points = power;
        for (j, b) in self.people.iter().enumerate() {
            if (id < 4) == (j < 4) || b.hp <= 0 || b.shield > 0. {
                continue;
            }
            let low = if j == 0 { self.crouch * 0.55 } else { 0. };
            if let Some(t) = ray_box(
                o,
                dir,
                [b.x - 0.33, b.y + 0.03, b.z - 0.33],
                [b.x + 0.33, b.y + 1.81 - low, b.z + 0.33],
                distance,
            ) {
                distance = t;
                victim = Some(j);
                points = if o[1] + dir[1] * t > b.y + 1.45 - low { power * 2 } else { power };
            }
        }
        let end = std::array::from_fn::<_, 3, _>(|i| o[i] + dir[i] * distance);
        let target = victim.map_or(-1., |j| j as f32);
        self.events
            .extend([1., id as f32, target, o[0], o[1], o[2], end[0], end[1], end[2]]);
        if let Some(j) = victim {
            if j == 0 {
                self.damage = 1.;
            }
            if id == 0 {
                self.confirm = 1.;
            }
            let b = &mut self.people[j];
            b.hp = (b.hp - points).max(0);
            if b.hp == 0 {
                b.deaths += 1;
                b.spawn = 3.0;
                b.moving = 0.;
                self.people[id].kills += 1;
                if !self.elimination {
                    self.scores[usize::from(id >= 4)] += 1;
                }
                self.events
                    .extend([2., id as f32, j as f32, 0., 0., 0., end[0], end[1], end[2]]);
                if j == 0 {
                    self.phase = Phase::Respawn;
                    self.stage = 0.;
                    self.loading = 0.;
                }
                if self.elimination {
                    let side = usize::from(j >= 4);
                    if self.people[side * 4..side * 4 + 4]
                        .iter()
                        .all(|a| a.hp == 0)
                    {
                        self.finish_round(Some(1 - side));
                    }
                } else if self.scores.iter().any(|&n| n >= LIMIT) {
                    self.phase = Phase::Over;
                    self.stage = 0.;
                }
            }
        }
    }
}
fn wrap(v: f32) -> f32 {
    (v + PI).rem_euclid(2. * PI) - PI
}
fn length(v: [f32; 3]) -> f32 {
    v[0].hypot(v[1]).hypot(v[2])
}
fn scale(v: [f32; 3], t: f32) -> [f32; 3] {
    v.map(|n| n * t)
}
fn direction(yaw: f32, pitch: f32) -> [f32; 3] {
    let (s, c) = yaw.sin_cos();
    let (p, q) = pitch.sin_cos();
    [-s * q, p, -c * q]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn blocked_shots_ammunition_and_restart_follow_the_round() {
        let mut g = FreightMatch::new(42);
        g.start();
        for _ in 0..380 {
            g.advance(STEP, 0., 0., 0., 0., 0)
        }
        assert!(g.phase == Phase::Live);
        for a in &mut g.people {
            a.shield = 0.;
        }
        let p = g.people[0];
        g.people[4].x = p.x;
        g.people[4].z = p.z - 3.3;
        g.fire(0, [0., 0., -1.], 100);
        assert_eq!(g.scores, [1, 0]);
        assert_eq!(g.people[4].hp, 0);
        g.people[0].x = 6.5;
        g.people[0].z = 22.;
        g.people[5].x = 6.5;
        g.people[5].z = 10.;
        g.fire(0, [0., 0., -1.], 100);
        assert_eq!(g.people[5].hp, 100);
        g.ammo[0] = 2;
        g.reload();
        for _ in 0..205 {
            g.advance(STEP, 0., 0., 0., 0., 0);
        }
        assert_eq!(g.ammo[0], 30);
        assert_eq!(g.reserve[0], 92);
        g.time = STEP;
        g.advance(STEP, 0., 0., 0., 0., 0);
        assert!(g.phase == Phase::Over);
        g.start();
        assert_eq!(g.scores, [0, 0]);
        assert_eq!(g.people[0].kills, 0);
    }
    #[test]
    fn climbing_the_ramp_and_stepping_off_the_platform_preserve_eye_continuity() {
        let mut g = FreightMatch::new(5);
        g.people[0].x = -11.8;
        g.people[0].z = 4.6;
        for _ in 0..265 {
            g.walk(1., 0., 0);
        }
        assert!((g.people[0].y - 2.65).abs() < 0.03);
        let mut previous = g.people[0].y;
        for _ in 0..165 {
            g.walk(0., 1., 0);
            let current = g.people[0].y;
            assert!(previous - current < 0.10);
            previous = current;
        }
        assert_eq!(g.people[0].y, 0.);
    }

    #[test]
    fn elimination_keeps_casualties_out_and_swaps_the_spawn_for_the_next_round() {
        let mut g = FreightMatch::new(12);
        g.choose_elimination(true);
        g.start();
        g.phase = Phase::Live;
        g.people[4].hp = 0;
        g.people[4].spawn = 0.;
        g.advance(0.05, 0., 0., 0., 0., 0);
        assert_eq!(g.people[4].hp, 0);
        let z = g.people[0].z;
        g.time = STEP;
        g.advance(STEP * 2., 0., 0., 0., 0., 0);
        assert!(g.phase == Phase::Interval);
        assert_eq!(g.scores, [1, 0]);
        for _ in 0..100 {
            g.advance(1. / 30., 0., 0., PI, 0., 0);
        }
        assert!(g.phase == Phase::Live);
        assert_eq!(g.round, 2);
        assert_eq!(g.people[0].z, -z);
        assert_eq!(g.people[4].hp, 100);
        g.scores = [2, 1];
        g.finish_round(Some(0));
        assert!(g.phase == Phase::Over);
    }
    #[test]
    fn bots_contest_the_deck_and_complete_both_matches_without_input() {
        for elimination in [false, true] {
            let mut game = FreightMatch::new(19);
            game.choose_elimination(elimination);
            game.start();
            for _ in 0..18000 {
                game.advance(1. / 30., 0., 0., 0., 0., 0);
                game.events.clear();
                if game.phase == Phase::Over {
                    break;
                }
            }
            assert!(game.phase == Phase::Over);
            if elimination {
                assert!(game.scores.contains(&3))
            } else {
                assert!(game.scores.iter().sum::<u16>() >= 6);
            }
        }
    }
}
