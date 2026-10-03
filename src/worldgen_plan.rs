//! High-level deterministic overworld intent for the Generated World.
//!
//! M2D090 deliberately pushes beyond the old four-green-discs prototype.  This
//! module owns the *purpose graph* first: seasonal islands, elevation terraces,
//! hydrology, settlement anchors, roads, crossings, waterfall drops and cave
//! approach points.  Rendering still chooses immutable ElizaWy source pixels.

pub const WORLD_PROFILE_ID: &str = "havenwild.four_season_archipelago.v2";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IslandDescriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub season: &'static str,
    pub center: [i64; 2],
    pub radius: [i64; 2],
    pub salt: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SettlementDescriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub class: &'static str,
    pub center: [i64; 2],
    pub island_id: &'static str,
}

pub const ISLANDS: [IslandDescriptor; 4] = [
    IslandDescriptor {
        id: "spring_isle",
        label: "Spring Isle",
        season: "spring",
        center: [-32, -32],
        radius: [25, 22],
        salt: 0x5350_5249_4e47,
    },
    IslandDescriptor {
        id: "summer_isle",
        label: "Summer Isle",
        season: "summer",
        center: [32, -32],
        radius: [27, 23],
        salt: 0x5355_4d4d_4552,
    },
    IslandDescriptor {
        id: "autumn_isle",
        label: "Autumn Isle",
        season: "autumn",
        center: [-32, 32],
        radius: [26, 22],
        salt: 0x4155_5455_4d4e,
    },
    IslandDescriptor {
        id: "winter_isle",
        label: "Winter Isle",
        season: "winter",
        center: [32, 32],
        radius: [25, 23],
        salt: 0x5749_4e54_4552,
    },
];

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn coord_hash(seed: u64, world: [i64; 2], salt: u64) -> u64 {
    splitmix64(
        seed ^ (world[0] as u64).wrapping_mul(0x632b_e59b_d9b4_e019)
            ^ (world[1] as u64).wrapping_mul(0x8cb9_2baa_3f3d_8dd7)
            ^ salt,
    )
}

fn coarse_hash(seed: u64, world: [i64; 2], salt: u64, scale: i64) -> u64 {
    coord_hash(
        seed,
        [world[0].div_euclid(scale), world[1].div_euclid(scale)],
        salt,
    )
}

fn normalized_island_score(seed: u64, world: [i64; 2], island: IslandDescriptor) -> i64 {
    let dx = i128::from(world[0]) - i128::from(island.center[0]);
    let dy = i128::from(world[1]) - i128::from(island.center[1]);
    let rx = i128::from(island.radius[0]);
    let ry = i128::from(island.radius[1]);
    let base = dx * dx * 10_000 / (rx * rx) + dy * dy * 10_000 / (ry * ry);
    // Large-scale edge wobble produces coves/headlands without per-cell noise.
    let edge = coarse_hash(seed, world, island.salt, 5) % 1801;
    let adjusted = base - i128::from(edge as i64 - 900);
    adjusted.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

fn nearest_island(seed: u64, world: [i64; 2]) -> (i64, IslandDescriptor) {
    ISLANDS
        .into_iter()
        .map(|island| (normalized_island_score(seed, world, island), island))
        .min_by_key(|(score, _)| *score)
        .expect("four-season archipelago always has islands")
}

pub fn island_at(seed: u64, world: [i64; 2]) -> Option<IslandDescriptor> {
    ISLANDS
        .into_iter()
        .filter_map(|island| {
            let score = normalized_island_score(seed, world, island);
            (score <= 10_000).then_some((score, island))
        })
        .min_by_key(|(score, _)| *score)
        .map(|(_, island)| island)
}

fn local_river_center(seed: u64, local_y: i64, island: IslandDescriptor) -> i64 {
    let phase = (local_y + (splitmix64(seed ^ island.salt) % 47) as i64).rem_euclid(47);
    let triangle = if phase < 24 { phase } else { 46 - phase };
    let lateral = triangle / 4 - 3;
    let bias = (splitmix64(seed ^ island.salt ^ 0x5249_5645_52) % 7) as i64 - 3;
    lateral + bias
}

fn interior_water(seed: u64, world: [i64; 2], island: IslandDescriptor) -> bool {
    let local = [world[0] - island.center[0], world[1] - island.center[1]];
    let river = local[1].abs() <= island.radius[1] - 2
        && (local[0] - local_river_center(seed, local[1], island)).abs() <= 1;

    let lake_center = [
        ((splitmix64(seed ^ island.salt ^ 0x4c41_4b45) % 9) as i64) - 4,
        ((splitmix64(seed ^ island.salt ^ 0x504f_4e44) % 9) as i64) + 5,
    ];
    let lx = local[0] - lake_center[0];
    let ly = local[1] - lake_center[1];
    let lake = lx * lx * 4 + ly * ly * 7 <= 72;
    river || lake
}

fn adjacent_to_interior_water(seed: u64, world: [i64; 2], island: IslandDescriptor) -> bool {
    [(0i64, -1i64), (1, 0), (0, 1), (-1, 0)]
        .into_iter()
        .any(|(dx, dy)| interior_water(seed, [world[0] + dx, world[1] + dy], island))
}

fn road_center_y(seed: u64, local_x: i64, island: IslandDescriptor) -> i64 {
    let phase =
        (local_x + (splitmix64(seed ^ island.salt ^ 0x524f_4144) % 53) as i64).rem_euclid(53);
    let triangle = if phase < 27 { phase } else { 52 - phase };
    triangle / 7 - 2
}

fn line_distance_chebyshev(point: [i64; 2], a: [i64; 2], b: [i64; 2]) -> i64 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let steps = dx.abs().max(dy.abs()).max(1);
    let mut best = i64::MAX;
    for step in 0..=steps {
        let x = a[0] + dx * step / steps;
        let y = a[1] + dy * step / steps;
        best = best.min((point[0] - x).abs().max((point[1] - y).abs()));
    }
    best
}

