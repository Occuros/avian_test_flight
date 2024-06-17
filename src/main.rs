//! A simple 3D scene with light shining over a cube sitting on a plane.

mod gameplay;

use std::f32::consts::TAU;
use avian3d::PhysicsPlugins;
use avian3d::prelude::{Collider, PhysicsDebugPlugin, RigidBody};
use bevy::input::*;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::prelude::KeyCode::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use crate::gameplay::GamePlayPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            PhysicsDebugPlugin::default(),
            GamePlayPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(
            PreUpdate,
            (
                capture_input_system,
                move_camera_system,
            )
                .chain()
                .after(mouse::mouse_button_input_system)
                .after(keyboard::keyboard_input_system)
                .after(gamepad::gamepad_axis_event_system)
                .after(gamepad::gamepad_button_event_system)
                .after(gamepad::gamepad_connection_system)
                .after(gamepad::gamepad_event_system)
                .after(touch::touch_screen_input_system),
        )
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::default());

    //ground
    commands.spawn((
        PbrBundle {
            mesh: cube_mesh.clone(),
            material: materials.add(Color::srgb(0.7, 0.7, 0.8)),
            transform: Transform::from_xyz(0.0, -2.0, 0.0).with_scale(Vec3::new(100.0, 1.0, 100.0)),
            ..default()
        },
        RigidBody::Static,
        Collider::cuboid(1.0, 1.0, 1.0),
    ));

    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        Collider::cuboid(1.0, 1.0, 1.0),
        RigidBody::Dynamic,
    ));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-TAU / 8.),
            ..default()
        },
        // The default cascade config is designed to handle large scenes.
        // As this example has a much smaller world, we can tighten the shadow
        // bounds for better visual quality.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
            .into(),
        ..default()
    });

    // commands.insert_resource(AmbientLight {
    //     color: Color::WHITE,
    //     brightness: FULL_DAYLIGHT,
    // });
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 1.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PlayerCamera,
        PlayerInput::default(),
    ));
}

const ANGLE_EPSILON: f32 = 0.001953125;

pub fn capture_input_system(
    key_input: Res<ButtonInput<KeyCode>>,
    mut mouse_events: EventReader<MouseMotion>,
    mut query: Query<&mut PlayerInput>,
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let mut primary_window = q_windows.single_mut();

    if key_input.just_pressed(KeyL) || key_input.just_pressed(Escape) {
        if primary_window.cursor.grab_mode == CursorGrabMode::Locked {
            primary_window.cursor.grab_mode = CursorGrabMode::None;
            primary_window.cursor.visible = true;
        } else {
            primary_window.cursor.grab_mode = CursorGrabMode::Locked;
            primary_window.cursor.visible = false;
        }
    }

    for mut input in query.iter_mut() {
        let mut mouse_delta = Vec2::ZERO;
        for mouse_event in mouse_events.read() {
            mouse_delta += mouse_event.delta;
        }
        mouse_delta *= 0.003;

        input.pitch = (input.pitch - mouse_delta.y)
            .clamp(-TAU * 0.25 + ANGLE_EPSILON, TAU * 0.25 - ANGLE_EPSILON);
        input.yaw -= mouse_delta.x;
        if input.yaw.abs() > TAU * 0.5 {
            input.yaw = input.yaw.rem_euclid(TAU);
        }

        input.movement = Vec3::new(
            get_axis(&key_input, KeyD, KeyA),
            get_axis(&key_input, KeyE, KeyQ),
            get_axis(&key_input, KeyW, KeyS),
        );
    }
}

fn get_pressed(key_input: &Res<ButtonInput<KeyCode>>, key: KeyCode) -> f32 {
    if key_input.pressed(key) {
        1.0
    } else {
        0.0
    }
}

fn get_axis(key_input: &Res<ButtonInput<KeyCode>>, key_pos: KeyCode, key_neg: KeyCode) -> f32 {
    get_pressed(key_input, key_pos) - get_pressed(key_input, key_neg)
}


#[derive(Component, Default)]
pub struct PlayerInput {
    pub pitch: f32,
    pub yaw: f32,
    pub movement: Vec3,
}

#[derive(Component, Default)]
pub struct PlayerCamera;

pub fn move_camera_system(
    time: Res<Time>,
    input_query: Query<&PlayerInput>,
    mut player_camera_query: Query<&mut Transform, With<PlayerCamera>>,
) {
    let Ok(input) = input_query.get_single() else { return };
    let Ok(mut camera_transform) = player_camera_query.get_single_mut() else { return };
    camera_transform.rotation =
        Quat::from_euler(EulerRot::YXZ, input.yaw, input.pitch, 0.0);


    let mut movement = camera_transform.forward() * input.movement.z
        + camera_transform.right() * input.movement.x;

    let sensitivity_factor = time.delta_seconds() * 10.0;
    movement.y = input.movement.y * sensitivity_factor;
    movement *= sensitivity_factor;

    camera_transform.translation += movement;
}