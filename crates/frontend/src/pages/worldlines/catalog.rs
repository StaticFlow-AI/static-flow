use serde::Serialize;

#[derive(Serialize)]
pub struct Item {
    pub key: &'static str,
    pub name: &'static str,
    pub form: &'static str,
    pub height: f32,
    pub tier: u8,
    pub seconds: f32,
}

macro_rules! items {
    ($( $id:ident = $number:literal, $key:literal, $name:literal, $form:literal, $height:expr, $tier:expr, $time:expr; )+) => {
        $(pub const $id: u8 = $number;)+
        pub const ITEMS: &[Item] = &[$(Item { key:$key, name:$name, form:$form, height:$height, tier:$tier, seconds:$time },)+];
    };
}
items! {
    AIR=0,"air","空气","air",0.,0,0.;
    GRASS=1,"grass","草方块","cube",1.,0,0.48;
    DIRT=2,"dirt","泥土","cube",1.,0,0.4;
    STONE=3,"stone","岩石","cube",1.,1,0.95;
    BEDROCK=4,"bedrock","基岩","cube",1.,9,0.;
    LOG=5,"log","橡木原木","cube",1.,0,0.83;
    LEAVES=6,"leaves","橡树叶","cutout",1.,0,0.25;
    PLANK=7,"plank","橡木板","cube",1.,0,0.55;
    COBBLE=8,"cobble","圆石","cube",1.,1,0.75;
    SAND=9,"sand","河沙","cube",1.,0,0.35;
    COAL=10,"coal_ore","煤矿石","cube",1.,1,0.96;
    COPPER=11,"copper_ore","铜矿石","cube",1.,2,1.1;
    CRYSTAL=12,"crystal_ore","共振晶簇","cube",1.,3,1.22;
    BRICK=13,"brick","红砖","cube",1.,1,0.85;
    PLASTER=14,"plaster","灰泥墙","cube",1.,0,0.65;
    GLASS=15,"glass","窗玻璃","cutout",1.,0,0.25;
    STEEL=16,"steel","压纹钢板","cube",1.,2,0.95;
    TILE=17,"tile","棋盘地砖","cube",1.,1,0.7;
    SHELF=18,"shelf","书架","cube",1.,0,0.65;
    BENCH=19,"bench","工作台","cube",1.,0,0.7;
    FURNACE=20,"furnace","熔炉","cube",1.,1,0.9;
    RELAY=21,"relay","世界线中继器","prop",0.65,0,0.65;
    WAVE=22,"wave","电话微波炉（暂定）","prop",0.79,0,0.65;
    PHONE=23,"phone","翻盖手机","prop",0.32,0,0.28;
    CRT=24,"crt","42 英寸显像管","prop",0.96,0,0.68;
    LEAP=25,"leap","时间跳跃耳机","prop",0.46,0,0.36;
    UPA=26,"upa","金属乌帕","prop",0.35,0,0.28;
    SOFA=27,"sofa","墨绿沙发","prop",0.58,0,0.55;
    TABLE=28,"table","矮茶几","prop",1.,0,0.55;
    CABINET=29,"cabinet","工作柜","cube",1.,0,0.55;
    MONITOR=30,"monitor","电脑终端","prop",0.78,0,0.55;
    FLOWER=31,"flower","野雏菊","prop",0.,0,0.18;
    TORCH=32,"torch","火把","prop",0.,0,0.18;
    LAMP=33,"lamp","乳白方灯","cube",1.,0,0.28;
    WATER=34,"water","河水","liquid",0.,0,0.;
    SLAB=35,"slab","半高石阶","slab",0.5,1,0.55;
    BOOKS=36,"books","实验手册","prop",0.36,0,0.28;
    CURTAIN=37,"curtain","浴帘","prop",0.,0,0.3;
    NIXIE=38,"nixie","世界线变动率探测仪","prop",0.6,0,0.5;
    CUSHION=39,"panda","熊猫靠垫","prop",0.5,0,0.3;
    PINE=40,"pine","山地针叶","cutout",1.,0,0.25;
    SNOW=41,"snow","积雪岩","cube",1.,1,0.7;
    STICK=42,"stick","木棍","item",0.,0,0.;
    WOOD_PICK=43,"wood_pick","木镐","item",0.,0,0.;
    STONE_PICK=44,"stone_pick","石镐","item",0.,0,0.;
    COPPER_PICK=45,"copper_pick","铜镐","item",0.,0,0.;
    FUEL=46,"fuel","煤","item",0.,0,0.;
    ORE=47,"ore","粗铜","item",0.,0,0.;
    INGOT=48,"ingot","铜锭","item",0.,0,0.;
    SHARD=49,"shard","共振晶体","item",0.,0,0.;
    ROAD=50,"road","沥青","cube",1.,1,0.85;
    STRIPE=51,"stripe","斑马线","cube",1.,1,0.85;
    SIGN=52,"sign","未来道具研究所招牌","prop",0.,0,0.45;
    SAPLING=53,"sapling","橡树苗","prop",0.,0,0.25;
    BEACON=54,"beacon","星辉灯","cube",1.,0,0.35;
    RUG=55,"rug","编织地毯","prop",0.,0,0.26;
    FERN=56,"fern","野草","prop",0.,0,0.18;
    WATCH=57,"watch","实验室挂钟","prop",0.,0,0.35;
    APPLE=58,"apple","苹果","item",0.,0,0.;
    ROOF=59,"roof","屋面板","cube",1.,1,0.8;
    FAN=60,"fan","台扇","prop",0.93,0,0.48;
    PINBOARD=61,"pinboard","世界线研究笔记","prop",0.,0,0.4;
    TUBE_LIGHT=62,"tube_light","吊管灯","prop",0.,0,0.4;
    RADIO=63,"radio","短波收音机","prop",0.65,0,0.4;
    CUP=64,"cup","实验室马克杯","prop",0.35,0,0.2;
    AWNING=65,"awning","帆布遮阳篷","prop",0.85,0,0.45;
    RAIL=66,"rail","铁艺栏杆","prop",1.,1,0.6;
    BUCKET=67,"bucket","空木桶","item",0.,0,0.;
    WATER_BUCKET=68,"water_bucket","装水的木桶","item",0.,0,0.;
    WATER_1=69,"water_1","流水","liquid",0.,0,0.;
    WATER_2=70,"water_2","流水","liquid",0.,0,0.;
    WATER_3=71,"water_3","流水","liquid",0.,0,0.;
    WATER_4=72,"water_4","流水","liquid",0.,0,0.;
    WATER_5=73,"water_5","流水","liquid",0.,0,0.;
    WATER_6=74,"water_6","流水","liquid",0.,0,0.;
    WATER_7=75,"water_7","浅水","liquid",0.,0,0.;
    FALLING_WATER=76,"falling_water","瀑布","liquid",0.,0,0.;
}
pub const WATER_LEVELS: [u8; 7] = [WATER_1, WATER_2, WATER_3, WATER_4, WATER_5, WATER_6, WATER_7];

