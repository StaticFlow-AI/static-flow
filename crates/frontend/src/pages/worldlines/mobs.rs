use serde::{Deserialize, Serialize};

use super::{
    catalog::*,
    player::{ray_box, Player, BODY, RADIUS},
    terrain::{hash, World, HEIGHT},
};

const STEP: f32 = 0.1;
const ZOMBIE_LIMIT: usize = 5;
const ZOMBIE_HEALTH: u8 = 4;
const FULL_STOCK: u8 = 4;
const RESTOCK_SECONDS: f32 = 60.;
type Position = [f32; 3];

struct Cottage {
    name: &'static str,
    price: (u8, u32),
    goods: (u8, u32),
    doorstep: [f32; 2],
    route: [[f32; 2]; 4],
}

// Each doorstep faces north. The paths meet the doors of the three authored
// cottages.
const COTTAGES: [Cottage; 3] = [
    Cottage {
        name: "果农",
        price: (LOG, 1),
        goods: (APPLE, 3),
        doorstep: [-17.5, 12.5],
        route: [[-12.5, 11.5], [-12.5, 9.5], [-17.5, 9.5], [-17.5, 12.5]],
    },
    Cottage {
        name: "矿工",
        price: (COBBLE, 4),
        goods: (TORCH, 6),
        doorstep: [22.5, 13.5],
        route: [[17.5, 8.5], [18.5, 10.5], [22.5, 12.5], [22.5, 13.5]],
    },
    Cottage {
        name: "护林员",
        price: (SAPLING, 2),
        goods: (FUEL, 3),
        doorstep: [-17.5, -4.5],
        route: [[-17.5, -5.5], [-13.5, -6.5], [-13.5, -9.5], [-17.5, -8.5]],
    },
];

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Body {
    pos: Position,
    yaw: f32,
    #[serde(skip)]
    speed: f32,
}

impl Body {
    fn new(pos: Position) -> Self {
        Self {
            pos,
            yaw: 0.,
            speed: 0.,
        }
    }

    fn valid(&self) -> bool {
        let [x, y, z] = self.pos;
        x.abs() < 1_000_000.
            && z.abs() < 1_000_000.
            && (1.0..HEIGHT as f32).contains(&y)
            && self.yaw.is_finite()
    }

    fn face(&mut self, x: f32, z: f32) {
        self.yaw = (x - self.pos[0]).atan2(z - self.pos[2]);
    }

    fn walk(
        &mut self,
        world: &mut World,
        goal: [f32; 2],
        speed: f32,
        passing_side: f32,
        neighbors: &[Option<Position>],
    ) {
        let [x, y, z] = self.pos;
        let [dx, dz] = [goal[0] - x, goal[1] - z];
        let distance = dx.hypot(dz);
        if distance < 0.03 {
            return;
        }
        let step = (speed * STEP).min(distance) / distance;
        let (dx, dz) = (dx * step, dz * step);
        // Try the desired heading first, then follow the face of an obstacle.
        let diagonal = std::f32::consts::FRAC_1_SQRT_2;
        for (forward, side) in [
            (1., 0.),
            (diagonal, diagonal * passing_side),
            (diagonal, -diagonal * passing_side),
            (0., passing_side),
            (0., -passing_side),
        ] {
            let next_x = x + dx * forward + dz * side;
            let next_z = z + dz * forward - dx * side;
            if neighbors.iter().flatten().any(|&[ox, oy, oz]| {
                let previous = (x - ox).powi(2) + (z - oz).powi(2);
                let next = (next_x - ox).powi(2) + (next_z - oz).powi(2);
                (oy - y).abs() < BODY && next < (2. * RADIUS).powi(2) && next < previous
            }) {
                continue;
            }
            let Some(top) = footing(world, next_x, next_z, y + 1., y - 1.) else { continue };
            if top > y && Player::collides(world, x, top, z)
                || top < y && Player::collides(world, next_x, y, next_z)
            {
                continue;
            }
            self.face(next_x, next_z);
            self.pos = [next_x, top, next_z];
            self.speed = (next_x - x).hypot(next_z - z) / STEP;
            return;
        }
    }
}

