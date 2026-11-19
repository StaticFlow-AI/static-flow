use super::*;

// Drive the same reach, placement, recipe and save operations used by the
// browser.
fn look_at(g: &mut Sandbox, feet: [f32; 3], p: [f32; 3]) {
    [g.player.x, g.player.y, g.player.z] = feet;
    let dx = p[0] - feet[0];
    let dz = p[2] - feet[2];
    g.player.yaw = (-dx).atan2(-dz);
    g.player.pitch = (p[1] - feet[1] - 1.58).atan2(dx.hypot(dz));
}
fn excavate(g: &mut Sandbox, id: u8, count: usize) {
    let mut n = 0;
    for y in (3..=11).rev() {
        for x in 19..32 {
            for z in -7..4 {
                let pos = [x, y, z];
                if g.get(pos) == id {
                    g.mine_hit(Hit {
                        pos,
                        normal: [0, 1, 0],
                        id,
                        distance: 1.,
                    });
                    assert_eq!(g.get(pos), AIR);
                    n += 1;
                    if n == count {
                        return;
                    }
                }
            }
        }
    }
    panic!("authored quarry lacks {count} blocks of {}", ITEMS[id as usize].key);
}
fn on_floor(g: &mut Sandbox, p: [i32; 3], id: u8) {
    g.equip(id);
    let floor = [p[0], p[1] - 1, p[2]];
    let support = g.get(floor);
    assert_ne!(support, AIR);
    g.place_hit(Hit {
        pos: floor,
        id: support,
        normal: [0, 1, 0],
        distance: 1.,
    });
    assert_eq!(g.get(p), id);
}

#[test]
fn quarry_to_relay_and_reunion_preserve_the_editable_world() {
    let mut g = Sandbox::new(407);
    look_at(&mut g, [1.5, 12., 17.2], [1.5, 12.5, 14.5]);
    assert!(g.mine().contains("原木"));
    for y in 13..=15 {
        look_at(&mut g, [1.5, 12., if y > 13 { 14.65 } else { 17.2 }], [1.5, y as f32 + 0.4, 14.5]);
        g.mine();
    }
    assert_eq!(g.bag[LOG as usize], 4);
    for _ in 0..4 {
        g.craft(PLANK);
    }
    g.craft(STICK);
    g.craft(STICK);
    g.craft(WOOD_PICK);
    assert_eq!(g.bag[WOOD_PICK as usize], 0);
    g.player.x = -4.5;
    g.player.y = 17.;
    g.player.z = -1.5;
    g.craft(WOOD_PICK);
    g.equip(WOOD_PICK);
    assert!(!g.can_mine(COPPER));
    excavate(&mut g, STONE, 9);
    excavate(&mut g, COAL, 5);
    g.craft(STONE_PICK);
    g.equip(STONE_PICK);
    excavate(&mut g, COPPER, 5);
    assert!(!g.can_mine(CRYSTAL));
    g.craft(FURNACE);
    g.craft(INGOT);
    assert_eq!(g.bag[INGOT as usize], 0);
    on_floor(&mut g, [-5, 17, -1], FURNACE);
    for _ in 0..5 {
        g.craft(INGOT);
    }
    g.craft(COPPER_PICK);
    g.equip(COPPER_PICK);
    excavate(&mut g, CRYSTAL, 1);
    g.craft(RELAY);
    g.story.inspect("phone");
    g.story.inspect("crt");
    g.story.inspect("wave");
    assert!(!g.send(0));
    g.equip(RELAY);
    g.place_hit(Hit {
        pos: [-4, 18, -4],
        normal: [1, 0, 0],
        id: WAVE,
        distance: 2.,
    });
    assert!(g.linked());
    assert!(g.send(1));
    assert!(g.story.line == Line::Beta);
    g.story.inspect("mayuri");
    g.story.inspect("leap");
    assert!(g.leap());
    assert!(g.linked());
    g.story.inspect("upa");
    g.story.inspect("suzuha");
    g.story.inspect("crt");
    g.story.inspect("wave");
    assert!(g.send(2));
    g.story.inspect("mayuri");
    let mut restored = Sandbox::new(123);
    assert!(restored.restore(&g.save()));
    assert!(restored.story.reunion && restored.linked());
    assert_eq!(restored.story.mail, [1, 2]);
    assert_eq!(restored.get([-5, 17, -1]), FURNACE);
    assert_eq!(restored.bag[INGOT as usize], 0);
    assert_eq!(restored.bag[PLANK as usize], 9);
    on_floor(&mut restored, [1, 17, 3], PLANK);
    assert_eq!(restored.bag[PLANK as usize], 8);
}