pub fn is_water(id: u8) -> bool {
    id == WATER || (WATER_1..=FALLING_WATER).contains(&id)
}

// Exposed water surfaces are measured in sixteenths. A cell covered by water is
// full.
pub fn water_height(id: u8) -> u8 {
    match id {
        WATER => 14,
        WATER_1..=WATER_7 => 13 - 2 * (id - WATER_1),
        FALLING_WATER => 16,
        _ => 0,
    }
}

// Selection boxes use the same sixteenths-of-a-block scale as the voxel models.
pub fn bounds(id: u8) -> [[f32; 3]; 2] {
    let (lo, hi) = match id {
        id if is_water(id) => ([0, 0, 0], [16, water_height(id), 16]),
        RUG => ([0, 0, 0], [16, 1, 16]),
        PHONE => ([5, 0, 4], [11, 5, 12]),
        UPA => ([4, 0, 4], [12, 7, 12]),
        LEAP => ([2, 0, 3], [14, 7, 13]),
        WAVE => ([0, 0, 1], [16, 13, 15]),
        RELAY => ([1, 0, 2], [15, 10, 14]),
        CRT | MONITOR => ([0, 0, 1], [16, 14, 16]),
        SOFA => ([0, 0, 1], [16, 15, 15]),
        CUSHION => ([2, 0, 3], [14, 10, 9]),
        BOOKS => ([1, 0, 4], [14, 6, 13]),
        CURTAIN => ([2, 0, 0], [4, 16, 16]),
        NIXIE => ([0, 0, 3], [16, 9, 13]),
        TORCH => ([7, 0, 7], [10, 12, 10]),
        FLOWER | FERN | SAPLING => ([4, 0, 5], [12, 10, 11]),
        SLAB => ([0, 0, 0], [16, 8, 16]),
        SIGN => ([0, 2, 0], [16, 14, 2]),
        WATCH => ([4, 5, 0], [12, 14, 2]),
        FAN => ([3, 0, 4], [14, 15, 12]),
        PINBOARD => ([0, 1, 0], [16, 14, 3]),
        TUBE_LIGHT => ([0, 11, 4], [16, 16, 9]),
        RADIO => ([1, 0, 4], [15, 13, 12]),
        CUP => ([5, 0, 5], [12, 6, 10]),
        AWNING => ([0, 4, 0], [16, 14, 16]),
        RAIL => ([1, 0, 0], [4, 16, 16]),
        _ => ([0, 0, 0], [16, 16, 16]),
    };
    [lo, hi].map(|p| p.map(|n| n as f32 / 16.))
}
pub fn drop_item(id: u8) -> u8 {
    match id {
        GRASS => DIRT,
        STONE => COBBLE,
        COAL => FUEL,
        COPPER => ORE,
        CRYSTAL => SHARD,
        LEAVES => SAPLING,
        _ => id,
    }
}
pub fn tool_tier(id: u8) -> u8 {
    match id {
        WOOD_PICK => 1,
        STONE_PICK => 2,
        COPPER_PICK => 3,
        _ => 0,
    }
}
pub fn placeable(id: u8) -> bool {
    id > 0
        && (id as usize) < ITEMS.len()
        && id != BEDROCK
        && !is_water(id)
        && ITEMS[id as usize].form != "item"
}

