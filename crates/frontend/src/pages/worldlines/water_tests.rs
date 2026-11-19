use super::{
    super::{
        catalog::{BUCKET, STONE, WATER_BUCKET},
        sandbox::Sandbox,
    },
    *,
};

fn settle(flow: &mut WaterFlow, world: &mut World) {
    for _ in 0..200 {
        if flow.pending.is_empty() {
            return;
        }
        flow.advance(world, STEP, &mut Vec::new());
    }
    panic!("water continued to schedule updates after the channel should have settled");
}

#[test]
fn water_advances_one_level_per_tick_crosses_a_negative_seam_and_drains() {
    let mut world = World::new(18);
    let mut flow = WaterFlow::default();
    for x in -25..=-7 {
        world.set([x, 31, -16], STONE);
        for z in [-17, -15] {
            world.set([x, 32, z], STONE);
        }
    }
    let source = [-16, 32, -16];
    world.set(source, WATER);
    flow.wake(source);
    let mut dirty = Vec::new();
    flow.advance(&mut world, STEP / 2., &mut dirty);
    assert!(dirty.is_empty());
    flow.advance(&mut world, STEP / 2., &mut dirty);
    assert_eq!(world.get(-17, 32, -16), WATER_1);
    assert_eq!(world.get(-18, 32, -16), AIR);
    settle(&mut flow, &mut world);
    for (i, level) in WATER_LEVELS.into_iter().enumerate() {
        let offset = i as i32 + 1;
        assert_eq!(world.get(-16 - offset, 32, -16), level);
        assert_eq!(world.get(-16 + offset, 32, -16), level);
    }
    assert_eq!(world.get(-24, 32, -16), AIR);
    assert_eq!(world.get(-8, 32, -16), AIR);
    assert!(world.in_water([-22.5, 32.02, -15.5]));
    assert!(!world.in_water([-22.5, 32.42, -15.5]));
    world.set(source, AIR);
    flow.wake(source);
    settle(&mut flow, &mut world);
    for x in -24..=-8 {
        assert_eq!(world.get(x, 32, -16), AIR);
    }
    assert!(world.edits().iter().all(|p| p[3] == i32::from(STONE)));
}

#[test]
fn opening_and_closing_a_sluice_releases_then_drains_a_waterfall_after_reload() {
    let mut world = World::new(77);
    let mut flow = WaterFlow::default();
    for x in 6..=24 {
        for z in -9..=9 {
            world.set([x, 29, z], STONE);
        }
    }
    // Leave the source itself exposed so a mid-air sideways spill would be visible.
    for y in 31..=33 {
        for [dx, dz] in SIDES {
            world.set([15 + dx, y, dz], STONE);
        }
    }
    let gate = [15, 31, 0];
    world.set(gate, STONE);
    world.set([15, 34, 0], WATER);
    flow.wake([15, 34, 0]);
    settle(&mut flow, &mut world);
    assert_eq!(world.get(15, 32, 0), FALLING_WATER);
    assert_eq!(world.get(16, 34, 0), AIR);
    assert_eq!(world.get(15, 30, 0), AIR);
    world.set(gate, AIR);
    flow.wake(gate);
    flow.advance(&mut world, STEP, &mut Vec::new());
    assert_eq!(world.get(15, 31, 0), FALLING_WATER);
    assert_eq!(world.get(15, 30, 0), AIR);

    // Save while the first falling column is still midair.
    let mut restored = World::new(world.seed);
    let mut resumed = WaterFlow::default();
    for [x, y, z, id] in world.edits() {
        restored.set([x, y, z], id as u8);
    }
    resumed.resume(flow.pending());
    assert_eq!(resumed.pending(), flow.pending());
    restored.retain(-200, 180);
    settle(&mut resumed, &mut restored);
    assert_eq!(restored.get(15, 30, 0), FALLING_WATER);
    assert_eq!(restored.get(16, 30, 0), WATER_1);
    assert_eq!(restored.get(22, 30, 0), WATER_7);
    restored.set(gate, STONE);
    resumed.wake(gate);
    settle(&mut resumed, &mut restored);
    assert_eq!(restored.get(15, 32, 0), FALLING_WATER);
    for x in 8..=22 {
        assert_eq!(restored.get(x, 30, 0), AIR);
    }
}

#[test]
fn a_supported_well_replenishes_but_an_open_drain_between_two_sources_does_not() {
    let mut world = World::new(13);
    let mut flow = WaterFlow::default();
    let well = [-11, 11, 14];
    for _ in 0..3 {
        assert_eq!(world.get(well[0], well[1], well[2]), WATER);
        world.set(well, AIR);
        flow.wake(well);
        settle(&mut flow, &mut world);
    }
    assert!(world.edits().is_empty());
    let drain = [0, 33, 0];
    for x in [-1, 1] {
        world.set([x, 32, 0], STONE);
        world.set([x, 33, 0], WATER);
    }
    flow.wake(drain);
    flow.advance(&mut world, STEP, &mut Vec::new());
    assert_eq!(world.get(0, 33, 0), WATER_1);
    world.set([0, 32, 0], STONE);
    flow.wake([0, 32, 0]);
    flow.advance(&mut world, STEP, &mut Vec::new());
    assert_eq!(world.get(0, 33, 0), WATER);
}

#[test]
fn the_starter_bucket_can_draw_from_the_well_pour_on_land_and_scoop_the_source_back() {
    let mut game = Sandbox::new(37);
    let mut save: serde_json::Value =
        serde_json::from_str(&game.save()).expect("read sandbox save");
    save["player"] = serde_json::json!({
        "x":-11.5, "y":12.5, "z":15.5, "yaw":0., "pitch":-1.145
    });
    assert!(game.restore(&save.to_string()));
    game.equip(BUCKET);
    assert_eq!(game.aim()[..4], [f32::from(WATER), -12., 11., 14.]);
    game.place();
    assert_eq!(game.changes(), [-12, 11, 14, i32::from(AIR)]);
    let filled: serde_json::Value = serde_json::from_str(&game.save()).expect("read sandbox save");
    assert_eq!(filled["bag"][WATER_BUCKET as usize], 1);
    assert_eq!(filled["bag"][BUCKET as usize], 0);

    game.look(std::f32::consts::PI, 0.345);
    game.place();
    assert_eq!(game.changes(), [-12, 12, 17, i32::from(WATER)]);
    game.advance(STEP, 0., 0., false, false);
    assert!(game
        .world_edits()
        .as_chunks::<4>()
        .0
        .iter()
        .any(|p| p[3] == i32::from(WATER_1)));
    assert_eq!(game.aim()[..4], [f32::from(WATER), -12., 12., 17.]);
    game.place();
    let packed = game.save();
    assert!(game.restore(&packed));
    for _ in 0..16 {
        game.advance(STEP, 0., 0., false, false);
    }
    assert!(game.world_edits().is_empty());
    let end: serde_json::Value = serde_json::from_str(&game.save()).expect("read sandbox save");
    assert_eq!(end["bag"][WATER_BUCKET as usize], 1);
    assert_eq!(end["bag"][BUCKET as usize], 0);
    assert_eq!(end["water_queue"], serde_json::json!([]));
}
