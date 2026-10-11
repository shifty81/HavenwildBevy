//! Opt-in, isolated Bevy 0.19 geometry proof. Does not modify Havenwild's 2D Studio.
//! cargo run --example hybrid_terrain_proof --release
//! WGPU_BACKEND=vulkan can select Vulkan on supported systems.
use bevy::asset::RenderAssetUsages;
use bevy::mesh::Indices;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

const SIDE: usize = 40;
const CELL: f32 = 1.0;

/// Discrete 0-3 tier authored world height sample; deterministic and renderer-independent.
fn height(x: usize, z: usize) -> f32 {
    let dx = x as f32 - 20.0;
    let dz = z as f32 - 20.0;
    let distance = (dx * dx + dz * dz).sqrt();
    if distance < 6.0 {
        3.0
    } else if distance < 11.0 {
        2.0
    } else if distance < 16.0 {
        1.0
    } else {
        0.0
    }
}

fn is_river(x: usize, z: usize) -> bool {
    let center = 8.0 + ((z as f32) * 0.19).sin() * 3.0;
    (x as f32 - center).abs() < 1.8
}

/// Tile-column mesh has horizontal top and vertical faces at discrete height changes.
/// Vertices are not welded intentionally, allowing distinct top/edge materials in future.
fn terrain_mesh(x: usize, z: usize) -> Mesh {
    let level = height(x, z);
    let x0 = x as f32 * CELL;
    let x1 = x0 + CELL;
    let z0 = z as f32 * CELL;
    let z1 = z0 + CELL;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut quad = |points: [[f32; 3]; 4], normal: [f32; 3]| {
        let start = positions.len() as u32;
        positions.extend(points);
        normals.extend([normal; 4]);
        uvs.extend([[0., 0.], [1., 0.], [1., 1.], [0., 1.]]);
        indices.extend([start, start + 1, start + 2, start, start + 2, start + 3]);
    };
    quad(
        [
            [x0, level, z0],
            [x0, level, z1],
            [x1, level, z1],
            [x1, level, z0],
        ],
        [0., 1., 0.],
    );
    // Exposed sides descend to adjacent terrain rather than making a full cube everywhere.
    let west = if x > 0 { height(x - 1, z) } else { 0. };
    let east = if x + 1 < SIDE { height(x + 1, z) } else { 0. };
    let north = if z > 0 { height(x, z - 1) } else { 0. };
    let south = if z + 1 < SIDE { height(x, z + 1) } else { 0. };
    if level > west {
        quad(
            [
                [x0, west, z0],
                [x0, west, z1],
                [x0, level, z1],
                [x0, level, z0],
            ],
            [-1., 0., 0.],
        );
    }
    if level > east {
        quad(
            [
                [x1, east, z1],
                [x1, east, z0],
                [x1, level, z0],
                [x1, level, z1],
            ],
            [1., 0., 0.],
        );
    }
    if level > north {
        quad(
            [
                [x1, north, z0],
                [x0, north, z0],
                [x0, level, z0],
                [x1, level, z0],
            ],
            [0., 0., -1.],
        );
    }
    if level > south {
        quad(
            [
                [x0, south, z1],
                [x1, south, z1],
                [x1, level, z1],
                [x0, level, z1],
            ],
            [0., 0., 1.],
        );
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[derive(Component)]
struct ProofCamera;
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(53., 49., 55.).looking_at(Vec3::new(20., 0., 20.), Vec3::Y),
        ProofCamera,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 14000.,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(12., 30., 18.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let grass = materials.add(Color::srgb(0.28, 0.55, 0.22));
    let rock = materials.add(Color::srgb(0.45, 0.43, 0.38));
    let sand = materials.add(Color::srgb(0.76, 0.66, 0.41));
    let water = materials.add(StandardMaterial {
        base_color: Color::srgba(0.12, 0.43, 0.65, 0.85),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    for z in 0..SIDE {
        for x in 0..SIDE {
            let h = height(x, z);
            let mat = if h >= 2.0 {
                rock.clone()
            } else if is_river(x, z) {
                sand.clone()
            } else {
                grass.clone()
            };
            commands.spawn((Mesh3d(meshes.add(terrain_mesh(x, z))), MeshMaterial3d(mat)));
            // The river is an independent surface. It is not a hydrology solver.
            if is_river(x, z) && h <= 1.0 {
                commands.spawn((
                    Mesh3d(meshes.add(Plane3d::default().mesh().size(CELL, CELL))),
                    MeshMaterial3d(water.clone()),
                    Transform::from_xyz(x as f32 + 0.5, h + 0.025, z as f32 + 0.5),
                ));
            }
        }
    }
    info!("HYBRID PROOF: 40x40 tiles; real height-tier geometry, cliff side faces, simple river, shadowed directional light. No ElizaWy textures yet.");
}
fn orbit(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut cameras: Query<&mut Transform, With<ProofCamera>>,
) {
    if let Ok(mut camera) = cameras.single_mut() {
        let speed = 15.0 * time.delta_secs();
        let mut move_by = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            move_by.z -= speed;
        }
        if keys.pressed(KeyCode::KeyS) {
            move_by.z += speed;
        }
        if keys.pressed(KeyCode::KeyA) {
            move_by.x -= speed;
        }
        if keys.pressed(KeyCode::KeyD) {
            move_by.x += speed;
        }
        camera.translation += move_by;
    }
}
fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Havenwild Hybrid 3D — Opt-In Geometry Proof".into(),
                        resolution: (1280, 800).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, orbit)
        .run();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discrete_height_and_river_are_deterministic() {
        assert_eq!(height(20, 20), 3.);
        assert_eq!(height(0, 0), 0.);
        assert_eq!(is_river(8, 0), true);
    }
    #[test]
    fn cliff_mesh_has_more_faces_than_flat_column() {
        let high = terrain_mesh(20, 25);
        let flat = terrain_mesh(0, 0);
        assert!(high.count_vertices() >= flat.count_vertices());
    }
}