#[derive(Serialize)]
pub struct Recipe {
    pub output: u8,
    pub amount: u16,
    pub station: u8,
    pub ingredients: &'static [(u8, u16)],
}
macro_rules! recipes {
    ($( $id:ident * $n:literal @ $station:ident : $($input:ident * $qty:literal),+; )+) => {
        pub const RECIPES:&[Recipe]=&[$(Recipe{output:$id,amount:$n,station:$station,ingredients:&[$(($input,$qty)),+]},)+];
    };
}
recipes! {
    PLANK*4 @ AIR: LOG*1;
    STICK*4 @ AIR: PLANK*2;
    BENCH*1 @ AIR: PLANK*4;
    WOOD_PICK*1 @ BENCH: PLANK*3, STICK*2;
    STONE_PICK*1 @ BENCH: COBBLE*3, STICK*2;
    FURNACE*1 @ BENCH: COBBLE*6;
    INGOT*1 @ FURNACE: ORE*1, FUEL*1;
    COPPER_PICK*1 @ BENCH: INGOT*3, STICK*2;
    RELAY*1 @ BENCH: INGOT*2, SHARD*1;
    TORCH*4 @ AIR: FUEL*1, STICK*1;
    SLAB*6 @ BENCH: COBBLE*3;
    GLASS*2 @ FURNACE: SAND*2, FUEL*1;
    BRICK*4 @ FURNACE: DIRT*4, FUEL*1;
    SHELF*1 @ BENCH: PLANK*3, STICK*2;
    LAMP*2 @ BENCH: GLASS*2, TORCH*2;
    SOFA*1 @ BENCH: PLANK*2, LEAVES*2;
    TABLE*1 @ BENCH: PLANK*2, STICK*2;
    BEACON*2 @ BENCH: GLASS*2, SHARD*1;
    BUCKET*1 @ AIR: PLANK*3;
}