pub fn settlement_centers(seed: u64, island: IslandDescriptor) -> [SettlementDescriptor; 2] {
    let west_x = -13;
    let east_x = 12;
    let west_y = road_center_y(seed, west_x, island);
    let east_y = road_center_y(seed, east_x, island);
    [
        SettlementDescriptor {
            id: match island.id {
                "spring_isle" => "spring_hamlet",
                "summer_isle" => "summer_hamlet",
                "autumn_isle" => "autumn_hamlet",
                _ => "winter_hamlet",
            },
            label: "West Hamlet",
            class: "hamlet",
            center: [island.center[0] + west_x, island.center[1] + west_y],
            island_id: island.id,
        },
        SettlementDescriptor {
            id: match island.id {
                "spring_isle" => "spring_town",
                "summer_isle" => "summer_town",
                "autumn_isle" => "autumn_town",
                _ => "winter_town",
            },
            label: "East Town",
            class: "town",
            center: [island.center[0] + east_x, island.center[1] + east_y],
            island_id: island.id,
        },
    ]
}

pub fn settlement_at(seed: u64, world: [i64; 2]) -> Option<SettlementDescriptor> {
    let island = island_at(seed, world)?;
    settlement_centers(seed, island)
        .into_iter()
        .find(|settlement| {
            (world[0] - settlement.center[0]).abs()
                <= if settlement.class == "town" { 6 } else { 4 }
                && (world[1] - settlement.center[1]).abs()
                    <= if settlement.class == "town" { 5 } else { 4 }
        })
}

pub fn waterfall_anchor(seed: u64, island: IslandDescriptor) -> [i64; 2] {
    let local_y = -6;
    [
        island.center[0] + local_river_center(seed, local_y, island) - 1,
        island.center[1] + local_y + 1,
    ]
}

pub fn cave_anchor(seed: u64, island: IslandDescriptor) -> [i64; 2] {
    let side = if splitmix64(seed ^ island.salt ^ 0x4341_5645) & 1 == 0 {
        -1
    } else {
        1
    };
    [island.center[0] + side * 14, island.center[1] - 5]
}

pub fn bridge_anchor(seed: u64, island: IslandDescriptor) -> [i64; 2] {
    // Main road crosses the river close to the island centre. Search a compact
    // deterministic window so the placement follows the actual meander.
    let mut best = island.center;
    let mut best_score = i64::MAX;
    for lx in -6..=6 {
        let ly = road_center_y(seed, lx, island);
        let world = [island.center[0] + lx, island.center[1] + ly];
        let river_x = local_river_center(seed, ly, island);
        let score = (lx - river_x).abs();
        if score < best_score {
            best_score = score;
            best = world;
        }
    }
    [best[0] - 1, best[1] - 1]
}

fn country_road(seed: u64, world: [i64; 2], island: IslandDescriptor) -> bool {
    let local = [world[0] - island.center[0], world[1] - island.center[1]];
    if local[0].abs() > island.radius[0] - 4 || local[1].abs() > island.radius[1] - 4 {
        return false;
    }
    let main = (local[1] - road_center_y(seed, local[0], island)).abs() <= 1;
    let town = settlement_centers(seed, island)[1].center;
    let cave = cave_anchor(seed, island);
    let spur = line_distance_chebyshev(world, town, cave) <= 1;
    main || spur
}

