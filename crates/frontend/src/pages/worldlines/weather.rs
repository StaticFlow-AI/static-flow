use serde::{Deserialize, Serialize};

use super::{
    catalog::{water_height, AWNING, ITEMS, TABLE},
    terrain::{hash, World, HEIGHT, SIDE},
};

const DAY_SECONDS: f32 = 960.;
const FRONT_SECONDS: f32 = 216.;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Weather {
    Clear,
    Rain,
    Thunder,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Climate {
    solar_offset: f32,
    forced_weather: Option<Weather>,
}

impl Default for Climate {
    fn default() -> Self {
        Self {
            solar_offset: 9. / 24.,
            forced_weather: None,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sky {
    pub phase: f32,
    pub daylight: f32,
    pub weather: Weather,
    pub automatic: bool,
    pub day_seconds: f32,
}

impl Climate {
    pub fn valid(&self) -> bool {
        (0.0..1.0).contains(&self.solar_offset)
    }

    pub fn set_hour(&mut self, seconds: f32, hour: u8) -> bool {
        if hour >= 24 || !seconds.is_finite() {
            return false;
        }
        let elapsed = seconds.rem_euclid(DAY_SECONDS) / DAY_SECONDS;
        self.solar_offset = (f32::from(hour) / 24. - elapsed).rem_euclid(1.).fract();
        true
    }

    pub fn set_weather(&mut self, mode: &str) -> bool {
        self.forced_weather = match mode {
            "auto" => None,
            "clear" => Some(Weather::Clear),
            "rain" => Some(Weather::Rain),
            "thunder" => Some(Weather::Thunder),
            _ => return false,
        };
        true
    }

    pub fn weather(&self, seed: u32, seconds: f32) -> Weather {
        if let Some(weather) = self.forced_weather {
            return weather;
        }
        let front = (seconds / FRONT_SECONDS) as i32;
        let passing = seconds.rem_euclid(FRONT_SECONDS);
        let timing = hash(seed, front, 181);
        let arrival = 48. + (timing & 31) as f32;
        let departure = arrival + 69. + ((timing >> 5) & 31) as f32;
        if !(arrival..departure).contains(&passing) {
            return Weather::Clear;
        }
        // One front in each pair builds thunder in the middle of its rain band.
        let storm = (front as u32 & 1) == (hash(seed, front / 2, 677) & 1);
        let thunder_start = arrival + 17. + ((timing >> 10) & 15) as f32;
        let thunder_end = departure - 14. - ((timing >> 14) & 7) as f32;
        if storm && (thunder_start..thunder_end).contains(&passing) {
            Weather::Thunder
        } else {
            Weather::Rain
        }
    }

    pub fn sky(&self, seed: u32, seconds: f32) -> Sky {
        let phase = (seconds.rem_euclid(DAY_SECONDS) / DAY_SECONDS + self.solar_offset).fract();
        let altitude = -(std::f32::consts::TAU * phase).cos();
        let twilight = ((altitude + 0.16) / 0.78).clamp(0., 1.);
        Sky {
            phase,
            daylight: twilight * twilight * (3. - 2. * twilight),
            weather: self.weather(seed, seconds),
            automatic: self.forced_weather.is_none(),
            day_seconds: DAY_SECONDS,
        }
    }
}

// Crops and visible raindrops must agree on what stops a column of rain.
fn rain_surface_in(mut block_at: impl FnMut(i32) -> u8) -> f32 {
    (0..HEIGHT)
        .rev()
        .find_map(|y| {
            let id = block_at(y);
            let item = &ITEMS[id as usize];
            let top = match id {
                AWNING | TABLE => item.height,
                _ if item.form == "liquid" => f32::from(water_height(id)) / 16.,
                _ if item.form == "prop" => 0.,
                _ => item.height,
            };
            (top > 0.).then_some(y as f32 + top)
        })
        .unwrap_or(0.)
}

pub fn rain_surface(world: &mut World, x: i32, z: i32) -> f32 {
    rain_surface_in(|y| world.get(x, y, z))
}

pub fn rain_heights(world: &mut World, cx: i32, cz: i32) -> Vec<f32> {
    let blocks = world.volume(cx, cz, 0);
    let layer = (SIDE * SIDE) as usize;
    (0..layer)
        .map(|column| rain_surface_in(|y| blocks[y as usize * layer + column]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        super::catalog::{AIR, GLASS, SAPLING, SLAB},
        *,
    };

    #[test]
    fn time_choices_and_weather_survive_a_reload_without_advancing_growth_time() {
        let mut climate = Climate::default();
        let elapsed = 73.;
        let natural = climate.weather(123, elapsed);
        assert!(climate.set_hour(elapsed, 0));
        assert!(climate.set_weather("thunder"));
        let mut restored: Climate =
            serde_json::from_str(&serde_json::to_string(&climate).expect("serialize weather"))
                .expect("restore weather");
        assert!(restored.valid());
        assert!(restored.sky(123, elapsed).daylight < 0.01);
        assert!(restored.sky(123, elapsed + DAY_SECONDS / 2.).daylight > 0.99);
        assert!(!restored.set_hour(elapsed, 24));
        assert!(!restored.set_weather("hail"));
        assert_eq!(restored.weather(123, elapsed), Weather::Thunder);
        assert!(restored.set_weather("auto"));
        assert_eq!(restored.weather(123, elapsed), natural);
    }

    #[test]
    fn natural_fronts_begin_clear_and_pass_through_rain_on_both_sides_of_thunder() {
        let climate = Climate::default();
        let fronts = |seed| {
            let mut changes = Vec::new();
            for second in 0..(2. * FRONT_SECONDS) as u32 {
                let weather = climate.weather(seed, second as f32);
                if changes.last().map(|(_, kind)| *kind) != Some(weather) {
                    changes.push((second, weather));
                }
            }
            changes
        };
        let passage = fronts(489);
        assert_ne!(passage, fronts(940));
        let kinds: Vec<_> = passage.into_iter().map(|(_, kind)| kind).collect();
        assert_eq!(kinds.first(), Some(&Weather::Clear));
        assert_eq!(kinds.last(), Some(&Weather::Clear));
        assert!(kinds
            .windows(3)
            .any(|win| win == [Weather::Rain, Weather::Thunder, Weather::Rain]));
    }

    #[test]
    fn rain_and_growth_share_the_edited_roof_at_negative_chunk_coordinates() {
        let mut world = World::new(3724);
        let (x, z) = (-1, 17);
        world.set([x, 44, z], SLAB);
        world.set([x, 45, z], SAPLING);
        let column = (z.rem_euclid(SIDE) * SIDE + x.rem_euclid(SIDE)) as usize;
        assert_eq!(rain_heights(&mut world, -1, 1)[column], 44.5);
        world.set([x, 46, z], GLASS);
        assert_eq!(rain_surface(&mut world, x, z), 47.);
        assert_eq!(rain_heights(&mut world, -1, 1)[column], 47.);
        world.set([x, 46, z], AIR);
        assert_eq!(rain_surface(&mut world, x, z), 44.5);
    }
}