#[test]
fn movement_stops_at_walls_and_changing_the_wall_opens_a_path() {
    let mut g = Sandbox::new(19);
    look_at(&mut g, [0.5, 17., -3.4], [0.5, 18.4, -5.5]);
    for _ in 0..38 {
        g.advance(0.03, 0., 1., false, false);
    }
    assert!((g.player.z + 3.71).abs() < 0.07);
    g.look(0., -1.03);
    g.mine();
    assert_eq!(g.get([0, 17, -5]), AIR);
    g.look(0., 1.18);
    g.mine();
    assert_eq!(g.get([0, 18, -5]), AIR);
    for _ in 0..25 {
        g.advance(0.03, 0., 1., false, false);
    }
    assert!(g.player.z < -4.7);
    g.home();
    g.advance(0.03, 0., 0., false, false);
    let floor = g.player.y;
    let mut apex = floor;
    for i in 0..60 {
        g.advance(0.02, 0., 0., i == 0, false);
        apex = apex.max(g.player.y);
    }
    assert!(apex > floor + 1.1 && (g.player.y - floor).abs() < 0.01);
}

#[test]
fn seam_edits_survive_eviction_and_invalid_saves_are_atomic() {
    let mut w = World::new(19);
    for (x, z) in [(-17, -16), (-16, -17), (15, 16), (16, 15)] {
        w.set([x, 32, z], BRICK);
        let cx = x.div_euclid(16);
        let cz = z.div_euclid(16);
        let dx = if x.rem_euclid(16) == 15 { 1 } else { -1 };
        let region = w.region(cx + dx, cz);
        let local_x = x - (cx + dx) * 16 + 1;
        let local_z = z - cz * 16 + 1;
        assert_eq!(region[(32 * 324 + local_z * 18 + local_x) as usize], BRICK);
    }
    w.retain(120, -120);
    assert_eq!(w.get(-17, 32, -16), BRICK);
    let mut g = Sandbox::new(41);
    assert!(!g.restore("{}"));
    let original = g.save();
    let mut bad: serde_json::Value =
        serde_json::from_str(&original).expect("serialize current save");
    bad["edits"] = serde_json::json!([[0, HEIGHT, 0, 255]]);
    assert!(!g.restore(&bad.to_string()));
    assert_eq!(g.save(), original);
}

#[test]
fn low_carpet_leaves_the_upa_reachable_and_a_seedling_can_be_harvested_again() {
    let mut g = Sandbox::new(71);
    look_at(&mut g, [2.7, 17., 3.4], [4.5, 17.19, 1.5]);
    assert_eq!(g.aim().first(), Some(&f32::from(UPA)));
    g.give(SAPLING, 1);
    on_floor(&mut g, [12, 12, 14], SAPLING);
    g.seconds += 66.;
    g.grow();
    assert_eq!(g.get([12, 12, 14]), LOG);
    assert_eq!(g.get([12, 16, 14]), LEAVES);
    assert!(g.planted.is_empty());
}

#[test]
fn falling_costs_health_and_respawn_preserves_buildings_and_inventory() {
    let mut g = Sandbox::new(71);
    g.player.x = 12.5;
    g.player.z = 16.5;
    g.player.y = 18.;
    for _ in 0..75 {
        g.advance(0.025, 0., 0., false, false);
    }
    assert!(g.health < 5 && g.health > 0);
    g.give(APPLE, 1);
    g.equip(APPLE);
    let health = g.health;
    g.eat();
    assert!(g.health > health);
    assert_eq!(g.bag[APPLE as usize], 0);
    g.give(BRICK, 3);
    on_floor(&mut g, [12, 12, 14], BRICK);
    g.player.y = 37.;
    for _ in 0..120 {
        g.advance(0.025, 0., 0., false, false);
    }
    assert_eq!(g.health, 5);
    assert!((g.player.x - Player::default().x).abs() < 0.01);
    assert_eq!(g.get([12, 12, 14]), BRICK);
    assert_eq!(g.bag[BRICK as usize], 2);
}

