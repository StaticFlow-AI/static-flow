use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

use super::{
    catalog::*,
    player::{ray_box, Hit, Player, REACH},
    story::{self, Line, Panel, Story, CHOICES},
    terrain::{World, HEIGHT},
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
}
#[wasm_bindgen]
pub struct Sandbox {
    world: World,
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
        Self {
            world: World::new(seed),
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
        }
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
            && id != WATER
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
            || !matches!(self.get(p), AIR | WATER)
            || self.player.overlaps(p, ITEMS[id as usize].height)
        {
            return "这里没有放置空间".into();
        }
        if id == SAPLING && !matches!(self.world.get(p[0], p[1] - 1, p[2]), DIRT | GRASS) {
            return "树苗需要泥土或草方块".into();
        }
        if ITEMS[id as usize].height > 0.
            && ACTORS.iter().enumerate().any(|(i, (name, pos, _))| {
                self.visible(name)
                    && p[0] == pos[0].floor() as i32
                    && p[2] == pos[2].floor() as i32
                    && (p[1] as f32) < self.actor_y[i] + 1.8
                    && (p[1] as f32) + ITEMS[id as usize].height > self.actor_y[i]
            })
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
    fn actor_hit(&self, max: f32) -> Option<&'static str> {
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
                target = Some(id);
            }
        }
        target
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
            "panel":self.panel(),"health":self.health,"actorY":self.actor_y,"mined":self.mined,"built":self.built,"seconds":self.seconds,"seed":self.world.seed}).to_string()
    }
    pub fn pose(&self) -> Vec<f32> {
        self.player.pose()
    }
    pub fn look(&mut self, yaw: f32, pitch: f32) {
        self.player.look(yaw, pitch)
    }
    pub fn advance(&mut self, dt: f32, side: f32, forward: f32, jump: bool, sprint: bool) -> bool {
        let hp = self.health;
        let dirty = self.dirty.len();
        let stamp = self.seconds as u32;
        self.seconds += dt.clamp(0., 0.2);
        self.player
            .advance(&mut self.world, dt, side, forward, jump, sprint);
        if !self.planted.is_empty() && stamp != self.seconds as u32 {
            self.grow();
        }
        let p = self.player;
        self.fall_top = self.fall_top.max(p.y);
        if self
            .world
            .get(p.x.floor() as i32, (p.y + 0.25).floor() as i32, p.z.floor() as i32)
            == WATER
        {
            self.fall_top = p.y;
        }
        if p.grounded {
            let damage = (self.fall_top - p.y - 3.).floor().max(0.) as u8;
            self.fall_top = p.y;
            if damage > 0 {
                self.health = self.health.saturating_sub(damage);
                if self.health == 0 {
                    self.health = 5;
                    self.home();
                    return true;
                }
            }
        }
        hp != self.health || dirty != self.dirty.len()
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
        let block = self.player.pick(&mut self.world);
        let actor = self.actor_hit(block.as_ref().map_or(REACH, |hit| hit.distance));
        if let Some(id) = actor {
            let i = ACTORS.iter().position(|a| a.0 == id).unwrap_or_default();
            return vec![-1., i as f32];
        }
        match block {
            Some(h) => vec![
                h.id as f32,
                h.pos[0] as f32,
                h.pos[1] as f32,
                h.pos[2] as f32,
                h.normal[0] as f32,
                h.normal[1] as f32,
                h.normal[2] as f32,
            ],
            None => Vec::new(),
        }
    }
    pub fn break_time(&self, id: u8) -> f32 {
        if id as usize >= ITEMS.len() || !self.can_mine(id) {
            return -1.;
        }
        ITEMS[id as usize].seconds / (1. + f32::from(tool_tier(self.held())) * 0.37)
    }
    pub fn mine(&mut self) -> String {
        let Some(hit) = self.player.pick(&mut self.world) else { return String::new() };
        if self.actor_hit(hit.distance).is_some() {
            return String::new();
        }
        self.mine_hit(hit)
    }
    pub fn place(&mut self) -> String {
        let Some(hit) = self.player.pick(&mut self.world) else {
            return "准星对准一个相邻方块的表面".into();
        };
        if self.actor_hit(hit.distance).is_some() {
            return "请瞄准方块表面".into();
        }
        self.place_hit(hit)
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
        let hit = self.player.pick(&mut self.world);
        if let Some(actor) = self.actor_hit(hit.as_ref().map_or(REACH, |h| h.distance)) {
            self.story.inspect(actor);
            return String::new();
        }
        let Some(h) = hit else { return String::new() };
        match h.id {
            BENCH => return "bench".into(),
            FURNACE => return "furnace".into(),
            WAVE if !self.linked() => return self.goal(false),
            WAVE | PHONE | LEAP | UPA | CRT => self.story.inspect(ITEMS[h.id as usize].key),
            NIXIE => return format!("世界线变动率 {} %", self.story.line.number()),
            _ => return "bag".into(),
        }
        String::new()
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
        self.story.panel = None;
    }
    pub fn home(&mut self) {
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
            next.world.restore_edit([x, y, z], id as u8);
        }
        next.bag = save.bag;
        next.bag.resize(ITEMS.len(), 0);
        next.bar = save.bar;
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
