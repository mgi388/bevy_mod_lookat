#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

use bevy::{camera::primitives::Aabb, color::palettes::tailwind::*, prelude::*};
use bevy_mod_lookat::*;
use bevy_spritesheet_animation::prelude::*;
use camera_controller::{EditorCameraController, EditorCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EditorCameraPlugin)
        .add_plugins(RotateTowardsPlugin::default())
        .add_plugins(SpritesheetAnimationPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            PostUpdate,
            (draw_axes, draw_forward).after(TransformSystems::Propagate),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let camera_transform =
        Transform::from_xyz(1000.0, 100.0, 1000.0).looking_at(Vec3::ZERO, Vec3::Y);

    let mut controller = EditorCameraController {
        walk_speed: 3000.0,
        run_speed: 5000.0,
        ..Default::default()
    };

    let (yaw, pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
    controller.yaw = yaw;
    controller.pitch = pitch;

    let camera = commands
        .spawn((
            Camera3d::default(),
            Camera {
                ..Default::default()
            },
            controller,
            camera_transform,
        ))
        .id();

    // let image = assets.load("character.png");
    let image = assets.load("bevy_bird_dark.png"); // bird looks to the "right"

    // let spritesheet = Spritesheet::new(&image, 8, 8);
    let spritesheet = Spritesheet::new(&image, 1, 1);

    let atlas = spritesheet
        // .with_size_hint(768, 768)
        .with_size_hint(256, 256)
        .atlas(&mut atlas_layouts);

    commands.spawn((
        ShowAxes,
        ShowForward,
        // Note: The sprite's forward direction is an arrow pointing from the
        // center of the sprite in the direction towards the negative Z axis. If
        // you (as a human) put your eye at the end of the arrow and look at the
        // center of the sprite, the sprite appears to be looking to the "left".
        //
        // So when we have flip_vertical: false below (without my change), the
        // sprite looks to the "left" because the forward direction is being
        // "made to" point towards the camera.
        //
        // When we have flip_vertical: true below (with my change), the sprite
        // looks to the "right" because the forward direction is being "made to"
        // point away from the camera.
        //
        // So flip_vertical is also actually the wrong term here. It should be
        // something like "invert_forward_direction".
        Sprite3d::from_atlas_image(image.clone(), atlas.clone()).with_double_sided(true),
        RotateTo {
            entity: camera,
            up_direction: UpDirection::Target,
            forward_direction: ForwardDirection::Back,
        },
    ));
}

#[derive(Component)]
struct ShowAxes;

#[derive(Component)]
struct ShowForward;

fn draw_axes(mut gizmos: Gizmos, query: Query<(&GlobalTransform, &Aabb), With<ShowAxes>>) {
    for (&transform, &aabb) in &query {
        let t = transform.compute_transform();

        let length = aabb.half_extents.length();
        gizmos.axes(t, length);
    }
}
fn draw_forward(mut gizmos: Gizmos, query: Query<&GlobalTransform, With<ShowForward>>) {
    for &transform in &query {
        let t = transform.translation();

        gizmos.arrow(t, t + transform.forward() * 100.0, PINK_400);
    }

    // Also let's draw 3 arrows starting at the origin in each of the basis
    // directions
    let origin = Vec3::ZERO;
    gizmos.arrow(origin, origin + Vec3::X * 50.0, RED_400);
    gizmos.arrow(origin, origin + Vec3::Y * 50.0, GREEN_400);
    gizmos.arrow(origin, origin + Vec3::Z * 50.0, BLUE_400);
}