pub fn elevation(seed: u64, world: [i64; 2]) -> i16 {
    let Some(island) = island_at(seed, world) else {
        return 0;
    };
    let local = [world[0] - island.center[0], world[1] - island.center[1]];
    let score = normalized_island_score(seed, world, island);
    if score >= 7_700 {
        return 0;
    }
    // A broad north terrace creates a real +1 cliff line crossed by the river.
    // A smaller inner ridge creates occasional +2 geography without turning the
    // whole island into staircase noise.
    if local[1] <= -13 && score < 4_900 {
        2
    } else if local[1] <= -6 && score < 7_200 {
        1
    } else {
        0
    }
}

pub fn terrain_role(seed: u64, world: [i64; 2]) -> &'static str {
    let (score, nearest) = nearest_island(seed, world);
    if score > 10_000 {
        return if score <= 12_300 {
            "RiverWater"
        } else {
            "DeepWater"
        };
    }
    let island = nearest;
    if interior_water(seed, world, island) {
        return "RiverWater";
    }
    if adjacent_to_interior_water(seed, world, island) {
        return "MudBank";
    }
    if score >= 9_250 {
        return "WetSand";
    }
    if score >= 8_250 {
        return "Sand";
    }
    if country_road(seed, world, island) {
        return "PebblePath";
    }
    // Settlement yards deliberately stay clear and readable; surrounding biomes
    // supply the denser vegetation rather than burying buildings in trees.
    if settlement_at(seed, world).is_some() {
        return "Grass";
    }
    "Grass"
}

pub fn season(seed: u64, world: [i64; 2]) -> &'static str {
    island_at(seed, world).map_or("ocean", |island| island.season)
}

pub fn region_id(seed: u64, world: [i64; 2]) -> &'static str {
    island_at(seed, world).map_or("open_ocean", |island| island.id)
}

pub fn biome(seed: u64, world: [i64; 2]) -> &'static str {
    let Some(island) = island_at(seed, world) else {
        return "ocean";
    };
    if interior_water(seed, world, island) {
        return "river_or_lake";
    }
    if settlement_at(seed, world).is_some() {
        return "settlement";
    }
    if country_road(seed, world, island) {
        return "road_corridor";
    }
    let score = normalized_island_score(seed, world, island);
    if score >= 7_900 {
        return "coast";
    }
    let h = elevation(seed, world);
    if h >= 2 {
        return "rocky_upland";
    }
    let zone = coarse_hash(seed, world, island.salt ^ 0x4249_4f4d_45, 7) % 100;
    if zone < 46 {
        "dense_forest"
    } else if zone < 70 {
        "light_woodland"
    } else {
        "meadow"
    }
}

pub fn tree_density_percent(seed: u64, world: [i64; 2]) -> u64 {
    match biome(seed, world) {
        "dense_forest" => 88,
        "light_woodland" => 62,
        "meadow" => 20,
        "rocky_upland" => 24,
        _ => 0,
    }
}

pub fn is_cave_approach(seed: u64, world: [i64; 2]) -> bool {
    island_at(seed, world).is_some_and(|island| {
        let cave = cave_anchor(seed, island);
        (world[0] - cave[0]).abs() <= 1 && (world[1] - cave[1]).abs() <= 1
    })
}

pub fn is_waterfall_site(seed: u64, world: [i64; 2]) -> bool {
    island_at(seed, world).is_some_and(|island| {
        let anchor = waterfall_anchor(seed, island);
        world == anchor
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_season_island_centers_are_land() {
        for island in ISLANDS {
            assert_eq!(island_at(7, island.center), Some(island));
            assert_eq!(season(7, island.center), island.season);
            assert!(matches!(
                terrain_role(7, island.center),
                "Grass" | "MudBank" | "RiverWater" | "PebblePath"
            ));
        }
    }

    #[test]
    fn center_channels_remain_ocean_separated() {
        assert_eq!(region_id(7, [0, 0]), "open_ocean");
        assert!(matches!(
            terrain_role(7, [0, 0]),
            "RiverWater" | "DeepWater"
        ));
    }

    #[test]
    fn islands_have_purpose_graph_and_elevation_features() {
        for island in ISLANDS {
            let settlements = settlement_centers(7, island);
            assert_eq!(settlements.len(), 2);
            assert!(settlements
                .iter()
                .all(|s| island_at(7, s.center) == Some(island)));
            let bridge = bridge_anchor(7, island);
            assert!(island_at(7, bridge).is_some());
            let fall = waterfall_anchor(7, island);
            assert!(island_at(7, fall).is_some());
            assert!(elevation(7, [island.center[0], island.center[1] - 10]) >= 1);
        }
    }
}