// A footprint can straddle a half slab or ledge. Pick its highest support in
// range, then require the same headroom as the player. Water is a barrier for
// walkers.
fn footing(world: &mut World, x: f32, z: f32, high: f32, low: f32) -> Option<f32> {
    let low = low.max(1.);
    let high = high.min(HEIGHT as f32 - BODY);
    for y in ((low - 1.).floor() as i32..=high.floor() as i32).rev() {
        let mut top: Option<f32> = None;
        for bz in (z - RADIUS).floor() as i32..=(z + RADIUS - 0.001).floor() as i32 {
            for bx in (x - RADIUS).floor() as i32..=(x + RADIUS - 0.001).floor() as i32 {
                let height = ITEMS[world.get(bx, y, bz) as usize].height;
                let surface = y as f32 + height;
                if height > 0. && (low..=high).contains(&surface) {
                    top = Some(top.map_or(surface, |last| last.max(surface)));
                }
            }
        }
        if let Some(top) = top {
            return (!world.in_water([x, top + 0.01, z]) && !Player::collides(world, x, top, z))
                .then_some(top);
        }
    }
    None
}

#[derive(Clone, Serialize, Deserialize)]
struct Villager {
    body: Option<Body>,
    waypoint: usize,
    wait: f32,
    stock: u8,
    restock: f32,
}

impl Default for Villager {
    fn default() -> Self {
        Self {
            body: None,
            waypoint: 0,
            wait: 1.5,
            stock: FULL_STOCK,
            restock: 0.,
        }
    }
}

impl Villager {
    fn resupply(&mut self) {
        if self.stock < FULL_STOCK {
            self.restock = (self.restock - STEP).max(0.);
            if self.restock == 0. {
                self.stock += 1;
                self.restock = if self.stock < FULL_STOCK { RESTOCK_SECONDS } else { 0. };
            }
        }
    }

    fn settle(&mut self, world: &mut World, cottage: &Cottage) {
        if let Some(body) = &mut self.body {
            let [x, y, z] = body.pos;
            if let Some(top) = footing(world, x, z, y + 1., y - 3.) {
                body.pos[1] = top;
                return;
            }
        }
        // The street or cottage may have been rebuilt in a previous save.
        self.body = cottage
            .route
            .iter()
            .find_map(|&[x, z]| footing(world, x, z, 13., 9.).map(|y| Body::new([x, y, z])));
    }

