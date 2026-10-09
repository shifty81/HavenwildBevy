//! High-level deterministic overworld intent for the Generated World.
//!
//! M2D090 deliberately pushes beyond the old four-green-discs prototype.  This
//! module owns the *purpose graph* first: seasonal islands, elevation terraces,
//! hydrology, settlement anchors, roads, crossings, waterfall drops and cave
//! approach points.  Rendering still chooses immutable ElizaWy source pixels.

pub const WORLD_PROFILE_ID: &str = "havenwild.four_season_archipelago.v3";

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaterContext {
    None,
    Ocean,
    Coast,
    River,
    PondLake,
}

impl WaterContext {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Ocean => "ocean",
            Self::Coast => "coast",
            Self::River => "river",
            Self::PondLake => "pond_lake",
        }
    }

    pub fn supports_lily_pads(self) -> bool {
        matches!(self, Self::River | Self::PondLake)
    }

    pub fn lily_pad_density_percent(self) -> u64 {
        match self {
            Self::PondLake => 58,
            Self::River => 14,
            Self::None | Self::Ocean | Self::Coast => 0,
        }
    }
}

// Shared structural cliff topology. A set bit means the current cell is the
// higher side of a drop toward that cardinal neighbor. This is intentionally
// source-art agnostic: editor rendering, runtime collision/navigation and the
// later cliff sprite resolver consume the same elevation-derived mask.
pub const CLIFF_NORTH: u8 = 0b0001;
pub const CLIFF_EAST: u8 = 0b0010;
pub const CLIFF_SOUTH: u8 = 0b0100;
pub const CLIFF_WEST: u8 = 0b1000;

pub fn cliff_mask(seed: u64, world: [i64; 2]) -> u8 {
    let here = elevation(seed, world);
    if here <= 0 {
        return 0;
    }
    let mut mask = 0u8;
    for (bit, delta) in [
        (CLIFF_NORTH, [0, -1]),
        (CLIFF_EAST, [1, 0]),
        (CLIFF_SOUTH, [0, 1]),
        (CLIFF_WEST, [-1, 0]),
    ] {
        let neighbor = [world[0] + delta[0], world[1] + delta[1]];
        if here > elevation(seed, neighbor) {
            mask |= bit;
        }
    }
    mask
}

pub fn cliff_drop(seed: u64, world: [i64; 2], delta: [i64; 2]) -> i16 {
    (elevation(seed, world) - elevation(seed, [world[0] + delta[0], world[1] + delta[1]])).max(0)
}

