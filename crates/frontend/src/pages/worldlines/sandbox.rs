use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

use super::{
    catalog::*,
    mobs::Mobs,
    player::{ray_box, Hit, Player, REACH},
    story::{self, Line, Panel, Story, CHOICES},
    survey::SurveyCamera,
    terrain::{World, HEIGHT},
    water::WaterFlow,
    weather::{self, Climate, Weather},
};

const ACTORS: [(&str, [f32; 3], f32); 8] = [
    ("okabe", [-4.65, 17., 1.15], 0.8),
    ("mayuri", [3.65, 17., 1.55], -0.95),
    ("daru", [5.15, 17., -2.45], -0.65),
    ("kurisu", [-2.55, 17., -2.35], 0.60),
    ("moeka", [2.6, 12., -1.6], -0.2),
    ("luka", [-10.5, 12., 7.2], 0.33),
    ("faris", [4.3, 12., 12.1], -2.2),
    ("suzuha", [0.3, 22., 0.35], 0.66),
];
const NEIGHBORS: [[i32; 3]; 6] =
    [[1, 0, 0], [-1, 0, 0], [0, 1, 0], [0, -1, 0], [0, 0, 1], [0, 0, -1]];
#[derive(Serialize, Deserialize)]
struct Save {
    version: u8,
    seed: u32,
    edits: Vec<[i32; 4]>,
    player: Player,
    bag: Vec<u32>,
    bar: [u8; 9],
    slot: u8,
    story: Story,
    seconds: f32,
    planted: Vec<([i32; 3], f32)>,
    built: u32,
    mined: u32,
    health: u8,
    #[serde(default)]
    climate: Climate,
    #[serde(default)]
    mobs: Mobs,
    #[serde(default)]
    survey: Option<SurveyCamera>,
    #[serde(default)]
    water_queue: Option<Vec<[i32; 3]>>,
}
enum Target {
    Sky,
    Block(Hit),
    Story(usize),
    Mob(u16),
}
#[wasm_bindgen]
pub struct Sandbox {
    world: World,
    water: WaterFlow,
    climate: Climate,
    mobs: Mobs,
    survey: Option<SurveyCamera>,
    trader: Option<u16>,
    player: Player,
    bag: Vec<u32>,
    bar: [u8; 9],
    slot: u8,
    story: Story,
    seconds: f32,
    planted: Vec<([i32; 3], f32)>,
    dirty: Vec<[i32; 3]>,
    built: u32,
    mined: u32,
    health: u8,
    fall_top: f32,
    actor_y: [f32; 8],
}
impl Sandbox {
    pub fn new(seed: u32) -> Self {
        let mut game = Self {
            world: World::new(seed),
            water: WaterFlow::default(),
            climate: Climate::default(),
            mobs: Mobs::default(),
            survey: None,
            trader: None,
            player: Player::default(),
            bag: vec![0; ITEMS.len()],
            bar: [0; 9],
            slot: 0,
            story: Story::default(),
            seconds: 0.,
            planted: Vec::new(),
            dirty: Vec::new(),
            built: 0,
            mined: 0,
            health: 5,
            fall_top: 17.,
            actor_y: ACTORS.map(|a| a.1[1]),
        };
        game.bag[BUCKET as usize] = 1;
        game.bar[8] = BUCKET;
        game
    }
    fn give(&mut self, id: u8, amount: u32) {
        self.bag[id as usize] = self.bag[id as usize].saturating_add(amount);
        if !self.bar.contains(&id) {
            if let Some(slot) = self.bar.iter_mut().find(|entry| **entry == AIR) {
                *slot = id;
            }
        }
    }
    fn change(&mut self, p: [i32; 3], id: u8) {
        self.world.set(p, id);
        self.water.wake(p);
        self.dirty.push(p);
        for (i, (_, pos, _)) in ACTORS.iter().enumerate() {
            if pos[0].floor() as i32 == p[0] && pos[2].floor() as i32 == p[2] {
                self.settle_actor(i);
            }
        }
    }
    fn settle_actor(&mut self, i: usize) {
        let p = ACTORS[i].1;
        self.actor_y[i] = (0..p[1] as i32)
            .rev()
            .find_map(|y| {
                let h = ITEMS[self.world.get(p[0].floor() as i32, y, p[2].floor() as i32) as usize]
                    .height;
                (h > 0.).then_some(y as f32 + h)
            })
            .unwrap_or(1.);
    }
    fn held(&self) -> u8 {
        self.bar[self.slot as usize]
    }
    fn get(&mut self, p: [i32; 3]) -> u8 {
        self.world.get(p[0], p[1], p[2])
    }
    fn can_mine(&self, id: u8) -> bool {
        id != AIR
            && !is_water(id)
            && id != BEDROCK
            && ITEMS[id as usize].tier <= tool_tier(self.held())
    }
    fn mine_hit(&mut self, hit: Hit) -> String {
        if !self.can_mine(hit.id) {
            return self.tool_hint(hit.id).into();
        }
        self.change(hit.pos, AIR);
        let drop = drop_item(hit.id);
        self.give(drop, 1);
        if hit.id == LEAVES {
            self.give(LEAVES, 1);
            if self.mined.is_multiple_of(3) {
                self.give(APPLE, 1);
            }
        }
        self.planted.retain(|(p, _)| *p != hit.pos);
        self.mined = self.mined.saturating_add(1);
        format!("+1 {}", ITEMS[drop as usize].name)
    }
    fn tool_hint(&self, id: u8) -> &'static str {
        match ITEMS[id as usize].tier {
            1 => "需要手持木镐或更好的镐",
            2 => "需要手持石镐或铜镐",
            3 => "需要手持铜镐",
            _ => "基岩无法开采",
        }
    }
    fn place_hit(&mut self, hit: Hit) -> String {
        let id = self.held();
        if !placeable(id) || self.bag[id as usize] == 0 {
            return "在快捷栏选择背包里可放置的方块".into();
        }
        let p = std::array::from_fn(|i| hit.pos[i] + hit.normal[i]);
        if !(1..HEIGHT).contains(&p[1])
            || !(self.get(p) == AIR || is_water(self.get(p)))
            || self.player.overlaps(p, ITEMS[id as usize].height)
        {
            return "这里没有放置空间".into();
        }
        if id == SAPLING && !matches!(self.world.get(p[0], p[1] - 1, p[2]), DIRT | GRASS) {
            return "树苗需要泥土或草方块".into();
        }
        if ITEMS[id as usize].height > 0.
            && (self.mobs.overlaps(p, ITEMS[id as usize].height)
                || ACTORS.iter().enumerate().any(|(i, (name, pos, _))| {
                    self.visible(name)
                        && p[0] == pos[0].floor() as i32
                        && p[2] == pos[2].floor() as i32
                        && (p[1] as f32) < self.actor_y[i] + 1.8
                        && (p[1] as f32) + ITEMS[id as usize].height > self.actor_y[i]
                }))
        {
            return "有人站在这里".into();
        }
        self.bag[id as usize] -= 1;
        self.change(p, id);
        self.built = self.built.saturating_add(1);
        if id == SAPLING {
            self.planted.push((p, self.seconds + 65.));
        }
        format!("已放置{}", ITEMS[id as usize].name)
    }
    fn bucket_at(&mut self, hit: Hit) -> String {
        let held = self.held();
        if self.bag[held as usize] == 0 {
            return "手里没有桶".into();
        }
        let (p, block, take, message) = if held == BUCKET {
            if hit.id != WATER {
                return "把空桶对准静止的水源".into();
            }
            (hit.pos, AIR, WATER_BUCKET, "已装满水桶")
        } else {
            let p = std::array::from_fn(|i| hit.pos[i] + hit.normal[i]);
            let old = self.get(p);
            if !(1..HEIGHT).contains(&p[1]) || old == WATER || (old != AIR && !is_water(old)) {
                return "在空处或流动的水中放置水源".into();
            }
            (p, WATER, BUCKET, "已倒出水源")
        };
        self.change(p, block);
        self.bag[held as usize] -= 1;
        self.give(take, 1);
        let slot = self.slot as usize;
        if let Some(other) = self.bar.iter().position(|&id| id == take) {
            self.bar.swap(slot, other);
        } else {
            self.bar[slot] = take;
        }
        for id in &mut self.bar {
            if *id == held && self.bag[held as usize] == 0 {
                *id = AIR;
            }
        }
        message.into()
    }
    fn grow(&mut self) {
        let mut waiting = Vec::new();
        for (p, time) in std::mem::take(&mut self.planted) {
            if self.seconds < time {
                waiting.push((p, time));
                continue;
            }
            let [x, y, z] = p;
            if self.get(p) != SAPLING {
                continue;
            }
            if !matches!(self.world.get(x, y - 1, z), DIRT | GRASS)
                || y + 5 >= HEIGHT
                || (1..=4).any(|i| self.world.get(x, y + i, z) != AIR)
            {
                waiting.push((p, self.seconds + 30.));
                continue;
            }
            for dy in 2..=4 {
                for dx in -2i32..=2 {
                    for dz in -2i32..=2 {
                        let at = [x + dx, y + dy, z + dz];
                        if dx * dx + dz * dz <= if dy == 4 { 2 } else { 5 } && self.get(at) == AIR {
                            self.change(at, LEAVES);
                        }
                    }
                }
            }
            for dy in 0..4 {
                self.change([x, y + dy, z], LOG);
            }
        }
        self.planted = waiting;
    }
    fn stations(&mut self) -> (bool, bool) {
        let [x, y, z] = [self.player.x, self.player.y + 0.8, self.player.z];
        let mut bench = false;
        let mut furnace = false;
        for dx in -4..=4 {
            for dz in -4..=4 {
                for dy in -3..=3 {
                    let (bx, by, bz) =
                        (x.floor() as i32 + dx, y.floor() as i32 + dy, z.floor() as i32 + dz);
                    let center = [bx as f32 + 0.5 - x, by as f32 + 0.5 - y, bz as f32 + 0.5 - z];
                    if center.iter().map(|n| n * n).sum::<f32>() > 15.0 {
                        continue;
                    }
                    match self.world.get(bx, by, bz) {
                        BENCH => bench = true,
                        FURNACE => furnace = true,
                        _ => {},
                    }
                }
            }
        }
        (bench, furnace)
    }
    fn linked(&mut self) -> bool {
        let waves: Vec<_> = self.world.waves.iter().copied().collect();
        waves.into_iter().any(|p| {
            NEIGHBORS
                .iter()
                .any(|d| self.get(std::array::from_fn(|i| p[i] + d[i])) == RELAY)
        })
    }
    fn visible(&self, id: &str) -> bool {
        !matches!((self.story.line, id), (Line::Echo, "mayuri") | (Line::Beta, "daru"))
    }
    fn actor_hit(&self, max: f32) -> Option<(usize, f32)> {
        let origin = self.player.eye();
        let dir = self.player.direction();
        let mut nearest = max;
        let mut target = None;
        for (i, (id, mut p, _)) in ACTORS.into_iter().enumerate() {
            p[1] = self.actor_y[i];
            if !self.visible(id) {
                continue;
            }
            let lo = [p[0] - 0.33, p[1], p[2] - 0.33];
            let hi = [p[0] + 0.33, p[1] + 1.85, p[2] + 0.33];
            if let Some((near, _)) = ray_box(origin, dir, lo, hi, nearest) {
                nearest = near;
                target = Some((i, near));
            }
        }
        target
    }
    fn target(&mut self, scoop: bool) -> Target {
        let block = if scoop {
            self.player.pick_with_bucket(&mut self.world, true)
        } else {
            self.player.pick(&mut self.world)
        };
        let limit = block.as_ref().map_or(REACH, |hit| hit.distance);
        let actor = self.actor_hit(limit);
        if let Some((id, _)) = self
            .mobs
            .aim(&self.player, actor.map_or(limit, |(_, distance)| distance))
        {
            Target::Mob(id)
        } else if let Some((i, _)) = actor {
            Target::Story(i)
        } else {
            block.map_or(Target::Sky, Target::Block)
        }
    }
    fn craft_allowed(&self, r: &Recipe, bench: bool, furnace: bool) -> bool {
        (r.station == AIR || (r.station == BENCH && bench) || (r.station == FURNACE && furnace))
            && r.ingredients
                .iter()
                .all(|&(id, n)| self.bag[id as usize] >= u32::from(n))
    }
    fn goal(&self, linked: bool) -> String {
        if self.story.line == Line::Gate && self.story.reunion {
            return "两人都已获救。世界继续运行，可以远行勘探、培育树林或扩建研究所。".into();
        }
        if self.story.phone && self.story.line == Line::Alpha && !linked {
            let has = |id: u8| self.bag[id as usize] > 0;
            return if has(RELAY) {
                "把中继器放在二楼电话微波炉紧邻的一格。"
            } else if has(COPPER_PICK) || has(SHARD) {
                "用铜镐采东侧露天矿的蓝色晶簇。在工作台用两枚铜锭和晶体装配中继器。"
            } else if has(STONE_PICK) || has(INGOT) {
                "用石镐采粗铜。工作台用六块圆石做熔炉，每份粗铜和煤烧一枚铜锭。先做铜镐。"
            } else if has(WOOD_PICK) {
                "研究所东边的露天矿有裸露岩层。木镐采圆石和煤，回来合成石镐。"
            } else {
                "到街道南边采原木。背包里做木板、木棍，再到二楼西墙的工作台做木镐。\
                 做出中继器才能发信。"
            }
            .into();
        }
        self.story.objective().1.into()
    }
    fn panel(&self) -> Value {
        if let Some((id, (name, price, units, reward, count))) = self
            .trader
            .and_then(|id| Mobs::offer(id).map(|offer| (id, offer)))
        {
            return json!({"type":"trade", "who":name, "price":price,"units":units,"reward":reward,"count":count,"stock":self.mobs.stock(id)});
        }
        match self.story.panel.as_ref() {
            Some(Panel::Talk(who, body)) => json!({"type":"talk","who":who,"body":body}),
            Some(Panel::Phone) => json!({"type":"phone"}),
            Some(Panel::Leap) => json!({"type":"leap"}),
            None => Value::Null,
        }
    }
}
#[wasm_bindgen]
impl Sandbox {
    pub fn catalog(&self) -> String {
        let actors: Vec<_> = ACTORS
            .iter()
            .map(|(id, pos, angle)| json!({"id":id,"name":story::name(id),"pos":pos,"angle":angle}))
            .collect();
        json!({"items":ITEMS,"bounds":(0..ITEMS.len() as u8).map(bounds).collect::<Vec<_>>(),"recipes":RECIPES,"actors":actors,"choices":CHOICES.iter().map(|c|[c.to,c.body]).collect::<Vec<_>>(),"height":HEIGHT}).to_string()
    }
    pub fn snapshot(&mut self) -> String {
        let (bench, furnace) = self.stations();
        let linked = self.linked();
        json!({"bag":self.bag,"bar":self.bar,"slot":self.slot,"bench":bench,"furnace":furnace,
            "linked":linked,"goal":self.goal(linked),"line":self.story.line.id(),"number":self.story.line.number(),"clock":self.story.line.time(),
            "phone":self.story.phone,"crt":self.story.crt,"armed":self.story.armed,"leaps":self.story.leaps,"reunion":self.story.reunion,
            "incoming":self.story.incoming(),"mail":self.story.mail,"send":(0..3).map(|i|linked&&self.story.can_send(i)).collect::<Vec<_>>(),
            "panel":self.panel(),"sky":self.climate.sky(self.world.seed, self.seconds),"survey":self.survey.is_some(),"health":self.health,"actorY":self.actor_y,"mined":self.mined,"built":self.built,"seconds":self.seconds,"seed":self.world.seed}).to_string()
    }
    pub fn pose(&self) -> Vec<f32> {
        self.survey
            .as_ref()
            .map_or_else(|| self.player.pose(), SurveyCamera::pose)
    }
    pub fn look(&mut self, yaw: f32, pitch: f32) {
        if let Some(camera) = &mut self.survey {
            camera.look(yaw, pitch);
        } else {
            self.player.look(yaw, pitch);
        }
    }
    pub fn toggle_survey(&mut self) {
        self.close();
        self.survey =
            if self.survey.is_some() { None } else { Some(SurveyCamera::above(&self.player)) };
    }
    pub fn landmark(&mut self, id: u8) {
        if let Some(camera) = &mut self.survey {
            camera.landmark(id);
        }
    }
    pub fn set_weather(&mut self, kind: &str) -> bool {
        self.survey.is_some() && self.climate.set_weather(kind)
    }
    pub fn set_hour(&mut self, hour: u8) -> bool {
        self.survey.is_some() && self.climate.set_hour(self.seconds, hour)
    }
    pub fn rain_heights(&mut self, cx: i32, cz: i32) -> Vec<f32> {
        weather::rain_heights(&mut self.world, cx, cz)
    }
    pub fn fly(&mut self, dt: f32, side: f32, forward: f32, lift: f32, fast: bool) -> bool {
        if let Some(camera) = &mut self.survey {
            camera.fly(dt, side, forward, lift, fast);
        }
        self.advance(dt, 0., 0., false, false)
    }
    pub fn advance(&mut self, dt: f32, side: f32, forward: f32, jump: bool, sprint: bool) -> bool {
        let hp = self.health;
        let stamp = self.seconds as u32;
        let before = self.climate.weather(self.world.seed, self.seconds);
        let dt = dt.clamp(0., 0.2);
        self.seconds += dt;
        let weather = self.climate.weather(self.world.seed, self.seconds);
        let raining = weather != Weather::Clear;
        let surveying = self.survey.is_some();
        if !surveying {
            self.player
                .advance(&mut self.world, dt, side, forward, jump, sprint);
        }
        self.water.advance(&mut self.world, dt, &mut self.dirty);
        if !self.planted.is_empty() && stamp != self.seconds as u32 {
            if raining {
                for (p, due) in &mut self.planted {
                    if weather::rain_surface(&mut self.world, p[0], p[2]) <= p[1] as f32 + 0.01 {
                        *due -= 1.;
                    }
                }
            }
            self.grow();
        }
        let sky = self.climate.sky(self.world.seed, self.seconds);
        let hurt = self.mobs.advance(
            &mut self.world,
            (!surveying).then_some(&self.player),
            dt,
            self.seconds,
            sky.daylight < 0.24 || weather == Weather::Thunder,
            raining,
        );
        if surveying {
            return before != weather;
        }
        if hurt {
            self.health = self.health.saturating_sub(1);
        }
        let p = self.player;
        self.fall_top = self.fall_top.max(p.y);
        if self.world.in_water([p.x, p.y + 0.25, p.z]) {
            self.fall_top = p.y;
        }
        if p.grounded {
            let damage = (self.fall_top - p.y - 3.).floor().max(0.) as u8;
            self.fall_top = p.y;
            self.health = self.health.saturating_sub(damage);
        }
        if self.health == 0 {
            self.health = 5;
            self.home();
            return true;
        }
        hp != self.health || before != weather
    }
    pub fn mob_poses(&self) -> Vec<f32> {
        self.mobs.poses()
    }
    pub fn mob_hint(&self, id: u16) -> String {
        self.mobs.hint(id)
    }
    pub fn has_changes(&self) -> bool {
        !self.dirty.is_empty()
    }
    pub fn region(&mut self, cx: i32, cz: i32) -> Vec<u8> {
        self.world.region(cx, cz)
    }
    pub fn trim(&mut self, cx: i32, cz: i32) {
        self.world.retain(cx, cz);
    }
    pub fn world_edits(&self) -> Vec<i32> {
        self.world.edits().into_iter().flatten().collect()
    }
    pub fn changes(&mut self) -> Vec<i32> {
        std::mem::take(&mut self.dirty)
            .into_iter()
            .flat_map(|[x, y, z]| [x, y, z, i32::from(self.world.get(x, y, z))])
            .collect()
    }
    pub fn aim(&mut self) -> Vec<f32> {
        if self.survey.is_some() {
            return Vec::new();
        }
        match self.target(self.held() == BUCKET) {
            Target::Sky => Vec::new(),
            Target::Story(i) => vec![-1., i as f32],
            Target::Mob(id) => vec![-2., f32::from(id)],
            Target::Block(h) => vec![
                h.id as f32,
                h.pos[0] as f32,
                h.pos[1] as f32,
                h.pos[2] as f32,
                h.normal[0] as f32,
                h.normal[1] as f32,
                h.normal[2] as f32,
            ],
        }
    }
    pub fn break_time(&self, id: u8) -> f32 {
        if id as usize >= ITEMS.len() || !self.can_mine(id) {
            return -1.;
        }
        ITEMS[id as usize].seconds / (1. + f32::from(tool_tier(self.held())) * 0.37)
    }
    pub fn mine(&mut self) -> String {
        if self.survey.is_some() {
            return String::new();
        }
        match self.target(false) {
            Target::Block(hit) => self.mine_hit(hit),
            Target::Mob(id) if Mobs::is_zombie(id) => self.mobs.strike(id, self.held()),
            _ => String::new(),
        }
    }
    pub fn place(&mut self) -> String {
        if self.survey.is_some() {
            return String::new();
        }
        let held = self.held();
        match self.target(held == BUCKET) {
            Target::Block(hit) if matches!(held, BUCKET | WATER_BUCKET) => self.bucket_at(hit),
            Target::Block(hit) => self.place_hit(hit),
            _ => "请瞄准要操作的方块".into(),
        }
    }
    pub fn select(&mut self, slot: u8) {
        if slot < 9 {
            self.slot = slot;
        }
    }
    pub fn equip(&mut self, id: u8) {
        if id != AIR && (id as usize) < ITEMS.len() && self.bag[id as usize] > 0 {
            if let Some(i) = self.bar.iter().position(|&b| b == id) {
                self.slot = i as u8;
            } else {
                self.bar[self.slot as usize] = id;
            }
        }
    }
    pub fn craft(&mut self, id: u8) -> String {
        let Some(r) = RECIPES.iter().find(|r| r.output == id) else { return String::new() };
        let (bench, furnace) = self.stations();
        if !self.craft_allowed(r, bench, furnace) {
            return "请检查材料数量，并靠近配方所需的工作台或熔炉".into();
        }
        for &(i, n) in r.ingredients {
            self.bag[i as usize] -= u32::from(n);
        }
        self.give(id, u32::from(r.amount));
        format!("制成 {} × {}", ITEMS[id as usize].name, r.amount)
    }
    pub fn interact(&mut self) -> String {
        if self.survey.is_some() {
            return String::new();
        }
        let held = self.held();
        let target = self.target(held == BUCKET);
        let h = match target {
            Target::Mob(id) if !Mobs::is_zombie(id) => {
                self.trader = Some(id);
                return String::new();
            },
            Target::Mob(id) => return self.mobs.hint(id),
            Target::Story(i) => {
                self.story.inspect(ACTORS[i].0);
                return String::new();
            },
            _ if self.held() == APPLE => return self.eat(),
            Target::Block(hit) => hit,
            Target::Sky => return String::new(),
        };
        match h.id {
            BENCH => return "bench".into(),
            FURNACE => return "furnace".into(),
            WAVE if !self.linked() => return self.goal(false),
            WAVE | PHONE | LEAP | UPA | CRT => self.story.inspect(ITEMS[h.id as usize].key),
            _ if held == WATER_BUCKET || (held == BUCKET && h.id == WATER) => {
                return self.bucket_at(h)
            },
            NIXIE => return format!("世界线变动率 {} %", self.story.line.number()),
            _ => return "bag".into(),
        }
        String::new()
    }
    pub fn exchange(&mut self) -> String {
        let Some(id) = self.trader else {
            return String::new();
        };
        if !matches!(self.target(false), Target::Mob(now) if now == id) {
            self.trader = None;
            return "请走近村民再交换".into();
        }
        match self.mobs.trade(id, &mut self.bag) {
            Ok((item, count)) => {
                self.give(item, count);
                format!("换得{} × {}", ITEMS[item as usize].name, count)
            },
            Err(message) => message,
        }
    }
    pub fn open_phone(&mut self) -> bool {
        if self.story.phone {
            self.story.panel = Some(Panel::Phone);
        }
        self.story.phone
    }
    pub fn send(&mut self, choice: usize) -> bool {
        if !self.linked() || !self.story.can_send(choice) {
            return false;
        }
        self.story.send(choice);
        true
    }
    pub fn leap(&mut self) -> bool {
        if !matches!(self.story.panel, Some(Panel::Leap)) {
            return false;
        }
        let before = self.story.leaps;
        self.story.leap();
        self.story.leaps != before
    }
    pub fn close(&mut self) {
        self.trader = None;
        self.story.panel = None;
    }
    pub fn home(&mut self) {
        self.survey = None;
        self.close();
        self.player.return_home(&mut self.world);
        self.fall_top = self.player.y;
    }
    pub fn eat(&mut self) -> String {
        if self.held() != APPLE || self.bag[APPLE as usize] == 0 {
            return "先从背包拿起苹果".into();
        }
        if self.health == 5 {
            return "生命值已满".into();
        }
        self.bag[APPLE as usize] -= 1;
        self.health = (self.health + 2).min(5);
        "吃下苹果，恢复了体力".into()
    }
    pub fn restart(&mut self, seed: u32) {
        *self = Self::new(seed);
    }
    pub fn save(&self) -> String {
        serde_json::to_string(&Save {
            version: 2,
            seed: self.world.seed,
            edits: self.world.edits(),
            player: self.player,
            bag: self.bag.clone(),
            bar: self.bar,
            slot: self.slot,
            story: self.story.clone(),
            seconds: self.seconds,
            planted: self.planted.clone(),
            built: self.built,
            mined: self.mined,
            health: self.health,
            climate: self.climate,
            mobs: self.mobs.clone(),
            survey: self.survey,
            water_queue: Some(self.water.pending()),
        })
        .unwrap_or_default()
    }
    pub fn restore(&mut self, data: &str) -> bool {
        let Ok(save) = serde_json::from_str::<Save>(data) else { return false };
        let valid_pos = |p: &[i32]| {
            p[0].unsigned_abs() < 1_000_000
                && (1..HEIGHT).contains(&p[1])
                && p[2].unsigned_abs() < 1_000_000
        };
        if save.version != 2
            || !save.climate.valid()
            || !save.mobs.valid()
            || save.survey.as_ref().is_some_and(|v| !v.valid())
            || save
                .water_queue
                .as_ref()
                .is_some_and(|q| q.iter().any(|p| !valid_pos(p)))
            || save.bag.is_empty()
            || save.bag.len() > ITEMS.len()
            || !(1..=5).contains(&save.health)
            || save.slot >= 9
            || !save.player.valid()
            || !save.seconds.is_finite()
            || save.seconds < 0.
            || save.bar.iter().any(|b| *b as usize >= ITEMS.len())
            || save.story.mail.iter().any(|i| *i >= 3)
            || save.edits.iter().any(|p| {
                !valid_pos(p)
                    || !(0..ITEMS.len() as i32).contains(&p[3])
                    || ITEMS[p[3] as usize].form == "item"
            })
            || save
                .planted
                .iter()
                .any(|(p, t)| !valid_pos(p) || !t.is_finite())
        {
            return false;
        }
        let mut next = Self::new(save.seed);
        for [x, y, z, id] in save.edits {
            next.world.set([x, y, z], id as u8);
            if save.water_queue.is_none() {
                next.water.wake([x, y, z]);
            }
        }
        if let Some(queue) = save.water_queue {
            next.water.resume(queue);
        }
        let needs_bucket = save.bag.len() <= BUCKET as usize;
        next.bag = save.bag;
        next.bag.resize(ITEMS.len(), 0);
        next.bar = save.bar;
        if needs_bucket {
            if next.bar[8] == AIR {
                next.bar[8] = BUCKET;
            }
            next.give(BUCKET, 1);
        }
        next.mobs = save.mobs;
        next.climate = save.climate;
        next.survey = save.survey;
        next.slot = save.slot;
        next.player = save.player;
        next.seconds = save.seconds;
        next.story = save.story;
        next.planted = save.planted;
        next.built = save.built;
        next.mined = save.mined;
        next.health = save.health;
        next.fall_top = next.player.y;
        for i in 0..ACTORS.len() {
            next.settle_actor(i);
        }
        *self = next;
        true
    }
    pub fn import_story(&mut self, data: &str) -> bool {
        let Ok(s) = serde_json::from_str::<Story>(data) else { return false };
        if s.mail.iter().any(|i| *i >= 3) {
            return false;
        }
        self.story = s;
        true
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
