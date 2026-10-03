//! High-level deterministic overworld intent for the Generated World.
//!
//! This module owns *why* a place exists before the renderer chooses source art.
//! It deliberately keeps macro geography, island/season identity and biome intent
//! separate from ElizaWy source selection so the latter can be certified sheet by
//! sheet without rewriting world topology.

pub const WORLD_PROFILE_ID: &str = "havenwild.four_season_archipelago.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IslandDescriptor {
    pub id: &'static str,
    pub label: &'static str,
    pub season: &'static str,
    pub center: [i64; 2],
    pub radius: [i64; 2],
    pub salt: u64,
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
    // Large-scale edge wobble produces authored-looking coves/headlands without
    // per-cell salt-and-pepper noise. The perturbation is intentionally bounded.
    let edge = coarse_hash(seed, world, island.salt, 5) % 1801;
    let adjusted = base - i128::from(edge as i64 - 900);
    adjusted.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
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
    // A meandering north/south river intentionally reaches both coasts. It is
    // narrow enough to create bridges/settlement crossings in later passes.
    let river = local[1].abs() <= island.radius[1] - 2
        && (local[0] - local_river_center(seed, local[1], island)).abs() <= 1;

    // One offset pond/lake per island establishes interior hydrology variety.
    let lake_center = [
        ((splitmix64(seed ^ island.salt ^ 0x4c41_4b45) % 9) as i64) - 4,
        ((splitmix64(seed ^ island.salt ^ 0x504f_4e44) % 9) as i64) - 4,
    ];
    let lx = local[0] - lake_center[0];
    let ly = local[1] - lake_center[1];
    let lake = lx * lx * 4 + ly * ly * 7 <= 72;
    river || lake
}

pub fn terrain_role(seed: u64, world: [i64; 2]) -> &'static str {
    let Some(island) = island_at(seed, world) else {
        return "RiverWater"; // ocean authority currently uses the conservative water fill
    };
    if interior_water(seed, world, island) {
        "RiverWater"
    } else {
        "Grass"
    }
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
    let score = normalized_island_score(seed, world, island);
    if score >= 7_900 {
        return "coast";
    }
    let zone = coarse_hash(seed, world, island.salt ^ 0x4249_4f4d_45, 7) % 100;
    if zone < 42 {
        "dense_forest"
    } else if zone < 63 {
        "light_woodland"
    } else {
        "meadow"
    }
}

pub fn tree_density_percent(seed: u64, world: [i64; 2]) -> u64 {
    match biome(seed, world) {
        "dense_forest" => 78,
        "light_woodland" => 46,
        "meadow" => 16,
        _ => 0,
    }
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
                "Grass" | "RiverWater"
            ));
        }
    }

    #[test]
    fn center_channels_remain_ocean_separated() {
        assert_eq!(region_id(7, [0, 0]), "open_ocean");
        assert_eq!(terrain_role(7, [0, 0]), "RiverWater");
    }
}