#[test]
fn the_quarry_removes_the_whole_surface_and_its_markers_have_footing() {
    let mut w = World::new(253);
    for x in 18..=32 {
        for z in -8..=4 {
            for y in 12..=18 {
                assert_eq!(w.get(x, y, z), AIR);
            }
        }
    }
    for (x, z) in [(17, 2), (33, -7)] {
        assert_eq!(w.get(x, 12, z), TORCH);
        assert_eq!(ITEMS[w.get(x, 11, z) as usize].height, 1.);
    }
}

#[test]
fn camera_tracks_the_same_stairs_across_frame_rates_and_uses_the_visible_eye_to_aim() {
    let travel = |dt: f32, frames: usize| {
        let mut g = Sandbox::new(42);
        g.player.x = 8.7;
        g.player.z = 6.6;
        g.player.y = 12.;
        g.player.pitch = 0.;
        g.player.yaw = 0.;
        let start = g.player.y;
        let mut high_step = 0_f32;
        let mut last = start;
        for _ in 0..frames {
            g.advance(dt, 0., 1., false, false);
            let pose = g.pose();
            high_step = high_step.max((pose[1] - last).abs());
            assert!((g.player.eye()[1] - pose[1] - 1.58).abs() < 1e-5);
            last = pose[1];
        }
        (g.player.view_position(), high_step)
    };
    let (fast, step) = travel(1. / 120., 240);
    let (slow, _) = travel(1. / 30., 60);
    assert!(fast[1] > 15.5 && step < 0.11, "stairs {fast:?}, eye jump {step}");
    for i in 0..3 {
        assert!((fast[i] - slow[i]).abs() < 0.04, "{fast:?} vs {slow:?}");
    }
}

#[test]
fn surveying_round_trips_without_changing_the_survivor_or_the_story_clock() {
    let mut g = Sandbox::new(44);
    g.advance(0.1, 0., 0., false, false);
    g.give(PLANK, 8);
    let body = g.player.pose();
    g.toggle_survey();
    let above = g.pose();
    assert!(above[1] > body[1] + 20. && above[4] < -0.6);
    assert!(g.set_weather("thunder") && g.set_hour(0));
    for _ in 0..30 {
        g.fly(0.1, 1., 1., 1., true);
    }
    let camera = g.pose();
    assert!(camera[1] > above[1] && (camera[0] - above[0]).abs() > 8.);
    assert!(g.aim().is_empty() && g.mine().is_empty() && g.place().is_empty());
    assert_eq!(g.player.pose(), body);
    let mut resumed = Sandbox::new(0);
    assert!(resumed.restore(&g.save()));
    assert_eq!(resumed.pose(), camera);
    assert_eq!(resumed.climate.weather(44, resumed.seconds), Weather::Thunder);
    assert_eq!(resumed.story.line.time(), g.story.line.time());
    resumed.toggle_survey();
    assert_eq!([resumed.player.x, resumed.player.y, resumed.player.z], [
        g.player.x, g.player.y, g.player.z
    ]);
    assert!((resumed.player.eye()[1] - g.player.eye()[1]).abs() < 0.001);
    assert_eq!(resumed.bag[PLANK as usize], 8);
    assert!(resumed.survey.is_none() && resumed.health == 5 && !resumed.set_hour(12));
}

#[test]
fn a_pre_expansion_save_keeps_buildings_inventory_and_progress_and_receives_a_bucket_once() {
    let mut g = Sandbox::new(91);
    g.change([-16, 30, 0], BRICK);
    g.give(PLANK, 7);
    g.story.inspect("phone");
    let mut old: Value = serde_json::from_str(&g.save()).expect("read sandbox save");
    for key in ["mobs", "climate", "survey", "water_queue"] {
        old.as_object_mut().expect("save object").remove(key);
    }
    old["bag"]
        .as_array_mut()
        .expect("inventory slots")
        .truncate(BUCKET as usize);
    old["bar"][8] = json!(AIR);
    let mut restored = Sandbox::new(0);
    assert!(restored.restore(&old.to_string()));
    assert_eq!(restored.get([-16, 30, 0]), BRICK);
    assert!(
        restored.story.phone
            && restored.bag[PLANK as usize] == 7
            && restored.bag[BUCKET as usize] == 1
    );
    let current = restored.save();
    assert!(restored.restore(&current));
    assert_eq!(restored.bag[BUCKET as usize], 1);
}