pub const ISLANDS: [IslandDescriptor; 4] = [
    IslandDescriptor {
        id: "spring_isle",
        label: "Spring Isle",
        season: "spring",
        center: [-32, -32],
        radius: [30, 20],
        salt: 0x5350_5249_4e47,
    },
    IslandDescriptor {
        id: "summer_isle",
        label: "Summer Isle",
        season: "summer",
        center: [32, -32],
        radius: [23, 28],
        salt: 0x5355_4d4d_4552,
    },
    IslandDescriptor {
        id: "autumn_isle",
        label: "Autumn Isle",
        season: "autumn",
        center: [-32, 32],
        radius: [29, 22],
        salt: 0x4155_5455_4d4e,
    },
    IslandDescriptor {
        id: "winter_isle",
        label: "Winter Isle",
        season: "winter",
        center: [32, 32],
        radius: [24, 27],
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

fn smoothstep_fixed(offset: i64, scale: i64) -> i64 {
    debug_assert!(scale > 0);
    const ONE: i64 = 1024;
    let t = (offset.clamp(0, scale) * ONE) / scale;
    let t2 = (t * t) / ONE;
    (t2 * (3 * ONE - 2 * t)) / ONE
}

fn lerp_fixed(a: i64, b: i64, weight: i64) -> i64 {
    const ONE: i64 = 1024;
    (a * (ONE - weight) + b * weight) / ONE
}

fn smooth_coarse_noise(seed: u64, world: [i64; 2], salt: u64, scale: i64) -> i64 {
    debug_assert!(scale > 0);
    let lattice = [world[0].div_euclid(scale), world[1].div_euclid(scale)];
    let offset = [world[0].rem_euclid(scale), world[1].rem_euclid(scale)];
    let sample = |dx: i64, dy: i64| -> i64 {
        (coord_hash(seed, [lattice[0] + dx, lattice[1] + dy], salt) % 1601) as i64 - 800
    };
    let tx = smoothstep_fixed(offset[0], scale);
    let ty = smoothstep_fixed(offset[1], scale);
    let north = lerp_fixed(sample(0, 0), sample(1, 0), tx);
    let south = lerp_fixed(sample(0, 1), sample(1, 1), tx);
    lerp_fixed(north, south, ty)
}

fn normalized_island_score(seed: u64, world: [i64; 2], island: IslandDescriptor) -> i64 {
    let dx = i128::from(world[0]) - i128::from(island.center[0]);
    let dy = i128::from(world[1]) - i128::from(island.center[1]);
    let rx = i128::from(island.radius[0]);
    let ry = i128::from(island.radius[1]);
    let mut base = dx * dx * 10_000 / (rx * rx) + dy * dy * 10_000 / (ry * ry);

    // Each seasonal island owns a different macro silhouette rather than sharing
    // one rounded-disc recipe with only a palette swap. These broad deterministic
    // modifiers create coves/headlands/lobes without introducing per-cell noise.
    match island.id {
        "spring_isle" => {
            if dx < -(rx / 3) {
                base -= 650;
            }
            if dx > rx / 4 && dy > 0 {
                base += 700;
            }
        }
        "summer_isle" => {
            if dy.abs() < ry / 3 {
                base -= 420;
            }
            if dx > rx / 3 && dy < -(ry / 4) {
                base += 520;
            }
        }
        "autumn_isle" => {
            if dx < 0 {
                base -= 380;
            }
            if dx > rx / 4 && dy > ry / 5 {
                base += 620;
            }
        }
        "winter_isle" => {
            if dy < -(ry / 4) {
                base -= 340;
            }
            if dx.abs() < rx / 5 && dy < -(ry / 2) {
                base += 820;
            }
        }
        _ => {}
    }

    // Large-scale edge wobble produces coves/headlands without exposing the
    // square 5x5 hash cells as visible sand/coast blocks.  Interpolating the same
    // deterministic lattice keeps the silhouette organic while preserving seed
    // stability and broad seasonal shape differences.
    let edge = smooth_coarse_noise(seed, world, island.salt, 5);
    let adjusted = base - i128::from(edge);
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

fn local_stream_center_y(seed: u64, local_x: i64, island: IslandDescriptor) -> i64 {
    let phase =
        (local_x + (splitmix64(seed ^ island.salt ^ 0x5354_5245_414d) % 43) as i64).rem_euclid(43);
    let triangle = if phase < 22 { phase } else { 42 - phase };
    let bias = (splitmix64(seed ^ island.salt ^ 0x5354_5245_414d_59) % 7) as i64 - 3;
    triangle / 5 - 2 + bias
}

fn interior_water_context(seed: u64, world: [i64; 2], island: IslandDescriptor) -> WaterContext {
    let local = [world[0] - island.center[0], world[1] - island.center[1]];
    match island.id {
        // Spring is meadow/pond country. The cross-island creek intentionally
        // reaches the generated coastline; terrain_role() clips the corridor to
        // the actual irregular island silhouette, producing real sea mouths.
        "spring_isle" => {
            let creek = (local[1] - local_stream_center_y(seed, local[0], island)).abs() <= 1;
            let p1 = (local[0] + 8).pow(2) * 5 + (local[1] - 5).pow(2) * 7 <= 70;
            let p2 = (local[0] - 10).pow(2) * 6 + (local[1] + 5).pow(2) * 8 <= 58;
            if creek {
                WaterContext::River
            } else if p1 || p2 {
                WaterContext::PondLake
            } else {
                WaterContext::None
            }
        }
        // Summer's primary north/south river is a true coast-to-coast channel,
        // not a capped pond. The irregular island mask determines both mouths.
        "summer_isle" => {
            let river = (local[0] - local_river_center(seed, local[1], island)).abs() <= 1;
            let lx = local[0] + 3;
            let ly = local[1] - 7;
            let lake = lx * lx * 4 + ly * ly * 7 <= 72;
            if river {
                WaterContext::River
            } else if lake {
                WaterContext::PondLake
            } else {
                WaterContext::None
            }
        }
        // Autumn also owns a sea-connected river feeding its broad lower lake.
        "autumn_isle" => {
            let river = (local[0] - local_river_center(seed, local[1], island) + 5).abs() <= 1;
            let lx = local[0] - 5;
            let ly = local[1] - 7;
            let lake = lx * lx * 3 + ly * ly * 6 <= 90;
            if river {
                WaterContext::River
            } else if lake {
                WaterContext::PondLake
            } else {
                WaterContext::None
            }
        }
        // Winter deliberately remains lake-heavy with a short inland feeder.
        _ => {
            let river = local[1] >= -island.radius[1] + 3
                && local[1] <= 6
                && (local[0] - local_river_center(seed, local[1], island) - 2).abs() <= 1;
            let lx = local[0] + 2;
            let ly = local[1] - 4;
            let lake = lx * lx * 4 + ly * ly * 4 <= 110;
            if river {
                WaterContext::River
            } else if lake {
                WaterContext::PondLake
            } else {
                WaterContext::None
            }
        }
    }
}

fn interior_water(seed: u64, world: [i64; 2], island: IslandDescriptor) -> bool {
    !matches!(
        interior_water_context(seed, world, island),
        WaterContext::None
    )
}

pub fn water_context(seed: u64, world: [i64; 2]) -> WaterContext {
    if let Some(island) = island_at(seed, world) {
        let interior = interior_water_context(seed, world, island);
        if !matches!(interior, WaterContext::None) {
            return interior;
        }
        return WaterContext::None;
    }
    let (score, _) = nearest_island(seed, world);
    if score <= 11_100 {
        WaterContext::Coast
    } else {
        WaterContext::Ocean
    }
}

fn adjacent_to_interior_water(seed: u64, world: [i64; 2], island: IslandDescriptor) -> bool {
    [(0i64, -1i64), (1, 0), (0, 1), (-1, 0)]
        .into_iter()
        .any(|(dx, dy)| interior_water(seed, [world[0] + dx, world[1] + dy], island))
}

fn adjacent_interior_water_context(
    seed: u64,
    world: [i64; 2],
    island: IslandDescriptor,
) -> WaterContext {
    let mut saw_pond = false;
    for (dx, dy) in [(0i64, -1i64), (1, 0), (0, 1), (-1, 0)] {
        match interior_water_context(seed, [world[0] + dx, world[1] + dy], island) {
            WaterContext::River => return WaterContext::River,
            WaterContext::PondLake => saw_pond = true,
            _ => {}
        }
    }
    if saw_pond {
        WaterContext::PondLake
    } else {
        WaterContext::None
    }
}

fn bank_erosion_candidate(
    seed: u64,
    world: [i64; 2],
    island: IslandDescriptor,
    dirt_percent: u64,
) -> bool {
    // Bank dirt is a continuous erosion patch, not an independent tile lottery.
    // The previous coarse-hash threshold could leave a single MudBank cell inside
    // an otherwise grass-backed pond edge, which exposed a square one-tile notch.
    let noise = smooth_coarse_noise(seed, world, island.salt ^ 0x4241_4e4b, 7);
    let normalized = (noise + 800).clamp(0, 1600) as u64;
    normalized < dirt_percent.saturating_mul(16)
}

fn clustered_bank_erosion(
    seed: u64,
    world: [i64; 2],
    island: IslandDescriptor,
    dirt_percent: u64,
) -> bool {
    if !bank_erosion_candidate(seed, world, island, dirt_percent) {
        return false;
    }
    // Require at least one neighbouring land-bank candidate. This deliberately
    // removes isolated one-cell dirt hooks while retaining longer authored-bank
    // stretches around rivers and ponds.
    [(0i64, -1i64), (1, 0), (0, 1), (-1, 0)]
        .into_iter()
        .map(|(dx, dy)| [world[0] + dx, world[1] + dy])
        .any(|neighbour| {
            island_at(seed, neighbour) == Some(island)
                && normalized_island_score(seed, neighbour, island) < 9_050
                && !interior_water(seed, neighbour, island)
                && adjacent_to_interior_water(seed, neighbour, island)
                && bank_erosion_candidate(seed, neighbour, island, dirt_percent)
        })
}

fn road_center_y(seed: u64, local_x: i64, island: IslandDescriptor) -> i64 {
    let phase =
        (local_x + (splitmix64(seed ^ island.salt ^ 0x524f_4144) % 53) as i64).rem_euclid(53);
    let triangle = if phase < 27 { phase } else { 52 - phase };
    match island.id {
        "spring_isle" => triangle / 10 - 1,
        "summer_isle" => triangle / 8 - 2,
        "autumn_isle" => triangle / 6 + 1,
        _ => triangle / 12 - 3 + local_x.div_euclid(10),
    }
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
    let (west_x, east_x, west_class, east_class, west_label, east_label) = match island.id {
        "spring_isle" => (
            -15,
            9,
            "hamlet",
            "village",
            "Orchard Hamlet",
            "Meadow Village",
        ),
        "summer_isle" => (-13, 12, "hamlet", "town", "West Hamlet", "River Town"),
        "autumn_isle" => (-16, 10, "camp", "village", "Timber Camp", "Harvest Village"),
        _ => (
            -11,
            11,
            "outpost",
            "village",
            "Frost Outpost",
            "Winter Village",
        ),
    };
    let west_y = road_center_y(seed, west_x, island);
    let east_y = road_center_y(seed, east_x, island);
    [
        SettlementDescriptor {
            id: match island.id {
                "spring_isle" => "spring_orchard_hamlet",
                "summer_isle" => "summer_hamlet",
                "autumn_isle" => "autumn_timber_camp",
                _ => "winter_outpost",
            },
            label: west_label,
            class: west_class,
            center: [island.center[0] + west_x, island.center[1] + west_y],
            island_id: island.id,
        },
        SettlementDescriptor {
            id: match island.id {
                "spring_isle" => "spring_meadow_village",
                "summer_isle" => "summer_river_town",
                "autumn_isle" => "autumn_harvest_village",
                _ => "winter_village",
            },
            label: east_label,
            class: east_class,
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
                <= if matches!(settlement.class, "town" | "village") {
                    6
                } else {
                    4
                }
                && (world[1] - settlement.center[1]).abs()
                    <= if matches!(settlement.class, "town" | "village") {
                        5
                    } else {
                        4
                    }
        })
}

pub fn waterfall_anchor(seed: u64, island: IslandDescriptor) -> Option<[i64; 2]> {
    // The reviewed Waterfall.png recipe is south-facing. Do not stamp the same
    // waterfall onto every seasonal island or onto an east/west creek. Search for
    // an actual north->south elevation drop on a vertical watercourse instead.
    if island.id == "spring_isle" {
        return None;
    }
    for ly in (-(island.radius[1] - 4))..=(island.radius[1] - 5) {
        let lx = match island.id {
            "autumn_isle" => local_river_center(seed, ly, island) - 5,
            "winter_isle" => local_river_center(seed, ly, island) + 2,
            _ => local_river_center(seed, ly, island),
        };
        let upper = [island.center[0] + lx, island.center[1] + ly];
        let lower = [upper[0], upper[1] + 1];
        if interior_water(seed, upper, island)
            && interior_water(seed, lower, island)
            && elevation(seed, upper) > elevation(seed, lower)
        {
            // Top-left anchor for the reviewed 3x4 south-facing body/splash crop.
            return Some([upper[0] - 1, upper[1] - 2]);
        }
    }
    None
}

pub fn cave_anchor(seed: u64, island: IslandDescriptor) -> [i64; 2] {
    let side = if splitmix64(seed ^ island.salt ^ 0x4341_5645) & 1 == 0 {
        -1
    } else {
        1
    };
    [island.center[0] + side * 14, island.center[1] - 5]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BridgeSpan {
    /// Top-left world anchor. The first and last span cells are land approaches.
    pub anchor: [i64; 2],
    /// Horizontal span in 32px cells, including one land approach on each side.
    pub span_cells: u32,
}

pub fn bridge_span(seed: u64, island: IslandDescriptor) -> Option<BridgeSpan> {
    // Resolve the actual horizontal width of the interior-water run where the main
    // road crosses it. A bridge is not a fixed 3x2 sprite: the left/right source
    // ends sit one cell onto land and the middle source column repeats across water.
    let mut best: Option<(BridgeSpan, i64)> = None;
    for lx in -(island.radius[0] - 4)..=(island.radius[0] - 4) {
        let ly = road_center_y(seed, lx, island);
        let crossing = [island.center[0] + lx, island.center[1] + ly];
        if !interior_water(seed, crossing, island) {
            continue;
        }
        let mut left = crossing[0];
        while interior_water(seed, [left - 1, crossing[1]], island) {
            left -= 1;
        }
        let mut right = crossing[0];
        while interior_water(seed, [right + 1, crossing[1]], island) {
            right += 1;
        }
        let left_bank = [left - 1, crossing[1]];
        let right_bank = [right + 1, crossing[1]];
        if island_at(seed, left_bank) != Some(island)
            || island_at(seed, right_bank) != Some(island)
            || interior_water(seed, left_bank, island)
            || interior_water(seed, right_bank, island)
        {
            continue;
        }
        let water_cells = right - left + 1;
        let span_cells = u32::try_from(water_cells + 2).ok()?;
        if !(3..=16).contains(&span_cells) {
            continue;
        }
        let span = BridgeSpan {
            anchor: [left_bank[0], crossing[1] - 1],
            span_cells,
        };
        let score = lx.abs() + ly.abs() + water_cells;
        if best.is_none_or(|(_, best_score)| score < best_score) {
            best = Some((span, score));
        }
    }
    best.map(|(span, _)| span)
}

pub fn bridge_anchor(seed: u64, island: IslandDescriptor) -> Option<[i64; 2]> {
    bridge_span(seed, island).map(|span| span.anchor)
}

fn country_road(seed: u64, world: [i64; 2], island: IslandDescriptor) -> bool {
    let local = [world[0] - island.center[0], world[1] - island.center[1]];
    if local[0].abs() > island.radius[0] - 4 || local[1].abs() > island.radius[1] - 4 {
        return false;
    }
    // Keep roads one semantic cell wide. The old three-cell corridor amplified
    // every transition defect and read as a paved plaza across the entire island.
    let main = local[1] == road_center_y(seed, local[0], island);
    // Cave spurs stay disabled until continuous cliff faces own the cave entrance.
    // A road should not terminate at a floating standalone cliff/cave sprite.
    main
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
    match island.id {
        "spring_isle" => {
            if local[0] <= -14 && score < 5_500 {
                1
            } else {
                0
            }
        }
        "summer_isle" => {
            if local[1] <= -14 && score < 4_900 {
                2
            } else if local[1] <= -6 && score < 7_200 {
                1
            } else {
                0
            }
        }
        "autumn_isle" => {
            if local[0] <= -15 && local[1] <= -5 && score < 5_000 {
                2
            } else if local[0] <= -8 && score < 7_100 {
                1
            } else {
                0
            }
        }
        _ => {
            if local[1] <= -13 && score < 5_400 {
                2
            } else if local[1] <= -5 && score < 7_000 {
                1
            } else {
                0
            }
        }
    }
}

pub fn terrain_role(seed: u64, world: [i64; 2]) -> &'static str {
    let (score, nearest) = nearest_island(seed, world);
    if score > 10_000 {
        return if score <= 11_100 {
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
        // Where an inland channel reaches the ocean, its shoulders join the
        // authored Sand↔Shallow-Water topology instead of carrying a grass/mud
        // bank into the surf. Legacy WetSand is no longer generated because its
        // identity relative to Jaidyn's Deep Sand remains evidence-gated.
        if score >= 9_050 {
            return "Sand";
        }
        // Elsewhere ElizaWy has both grass-bank and dirt-bank water families. Do
        // not force a brown mud ring around every river/lake. Eroded dirt remains
        // a smooth contiguous patch; isolated one-cell dirt hooks are rejected.
        let base_dirt_percent = match island.season {
            "spring" => 16,
            "summer" => 30,
            "autumn" => 38,
            _ => 12,
        };
        let dirt_percent = match adjacent_interior_water_context(seed, world, island) {
            // Calm pond/lake edges stay predominantly grassy; tiny brown hooks
            // read like broken path tiles and do not communicate calm water.
            WaterContext::PondLake => (base_dirt_percent / 3).max(4),
            WaterContext::River => base_dirt_percent,
            _ => 0,
        };
        return if dirt_percent > 0 && clustered_bank_erosion(seed, world, island, dirt_percent) {
            "MudBank"
        } else {
            "Grass"
        };
    }
    // Ocean shore now uses the configured core Sand↔Shallow-Water topology as
    // one authority. The previous generated WetSand ring and pale-shore repaint
    // are retired until Deep Sand/shore animation evidence proves a replacement.
    if score >= 9_050 {
        return "Sand";
    }
    if country_road(seed, world, island) {
        // Generated roads use the warm dirt fill seen in the ElizaWy demo scene.
        // PebblePath remains available as a manual/POI material instead of being
        // stretched across every settlement as the primary road surface.
        return "MudBank";
    }
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
    if score >= 8_850 {
        return "coast";
    }
    let h = elevation(seed, world);
    if h >= 2 {
        return "rocky_upland";
    }
    let zone = coarse_hash(seed, world, island.salt ^ 0x4249_4f4d_45, 7) % 100;
    match island.season {
        "spring" => {
            if zone < 28 {
                "dense_forest"
            } else if zone < 53 {
                "light_woodland"
            } else {
                "meadow"
            }
        }
        "summer" => {
            if zone < 52 {
                "dense_forest"
            } else if zone < 78 {
                "light_woodland"
            } else {
                "meadow"
            }
        }
        "autumn" => {
            if zone < 42 {
                "dense_forest"
            } else if zone < 74 {
                "light_woodland"
            } else {
                "meadow"
            }
        }
        _ => {
            if zone < 24 {
                "dense_forest"
            } else if zone < 48 {
                "light_woodland"
            } else {
                "meadow"
            }
        }
    }
}

pub fn tree_density_percent(seed: u64, world: [i64; 2]) -> u64 {
    let seasonal_scale = match season(seed, world) {
        "spring" => 80,
        "summer" => 100,
        "autumn" => 88,
        "winter" => 58,
        _ => 0,
    };
    let base = match biome(seed, world) {
        "dense_forest" => 84,
        "light_woodland" => 56,
        "meadow" => 14,
        "rocky_upland" => 18,
        _ => 0,
    };
    base * seasonal_scale / 100
}

pub fn is_cave_approach(seed: u64, world: [i64; 2]) -> bool {
    island_at(seed, world).is_some_and(|island| {
        let cave = cave_anchor(seed, island);
        (world[0] - cave[0]).abs() <= 1 && (world[1] - cave[1]).abs() <= 1
    })
}

pub fn is_waterfall_site(seed: u64, world: [i64; 2]) -> bool {
    island_at(seed, world).is_some_and(|island| {
        waterfall_anchor(seed, island).is_some_and(|anchor| {
            world[0] >= anchor[0]
                && world[0] < anchor[0] + 3
                && world[1] >= anchor[1]
                && world[1] < anchor[1] + 4
        })
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
        let mut bridge_count = 0;
        let mut waterfall_count = 0;
        for island in ISLANDS {
            let settlements = settlement_centers(7, island);
            assert_eq!(settlements.len(), 2);
            assert!(settlements
                .iter()
                .all(|s| island_at(7, s.center) == Some(island)));
            if let Some(span) = bridge_span(7, island) {
                bridge_count += 1;
                assert!((3..=16).contains(&span.span_cells));
                let road_y = span.anchor[1] + 1;
                assert_eq!(island_at(7, [span.anchor[0], road_y]), Some(island));
                assert_eq!(
                    island_at(7, [span.anchor[0] + i64::from(span.span_cells) - 1, road_y]),
                    Some(island)
                );
                for x in 1..span.span_cells - 1 {
                    assert!(interior_water(
                        7,
                        [span.anchor[0] + i64::from(x), road_y],
                        island
                    ));
                }
            }
            if let Some(fall) = waterfall_anchor(7, island) {
                waterfall_count += 1;
                assert!(island_at(7, [fall[0] + 1, fall[1] + 2]).is_some());
            }
            let has_elevation = (-18..=18).any(|ly| {
                (-18..=18)
                    .any(|lx| elevation(7, [island.center[0] + lx, island.center[1] + ly]) > 0)
            });
            assert!(has_elevation);
        }
        assert!(
            bridge_count >= 2,
            "at least two islands should have valid reviewed bridge crossings"
        );
        assert!(
            (1..=3).contains(&waterfall_count),
            "waterfalls should be contextual, not cloned onto all four islands"
        );
        assert_eq!(
            waterfall_anchor(7, ISLANDS[0]),
            None,
            "Spring creek should not receive the south-facing waterfall prefab"
        );
    }

    #[test]
    fn cliff_masks_are_derived_from_cardinal_elevation_drops() {
        let seed = 0x4841_5645_4e57_494c;
        let mut found = 0usize;
        for island in ISLANDS {
            for y in (island.center[1] - island.radius[1])..=(island.center[1] + island.radius[1]) {
                for x in
                    (island.center[0] - island.radius[0])..=(island.center[0] + island.radius[0])
                {
                    let world = [x, y];
                    let mask = cliff_mask(seed, world);
                    if mask == 0 {
                        continue;
                    }
                    found += 1;
                    let here = elevation(seed, world);
                    for (bit, delta) in [
                        (CLIFF_NORTH, [0, -1]),
                        (CLIFF_EAST, [1, 0]),
                        (CLIFF_SOUTH, [0, 1]),
                        (CLIFF_WEST, [-1, 0]),
                    ] {
                        let neighbor = elevation(seed, [world[0] + delta[0], world[1] + delta[1]]);
                        assert_eq!(mask & bit != 0, here > neighbor);
                    }
                }
            }
        }
        assert!(
            found > 0,
            "generated archipelago must expose real cliff edges"
        );
    }

    #[test]
    fn ocean_cells_never_own_cliff_faces() {
        let seed = 0x4841_5645_4e57_494c;
        for world in [[0, 0], [0, -64], [0, 64], [-64, 0], [96, 96]] {
            if island_at(seed, world).is_none() {
                assert_eq!(cliff_mask(seed, world), 0);
            }
        }
    }

    #[test]
    fn island_edge_noise_is_smooth_across_lattice_boundaries() {
        let seed = 0x4841_5645_4e57_494c;
        for island in ISLANDS {
            let mut previous = smooth_coarse_noise(
                seed,
                [island.center[0] - 30, island.center[1]],
                island.salt,
                5,
            );
            for x in (island.center[0] - 29)..=(island.center[0] + 30) {
                let current = smooth_coarse_noise(seed, [x, island.center[1]], island.salt, 5);
                assert!(
                    (current - previous).abs() <= 520,
                    "interpolated island-edge noise jumped by {} at x={x}",
                    (current - previous).abs()
                );
                previous = current;
            }
        }
    }

    #[test]
    fn intended_rivers_open_into_the_ocean() {
        let seed = 5_206_537_499_884_079_436u64;
        for island in ISLANDS
            .into_iter()
            .filter(|island| island.id != "winter_isle")
        {
            let mut mouth_cells = 0usize;
            let x0 = island.center[0] - island.radius[0] - 6;
            let x1 = island.center[0] + island.radius[0] + 6;
            let y0 = island.center[1] - island.radius[1] - 6;
            let y1 = island.center[1] + island.radius[1] + 6;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let world = [x, y];
                    if island_at(seed, world) != Some(island)
                        || terrain_role(seed, world) != "RiverWater"
                    {
                        continue;
                    }
                    if [(0i64, -1i64), (1, 0), (0, 1), (-1, 0)]
                        .into_iter()
                        .any(|(dx, dy)| island_at(seed, [x + dx, y + dy]).is_none())
                    {
                        mouth_cells += 1;
                    }
                }
            }
            assert!(
                mouth_cells > 0,
                "island {} has no sea-connected river mouth",
                island.id
            );
        }
    }

    #[test]
    fn coastal_river_shoulders_use_beach_materials() {
        let seed = 5_206_537_499_884_079_436u64;
        let island = ISLANDS
            .into_iter()
            .find(|island| island.id == "summer_isle")
            .unwrap();
        let mut saw_shore = false;
        for y in island.center[1] - island.radius[1] - 3..=island.center[1] + island.radius[1] + 3 {
            for x in
                island.center[0] - island.radius[0] - 3..=island.center[0] + island.radius[0] + 3
            {
                let world = [x, y];
                let Some(found) = island_at(seed, world) else {
                    continue;
                };
                if found != island
                    || interior_water(seed, world, island)
                    || !adjacent_to_interior_water(seed, world, island)
                {
                    continue;
                }
                let score = normalized_island_score(seed, world, island);
                if score >= 9_050 {
                    saw_shore = true;
                    assert_eq!(terrain_role(seed, world), "Sand");
                }
            }
        }
        assert!(
            saw_shore,
            "summer river mouth produced no coastal shoulder sample"
        );
    }

    #[test]
    fn generated_coast_uses_core_sand_without_legacy_wetsand_ring() {
        for seed in [7_u64, 5_206_537_499_884_079_436_u64] {
            for island in ISLANDS {
                let mut sand = 0usize;
                let mut wet = 0usize;
                let margin = 3;
                for y in (island.center[1] - island.radius[1] - margin)
                    ..=(island.center[1] + island.radius[1] + margin)
                {
                    for x in (island.center[0] - island.radius[0] - margin)
                        ..=(island.center[0] + island.radius[0] + margin)
                    {
                        match terrain_role(seed, [x, y]) {
                            "Sand" => sand += 1,
                            "WetSand" => wet += 1,
                            _ => {}
                        }
                    }
                }
                assert!(sand > 0, "island {} has no generated Sand coast", island.id);
                assert_eq!(
                    wet, 0,
                    "island {} still generates the legacy WetSand coast ring",
                    island.id
                );
            }
        }
    }

    #[test]
    fn generated_inland_dirt_banks_do_not_form_single_cell_islands() {
        for seed in [7_u64, 5_206_537_499_884_079_436_u64] {
            for island in ISLANDS {
                for y in
                    (island.center[1] - island.radius[1])..=(island.center[1] + island.radius[1])
                {
                    for x in (island.center[0] - island.radius[0])
                        ..=(island.center[0] + island.radius[0])
                    {
                        let world = [x, y];
                        if terrain_role(seed, world) != "MudBank"
                            || !adjacent_to_interior_water(seed, world, island)
                        {
                            continue;
                        }
                        let neighbours = [(0_i64, -1_i64), (1, 0), (0, 1), (-1, 0)]
                            .into_iter()
                            .filter(|(dx, dy)| {
                                let neighbour = [world[0] + dx, world[1] + dy];
                                terrain_role(seed, neighbour) == "MudBank"
                                    && adjacent_to_interior_water(seed, neighbour, island)
                            })
                            .count();
                        assert!(
                            neighbours > 0,
                            "isolated generated MudBank shoreline cell at {world:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn seasonal_islands_have_distinct_hydrology_signatures() {
        let signatures = ISLANDS.map(|island| {
            let mut water = 0usize;
            let mut road = 0usize;
            for ly in -18..=18 {
                for lx in -18..=18 {
                    let world = [island.center[0] + lx, island.center[1] + ly];
                    water += interior_water(7, world, island) as usize;
                    road += country_road(7, world, island) as usize;
                }
            }
            (water, road, island.radius)
        });
        for a in 0..signatures.len() {
            for b in (a + 1)..signatures.len() {
                assert_ne!(signatures[a], signatures[b]);
            }
        }
    }
    #[test]
    fn lily_pad_water_context_never_allows_ocean_or_coast() {
        assert_eq!(WaterContext::Ocean.lily_pad_density_percent(), 0);
        assert_eq!(WaterContext::Coast.lily_pad_density_percent(), 0);
        assert!(!WaterContext::Ocean.supports_lily_pads());
        assert!(!WaterContext::Coast.supports_lily_pads());
        assert!(WaterContext::PondLake.supports_lily_pads());
        assert!(WaterContext::River.supports_lily_pads());
        assert!(
            WaterContext::PondLake.lily_pad_density_percent()
                > WaterContext::River.lily_pad_density_percent()
        );
    }

    #[test]
    fn summer_hydrology_distinguishes_river_lake_coast_and_ocean() {
        let seed = 5_206_537_499_884_079_436u64;
        let island = ISLANDS
            .into_iter()
            .find(|island| island.id == "summer_isle")
            .unwrap();
        let mut saw_river = false;
        let mut saw_lake = false;
        let mut saw_coast = false;
        let mut saw_ocean = false;
        for y in island.center[1] - island.radius[1] - 8..=island.center[1] + island.radius[1] + 8 {
            for x in
                island.center[0] - island.radius[0] - 8..=island.center[0] + island.radius[0] + 8
            {
                match water_context(seed, [x, y]) {
                    WaterContext::River => saw_river = true,
                    WaterContext::PondLake => saw_lake = true,
                    WaterContext::Coast => saw_coast = true,
                    WaterContext::Ocean => saw_ocean = true,
                    WaterContext::None => {}
                }
            }
        }
        assert!(saw_river && saw_lake && saw_coast && saw_ocean);
    }
}
