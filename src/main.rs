#![allow(unused_parens)]

use bevy::render::camera::Camera2d;
use bevy::{prelude::*, render::camera::ScalingMode, window::PresentMode};
use bevy_inspector_egui::{
    Inspectable, RegisterInspectable, WorldInspectorParams, WorldInspectorPlugin,
};
use heron::prelude::*;

pub const CLEAR: Color = Color::rgb(0.3, 0.3, 0.3);
pub const HEIGHT: f32 = 900.0;
pub const RESOLUTION: f32 = 16.0 / 9.0;

#[derive(Debug, Component, Inspectable)]
pub struct Player {
    is_grounded: bool,
}

#[derive(Debug, Component)]
pub struct Floor {}

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "Bevy Template".to_string(),
            present_mode: PresentMode::Fifo,
            resizable: false,
            ..Default::default()
        })
        .insert_resource(WorldInspectorParams {
            enabled: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::new(0.0, 0.5, 0.0)))
        .register_inspectable::<Player>()
        .add_startup_system(spawn_camera)
        .add_startup_system(spawn_environment)
        .add_startup_system(create_player)
        .add_system(toggle_inspector)
        .add_system(player_grounded_system)
        .add_system(player_input_system)
        .add_system(player_move_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    let mut camera = OrthographicCameraBundle::new_2d();

    camera.orthographic_projection.right = 1.0 * RESOLUTION;
    camera.orthographic_projection.left = -1.0 * RESOLUTION;

    camera.orthographic_projection.top = -1.0;
    camera.orthographic_projection.bottom = 1.0;

    camera.orthographic_projection.scaling_mode = ScalingMode::None;
    camera.orthographic_projection.scale = 3.0;

    commands.spawn_bundle(camera);
}

fn spawn_environment(mut commands: Commands) {
    let sprite = SpriteBundle {
        sprite: Sprite {
            color: Color::BLACK,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 1.5, 0.0),
        ..default()
    };

    commands
        .spawn_bundle(sprite)
        .insert(RigidBody::Static)
        .insert(Floor {})
        .insert(Name::new("floor"))
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(1.0, 1.0, 0.0),
            border_radius: None,
        });
}

fn create_player(mut commands: Commands) {
    let sprite = SpriteBundle {
        sprite: Sprite {
            color: Color::RED,
            ..Default::default()
        },

        ..Default::default()
    };
    commands
        .spawn_bundle(sprite)
        .insert(Player { is_grounded: false })
        .insert(Name::new("main_player"))
        .insert(RigidBody::Dynamic)
        .insert(PhysicMaterial {
            friction: 1.0,
            density: 10.0,
            ..Default::default()
        })
        .insert(RotationConstraints::lock())
        .insert(CollisionShape::Sphere { radius: 1.0 })
        .insert(Velocity::from_linear(Vec3::ZERO))
        .insert(Acceleration::from_linear(Vec3::X * 0.0));
}

fn player_grounded_system(
    mut player_query: Query<(&mut Player)>,
    floor_query: Query<(&Sprite), (With<Floor>)>,
    mut events: EventReader<CollisionEvent>,
) {
    for collision in events.iter() {
        let (ent1, ent2) = collision.rigid_body_entities();
        let floor_collided = [ent1, ent2]
            .into_iter()
            .filter_map(|ent| floor_query.get(ent).ok())
            .nth(0);
        let player_collided = [ent1, ent2]
            .into_iter()
            .filter_map(|ent| player_query.get(ent).ok())
            .nth(0);

        if floor_collided.is_some() && player_collided.is_some() {
            let mut player = player_query.single_mut();
            player.is_grounded = matches!(collision, CollisionEvent::Started(_, _));
            return;
        }
    }
}

fn player_input_system(
    mut player_query: Query<(&Player, &mut Velocity)>,
    input: Res<Input<KeyCode>>,
) {
    let (player, mut velocity) = player_query.single_mut();
    if player.is_grounded && input.just_pressed(KeyCode::Space) {
        *velocity = velocity.with_linear(-Vec3::Y);
    }
}

fn player_move_camera(
    player_query: Query<(&Transform), (With<Player>)>,
    mut camera_query: Query<(&mut Transform), (With<Camera2d>, Without<Player>)>,
) {
    let player_transform = player_query.single().translation;
    let camera = &mut camera_query.single_mut().translation;
    camera.x = player_transform.x;
    camera.y = player_transform.y;
}

fn toggle_inspector(input: Res<Input<KeyCode>>, mut window_params: ResMut<WorldInspectorParams>) {
    if input.just_pressed(KeyCode::Grave) {
        window_params.enabled = !window_params.enabled
    }
}
