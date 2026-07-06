use bevy::ecs::{
    message::{MessageWriter, MessageRegistry, Message, MessageReader},
    prelude::{Res, ResMut, Resource},
};
use bevy::color::palettes::css::WHITE;
use bevy::prelude::*;
use bevy_wasm_modding::prelude::*;
use cubes_protocol::{HostMessage, ModMessage, PROTOCOL_VERSION};

#[derive(Component)]
struct Movable;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WasmPlugin::<HostMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_systems(Startup, insert_mods)
        .add_systems(Startup, setup)
        .add_systems(Update, update_cubes_from_mods)
        .run();
}

fn insert_mods(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(WasmMod {
        wasm: asset_server.load("mod_with_bevy.wasm"),
    });
    commands.spawn(WasmMod {
        wasm: asset_server.load("mod_without_bevy.wasm"),
    });
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    //plane
    /*
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(5.0, 5.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3),
            perceptual_roughness: 0.5,
            ..default()
        })),
    ));
    */
    // Ambient light
    commands.insert_resource(GlobalAmbientLight {
        color: WHITE.into(),
        brightness: 400.0,
        ..default()
    });
    // Point light
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadow_maps_enabled: true,
            affects_lightmapped_mesh_diffuse: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    /*
    commands.spawn(PointLight {
        intensity: 1500.0,
        shadows_enabled: true,
        affects_lightmapped_mesh_diffuse: false,
        ..default()
    });
    Transform::from_xyz(4.0, 8.0, 4.0);
    */
    // Camera
    /*    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 3.5, 5.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        ..default()
    ));
    */
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 3.5, 5.0)
            .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
}

fn update_cubes_from_mods(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mod_messages: MessageReader<ModMessage>,
    mut host_messages: MessageWriter<HostMessage>,
    mut query: Query<&mut Transform, With<Movable>>,
) {
    for event in mod_messages.read() {
        match event {
            ModMessage::MoveCube { entity_id, x, y, z } => {
                if let Some(entity) = Entity::from_raw_u32(*entity_id) {
                    if let Ok(mut transform) = query.get_mut(entity) {
                        transform.translation = Vec3::new(*x, *y, *z);
                    }
                }
            }
            ModMessage::SpawnCube { mod_state, color } => {
                info!("Spawning cube from mod {:x}!", mod_state);

                let entity = commands
                    .spawn((
                        Mesh3d(meshes.add(Cuboid::default())),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(color.0, color.1, color.2),
                            ..default()
                        })),
                        Transform::from_xyz(0.0, 0.5, 0.0),
                        Movable,
                        Name::new(format!("mod_cube_{mod_state}")),
                    ))
                    .id();

                let entity_id = entity.index_u32();
                info!(
                    "HOST -> {:?}",
                    HostMessage::SpawnedCube {
                        mod_state: *mod_state,
                        entity_id,
                    }
                );
                host_messages.write(HostMessage::SpawnedCube {
                    mod_state: *mod_state,
                    entity_id,
                });
            }
        }
    }
}
