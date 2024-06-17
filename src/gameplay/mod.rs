use std::ops::Add;
use std::time::Duration;
use avian3d::collision::Collider;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::PlayerCamera;

pub struct GamePlayPlugin;

impl Plugin for GamePlayPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<TweenSize>()
            .add_systems(Update, (
                tween_size_system,
                spawn_cubes_system,
                shoot_bullet_system,
            ))
        ;
    }
}

#[derive(Component, Default, Reflect)]
pub struct TweenSize {
    pub start_size: Vec3,
    pub end_size: Vec3,
    pub duration: Duration,
    elapsed: Duration,
}


pub fn tween_size_system(
    time: Res<Time>,
    mut tween_query: Query<(&mut Transform, &mut TweenSize)>,
) {
    for (mut transform, mut tween_size) in tween_query.iter_mut() {
        let progress = (tween_size.elapsed.as_secs_f32() / tween_size.duration.as_secs_f32()).clamp(0.0, 1.0);

        transform.scale = tween_size.start_size.lerp(tween_size.end_size, progress);
        tween_size.elapsed = tween_size.elapsed.add(time.delta());
    }
}


pub fn spawn_cubes_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    player_camera_query: Query<&mut Transform, With<PlayerCamera>>,
) {
    if !mouse_button.just_pressed(MouseButton::Left) { return; }

    let player_transform = player_camera_query.single();
    let position = player_transform.translation + player_transform.forward().as_vec3() * 2.0;
    let start_size = Vec3::splat(0.01);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_translation(position).with_scale(start_size),
            ..default()
        },
        Collider::cuboid(1.0, 1.0, 1.0),
        RigidBody::Dynamic,
        TweenSize {
            start_size,
            end_size: Vec3::splat(1.0),
            duration: Duration::from_secs_f32(0.50),
            ..default()
        },
        LinearVelocity(Vec3::Y)
    ));
}

pub fn shoot_bullet_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
) {
    if !mouse_button.just_pressed(MouseButton::Right) { return; }
    let window = q_window.single();
    let center = Vec2::new(window.width() * 0.5, window.height() * 0.5);
    let (camera, camera_transform) = q_camera.single();
    let Some(ray) = camera.viewport_to_world(camera_transform, center) else {
        return;
    };

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Sphere::new(0.1)),
            material: materials.add(StandardMaterial::from_color(Color::srgb(1.0, 0.3, 0.0))),
            transform: Transform::from_translation(ray.origin),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::sphere(0.050),
        LinearVelocity(ray.direction * 100.0),
        Mass(0.1),
        GravityScale(0.1),
    ));
}