    fn walk(
        &mut self,
        world: &mut World,
        cottage: &Cottage,
        player: Option<&Player>,
        seek_shelter: bool,
        neighbors: &[Option<Position>],
    ) {
        let Some(body) = &mut self.body else { return };
        body.speed = 0.;
        let [x, y, z] = body.pos;
        if let Some(p) = player.filter(|p| (p.y - y).abs() < 1.6 && (p.x - x).hypot(p.z - z) < 3.) {
            body.face(p.x, p.z);
            return;
        }
        let [door_x, door_z] = cottage.doorstep;
        let in_doorway = (x - door_x).abs() < 0.17 && z >= door_z - 0.17;
        let goal = if seek_shelter {
            if in_doorway {
                [door_x, door_z + 2.8]
            } else {
                cottage.doorstep
            }
        } else if z > door_z + 0.2 {
            cottage.doorstep
        } else {
            if self.wait > 0. {
                self.wait = (self.wait - STEP).max(0.);
                return;
            }
            cottage.route[self.waypoint]
        };
        if (x - goal[0]).hypot(z - goal[1]) < 0.14 {
            if !seek_shelter {
                self.waypoint = (self.waypoint + 1) % cottage.route.len();
                self.wait = 1.4 + self.waypoint as f32 * 0.6;
            }
        } else {
            body.walk(world, goal, if seek_shelter { 1.2 } else { 0.78 }, 1., neighbors);
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Zombie {
    body: Body,
    health: u8,
    burn: f32,
    stagger: f32,
    #[serde(skip)]
    exposed: bool,
    #[serde(skip)]
    lunging: f32,
}

impl Zombie {
    fn new(pos: Position) -> Self {
        Self {
            body: Body::new(pos),
            health: ZOMBIE_HEALTH,
            burn: 0.,
            stagger: 0.,
            exposed: false,
            lunging: 0.,
        }
    }

    fn sunlight(&mut self, world: &mut World, sunny: bool, survey: bool) {
        if !sunny {
            self.exposed = false;
        } else if survey {
            let [x, y, z] = self.body.pos;
            self.exposed = !((y + BODY).floor() as i32..HEIGHT).any(|by| {
                ITEMS[world.get(x.floor() as i32, by, z.floor() as i32) as usize].height > 0.
            });
        }
        if self.exposed {
            self.burn += STEP;
            if self.burn >= 1. {
                self.health = self.health.saturating_sub(1);
                self.burn = 0.;
            }
        } else {
            self.burn = 0.;
        }
    }
}

fn contact_is_clear(world: &mut World, a: Position, b: Position) -> bool {
    let delta = std::array::from_fn(|i| b[i] - a[i]);
    let range = |i: usize| a[i].min(b[i]).floor() as i32..=a[i].max(b[i]).floor() as i32;
    for y in range(1) {
        for z in range(2) {
            for x in range(0) {
                let height = ITEMS[world.get(x, y, z) as usize].height;
                if height > 0. {
                    let lo = [x as f32, y as f32, z as f32];
                    let hi = [lo[0] + 1., lo[1] + height, lo[2] + 1.];
                    if ray_box(a, delta, lo, hi, 1.).is_some() {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn in_lab(x: f32, z: f32) -> bool {
    (-7.0..8.).contains(&x) && (-5.0..6.).contains(&z)
}

fn spawn_site(world: &mut World, x: i32, z: i32, player_y: f32) -> Option<Position> {
    if x.unsigned_abs() >= 999_999 || z.unsigned_abs() >= 999_999 || x.abs() <= 12 && z.abs() <= 8 {
        return None;
    }
    let y = (1..HEIGHT - 1).rev().find(|&y| {
        let id = world.get(x, y, z);
        is_water(id) || ITEMS[id as usize].height > 0.
    })?;
    if !matches!(world.get(x, y, z), GRASS | STONE | DIRT | SAND | SNOW)
        || (y as f32 + 1. - player_y).abs() > 9.
    {
        return None;
    }
    let pos = [x as f32 + 0.5, y as f32 + 1., z as f32 + 0.5];
    if Player::collides(world, pos[0], pos[1], pos[2]) {
        return None;
    }
    // A torch-lit camp gives explorers a place to rest. Check only at spawn.
    for dz in -4i32..=4 {
        for dx in -4i32..=4 {
            let reach = 4 - dx.abs() - dz.abs();
            for dy in -reach..=reach {
                if matches!(
                    world.get(x + dx, y + 1 + dy, z + dz),
                    TORCH | LAMP | TUBE_LIGHT | BEACON
                ) {
                    return None;
                }
            }
        }
    }
    Some(pos)
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub(super) struct Mobs {
    villagers: [Villager; COTTAGES.len()],
    zombies: [Option<Zombie>; ZOMBIE_LIMIT],
    spawn_wait: f32,
    grace: f32,
    swing_wait: f32,
    #[serde(skip)]
    remainder: f32,
    #[serde(skip)]
    survey_wait: f32,
}

impl Mobs {
    pub(super) fn valid(&self) -> bool {
        self.villagers.iter().zip(&COTTAGES).all(|(v, c)| {
            v.body.is_none_or(|b| {
                b.valid() && (b.pos[0] - c.doorstep[0]).hypot(b.pos[2] - c.doorstep[1]) < 32.
            }) && v.waypoint < c.route.len()
                && (0.0..=5.).contains(&v.wait)
                && v.stock <= FULL_STOCK
                && (0.0..=RESTOCK_SECONDS).contains(&v.restock)
        }) && self.zombies.iter().flatten().all(|z| {
            z.body.valid()
                && (1..=ZOMBIE_HEALTH).contains(&z.health)
                && (0.0..=1.).contains(&z.burn)
                && (0.0..=0.5).contains(&z.stagger)
        }) && (0.0..=12.).contains(&self.spawn_wait)
            && (0.0..=2.).contains(&self.grace)
            && (0.0..=0.5).contains(&self.swing_wait)
    }

    pub(super) fn is_zombie(id: u16) -> bool {
        (COTTAGES.len()..COTTAGES.len() + ZOMBIE_LIMIT).contains(&usize::from(id))
    }

    pub(super) fn offer(id: u16) -> Option<(&'static str, u8, u32, u8, u32)> {
        let c = COTTAGES.get(usize::from(id))?;
        Some((c.name, c.price.0, c.price.1, c.goods.0, c.goods.1))
    }

    pub(super) fn stock(&self, id: u16) -> u8 {
        self.villagers.get(usize::from(id)).map_or(0, |v| v.stock)
    }
    pub(super) fn hint(&self, id: u16) -> String {
        if let Some((name, cost, count, reward, quantity)) = Self::offer(id) {
            let stock = self.stock(id);
            return format!(
                "{name} · {count} {} 换 {quantity} {} · 余 {stock} 份",
                ITEMS[cost as usize].name, ITEMS[reward as usize].name
            );
        }
        self.zombie(id)
            .map_or_else(String::new, |z| format!("僵尸 · 生命 {}/{ZOMBIE_HEALTH}", z.health))
    }

    pub(super) fn trade(&mut self, id: u16, bag: &mut [u32]) -> Result<(u8, u32), String> {
        let Some((name, cost, count, reward, quantity)) = Self::offer(id) else {
            return Err("附近没有这位村民".into());
        };
        let villager = &mut self.villagers[usize::from(id)];
        if villager.body.is_none() {
            return Err("附近没有这位村民".into());
        }
        if villager.stock == 0 {
            return Err(format!("{name}还要 {} 秒才能补货", villager.restock.ceil() as u32));
        }
        if bag.get(cost as usize).copied().unwrap_or_default() < count {
            return Err(format!("{name}需要 {count} 个{}", ITEMS[cost as usize].name));
        }
        if bag
            .get(reward as usize)
            .and_then(|held| held.checked_add(quantity))
            .is_none()
        {
            return Err("背包装不下这份物资".into());
        }
        bag[cost as usize] -= count;
        if villager.stock == FULL_STOCK {
            villager.restock = RESTOCK_SECONDS;
        }
        villager.stock -= 1;
        Ok((reward, quantity))
    }

    fn zombie(&self, id: u16) -> Option<&Zombie> {
        let slot = usize::from(id).checked_sub(COTTAGES.len())?;
        self.zombies.get(slot)?.as_ref()
    }

    pub(super) fn strike(&mut self, id: u16, held: u8) -> String {
        if !Self::is_zombie(id) || self.swing_wait > 0. {
            return String::new();
        }
        let slot = &mut self.zombies[usize::from(id) - COTTAGES.len()];
        let Some(zombie) = slot else { return String::new() };
        self.swing_wait = 0.45;
        let power = if held == STICK { 2 } else { 1 + tool_tier(held) };
        zombie.health = zombie.health.saturating_sub(power);
        if zombie.health == 0 {
            *slot = None;
            return "击退了僵尸".into();
        }
        zombie.stagger = 0.4;
        format!("击中僵尸 · 生命 {}/{ZOMBIE_HEALTH}", zombie.health)
    }

    fn bodies(&self) -> impl Iterator<Item = (u16, &Body)> {
        self.villagers
            .iter()
            .map(|v| v.body.as_ref())
            .chain(self.zombies.iter().map(|z| z.as_ref().map(|z| &z.body)))
            .enumerate()
            .filter_map(|(i, b)| b.map(|b| (i as u16, b)))
    }

    pub(super) fn aim(&self, player: &Player, limit: f32) -> Option<(u16, f32)> {
        let from = player.eye();
        let dir = player.direction();
        let mut nearest = limit;
        let mut hit = None;
        for (id, b) in self.bodies() {
            let [x, y, z] = b.pos;
            let range = nearest.min(if Self::is_zombie(id) { 3. } else { 3.8 });
            let lo = [x - RADIUS, y, z - RADIUS];
            let hi = [x + RADIUS, y + BODY, z + RADIUS];
            if let Some((distance, _)) = ray_box(from, dir, lo, hi, range) {
                nearest = distance;
                hit = Some((id, distance));
            }
        }
        hit
    }

    pub(super) fn overlaps(&self, p: [i32; 3], height: f32) -> bool {
        let [x, y, z] = p.map(|v| v as f32);
        height > 0.
            && self.bodies().any(|(_, b)| {
                let [bx, by, bz] = b.pos;
                bx + RADIUS > x
                    && bx - RADIUS < x + 1.
                    && bz + RADIUS > z
                    && bz - RADIUS < z + 1.
                    && by + BODY > y
                    && by < y + height
            })
    }

    pub(super) fn poses(&self) -> Vec<f32> {
        // Read by the renderer at the AI tick rate. Stable slots keep interpolation
        // local to each mob.
        let mut out = Vec::with_capacity((COTTAGES.len() + ZOMBIE_LIMIT) * 8);
        for (id, b) in self.bodies() {
            let kind = usize::from(id).min(COTTAGES.len());
            let flags = self.zombie(id).map_or(0, |z| {
                u8::from(z.exposed) | u8::from(z.stagger > 0.) << 1 | u8::from(z.lunging > 0.) << 2
            });
            out.extend([
                f32::from(id),
                kind as f32,
                b.pos[0],
                b.pos[1],
                b.pos[2],
                b.yaw,
                b.speed,
                f32::from(flags),
            ]);
        }
        out
    }

    pub(super) fn advance(
        &mut self,
        world: &mut World,
        player: Option<&Player>,
        dt: f32,
        seconds: f32,
        night: bool,
        rain: bool,
    ) -> bool {
        self.remainder += dt.clamp(0., 0.2);
        let mut damage = false;
        while self.remainder >= STEP {
            self.remainder -= STEP;
            damage |= self.tick(world, player, seconds, night, rain);
        }
        damage
    }

    fn spawn(&mut self, world: &mut World, anchor: Position, seconds: f32) {
        let Some(slot) = self.zombies.iter().position(Option::is_none) else { return };
        let seed = hash(world.seed, seconds as i32, slot as i32);
        let offset = (seed & 65535) as f32 * std::f32::consts::TAU / 65536.;
        let radius = 14. + ((seed >> 16) % 8) as f32;
        for i in 0..8 {
            let (sin, cos) = (offset + i as f32 * std::f32::consts::FRAC_PI_4).sin_cos();
            let x = (anchor[0] + cos * radius).floor() as i32;
            let z = (anchor[2] + sin * radius).floor() as i32;
            if let Some(pos) = spawn_site(world, x, z, anchor[1]) {
                if self
                    .bodies()
                    .any(|(_, b)| (b.pos[0] - pos[0]).hypot(b.pos[2] - pos[2]) < 1.2)
                {
                    continue;
                }
                self.zombies[slot] = Some(Zombie::new(pos));
                return;
            }
        }
    }

    fn tick(
        &mut self,
        world: &mut World,
        player: Option<&Player>,
        seconds: f32,
        night: bool,
        rain: bool,
    ) -> bool {
        self.grace = (self.grace - STEP).max(0.);
        self.swing_wait = (self.swing_wait - STEP).max(0.);
        self.survey_wait -= STEP;
        let survey = self.survey_wait <= 0.;
        if survey {
            self.survey_wait = 0.8;
        }
        let mut neighbors = [None; COTTAGES.len() + ZOMBIE_LIMIT];
        for (i, b) in self.bodies() {
            neighbors[usize::from(i)] = Some(b.pos);
        }
        for (v, c) in self.villagers.iter_mut().zip(&COTTAGES) {
            v.resupply();
            if player.is_some_and(|p| (p.x - c.doorstep[0]).hypot(p.z - c.doorstep[1]) > 64.) {
                if let Some(b) = &mut v.body {
                    b.speed = 0.;
                }
                continue;
            }
            if survey {
                v.settle(world, c);
            }
            v.walk(world, c, player, night || rain, &neighbors);
        }
        // The original laboratory remains a refuge for the story. An observer watches
        // night activity around the settlement without becoming a target.
        let target = player.filter(|p| !in_lab(p.x, p.z));
        let anchor = player.map_or([1.5, 12., 9.5], |p| [p.x, p.y, p.z]);
        self.spawn_wait = (self.spawn_wait - STEP).max(0.);
        if night && self.spawn_wait == 0. && (target.is_some() || player.is_none()) {
            self.spawn(world, anchor, seconds);
            self.spawn_wait = 8.;
        }
        let mut damage = false;
        for (i, slot) in self.zombies.iter_mut().enumerate() {
            let Some(z) = slot else { continue };
            let [x, y, bz] = z.body.pos;
            if (anchor[0] - x).hypot(anchor[2] - bz) > 48. || (anchor[1] - y).abs() > 24. {
                *slot = None;
                continue;
            }
            if survey {
                let Some(top) = footing(world, x, bz, y + 1., y - 6.) else {
                    *slot = None;
                    continue;
                };
                z.body.pos[1] = top;
            }
            z.sunlight(world, !night && !rain, survey);
            if z.health == 0 {
                *slot = None;
                continue;
            }
            z.body.speed = 0.;
            z.stagger = (z.stagger - STEP).max(0.);
            z.lunging = (z.lunging - STEP).max(0.);
            if z.stagger > 0. {
                continue;
            }
            let side = if i % 2 == 0 { 1. } else { -1. };
            let Some(p) = target else {
                let direction = hash(world.seed, (seconds / 6.) as i32, i as i32) % 10;
                if direction < 8 {
                    let (dx, dz) = (direction as f32 * std::f32::consts::FRAC_PI_4).sin_cos();
                    if !in_lab(x + dx, bz + dz) {
                        z.body
                            .walk(world, [x + dx, bz + dz], 0.65, side, &neighbors);
                    }
                }
                continue;
            };
            let distance = (p.x - x).hypot(p.z - bz);
            if distance > 30. {
                continue;
            }
            let y = z.body.pos[1];
            if distance < 1.45
                && (p.y - y).abs() < 1.25
                && contact_is_clear(world, [x, y + 1.1, bz], [p.x, p.y + 1.1, p.z])
            {
                z.body.face(p.x, p.z);
                if self.grace == 0. {
                    damage = true;
                    self.grace = 1.8;
                    z.lunging = 0.3;
                }
            } else {
                z.body.walk(world, [p.x, p.z], 1.85, side, &neighbors);
            }
        }
        damage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player_at(pos: Position) -> Player {
        let mut player = Player::default();
        [player.x, player.y, player.z] = pos;
        player
    }

    #[test]
    fn villagers_shelter_in_their_cottages_and_keep_their_trade_stock() {
        let mut world = World::new(75);
        let mut mobs = Mobs::default();
        let player = Player::default();
        // An old save may contain a building on an original daytime waypoint.
        let [x, z] = COTTAGES[0].route[0].map(|v| v.floor() as i32);
        world.set([x, 12, z], BRICK);
        world.set([x, 13, z], BRICK);
        mobs.advance(&mut world, Some(&player), STEP, 0., false, false);
        let pos = mobs.villagers[0]
            .body
            .expect("resident spawned in the settlement")
            .pos;
        assert_ne!([pos[0], pos[2]], COTTAGES[0].route[0]);
        assert!(!Player::collides(&mut world, pos[0], pos[1], pos[2]));
        let mut bag = vec![0; ITEMS.len()];
        assert!(mobs.trade(0, &mut bag).is_err());
        bag[LOG as usize] = u32::from(FULL_STOCK) + 1;
        for _ in 0..FULL_STOCK {
            assert_eq!(mobs.trade(0, &mut bag), Ok((APPLE, 3)));
        }
        assert!(mobs.trade(0, &mut bag).is_err());
        assert_eq!(bag[LOG as usize], 1);
        let mut mobs: Mobs =
            serde_json::from_str(&serde_json::to_string(&mobs).expect("serialize residents"))
                .expect("restore residents");
        assert!(mobs.valid());
        assert!(mobs.trade(0, &mut bag).is_err());
        for step in 0..620 {
            mobs.advance(&mut world, Some(&player), STEP, step as f32 * STEP, false, true);
        }
        for (v, c) in mobs.villagers.iter().zip(COTTAGES) {
            let [x, y, z] = v.body.expect("resident spawned in the settlement").pos;
            assert!(z > c.doorstep[1] + 2. && (x - c.doorstep[0]).abs() < 0.2);
            assert!(ITEMS[world.get(x.floor() as i32, 16, z.floor() as i32) as usize].height > 0.);
            assert!(!Player::collides(&mut world, x, y, z));
        }
        assert_eq!(mobs.trade(0, &mut bag), Ok((APPLE, 3)));
        for step in 620..850 {
            mobs.advance(&mut world, Some(&player), STEP, step as f32 * STEP, false, false);
        }
        for (v, c) in mobs.villagers.iter().zip(COTTAGES) {
            assert!(
                v.body.expect("resident spawned in the settlement").pos[2] < c.doorstep[1] + 0.2
            );
        }
    }

    #[test]
    fn walls_block_movement_and_melee_then_an_open_path_allows_defense() {
        let mut world = World::new(47);
        for x in 40..=56 {
            for z in 16..=26 {
                world.set([x, 39, z], STONE);
            }
        }
        for z in 16..=26 {
            for y in 40..=42 {
                world.set([47, y, z], BRICK);
            }
        }
        let id = COTTAGES.len() as u16;
        let mut mobs = Mobs::default();
        mobs.zombies[0] = Some(Zombie::new([46.6, 40., 21.5]));
        let mut player = player_at([48.35, 40., 21.5]);
        player.yaw = std::f32::consts::FRAC_PI_2;
        let wall = player.pick(&mut world).expect("wall in reach");
        assert_eq!(wall.id, BRICK);
        assert_eq!(mobs.aim(&player, wall.distance), None);
        assert_eq!(mobs.aim(&player, 3.).expect("zombie in reach").0, id);
        assert!(!contact_is_clear(&mut world, [46.9, 41., 21.5], [48.1, 41., 21.5]));
        for step in 0..20 {
            assert!(!mobs.advance(
                &mut world,
                Some(&player),
                STEP,
                step as f32 * STEP,
                false,
                true
            ));
            let [x, y, z] = mobs.zombie(id).expect("zombie is still present").body.pos;
            assert!(x + RADIUS <= 47.001);
            assert!(!Player::collides(&mut world, x, y, z));
        }
        for z in 16..=26 {
            for y in 40..=42 {
                world.set([47, y, z], AIR);
            }
        }
        let mut hits = 0;
        for step in 20..45 {
            hits += usize::from(mobs.advance(
                &mut world,
                Some(&player),
                STEP,
                step as f32 * STEP,
                false,
                true,
            ));
        }
        assert!((1..=2).contains(&hits));
        mobs.strike(id, STICK);
        assert_eq!(mobs.zombie(id).expect("zombie is still present").health, 2);
        mobs.strike(id, STICK);
        assert_eq!(mobs.zombie(id).expect("zombie is still present").health, 2);
        for step in 45..51 {
            mobs.advance(&mut world, Some(&player), STEP, step as f32 * STEP, false, true);
        }
        mobs.strike(id, STICK);
        assert!(mobs.zombie(id).is_none());
    }

    #[test]
    fn observing_at_night_populates_the_settlement_and_sunburn_respects_roofs_and_rain() {
        let mut world = World::new(23);
        let mut mobs = Mobs::default();
        for step in 0..420 {
            assert!(!mobs.advance(&mut world, None, STEP, step as f32 * STEP, true, false));
        }
        assert!(mobs.zombies.iter().flatten().count() >= 3);
        for z in mobs.zombies.iter().flatten() {
            let [x, y, bz] = z.body.pos;
            assert!(!Player::collides(&mut world, x, y, bz));
        }
        let mut z = Zombie::new([29.5, 12., 21.5]);
        assert!(spawn_site(&mut world, 29, 21, 12.).is_some());
        world.set([30, 12, 21], TORCH);
        assert!(spawn_site(&mut world, 29, 21, 12.).is_none());
        world.set([29, 15, 21], PLANK);
        for _ in 0..16 {
            z.sunlight(&mut world, true, true);
        }
        assert_eq!(z.health, ZOMBIE_HEALTH);
        world.set([29, 15, 21], AIR);
        for _ in 0..16 {
            z.sunlight(&mut world, false, true);
        }
        assert_eq!(z.health, ZOMBIE_HEALTH);
        for _ in 0..16 {
            z.sunlight(&mut world, true, true);
        }
        assert_eq!(z.health, ZOMBIE_HEALTH - 1);
        assert!(z.exposed);
        z.sunlight(&mut world, false, true);
        assert!(!z.exposed);
    }
}
