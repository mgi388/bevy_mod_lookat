#[path = "../helpers/camera_controller.rs"]
mod camera_controller;

use bevy::{
    camera::primitives::Aabb, color::palettes::tailwind::*, light::CascadeShadowConfigBuilder,
    prelude::*,
};
use bevy_mod_lookat::*;
use camera_controller::{EditorCameraController, EditorCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(EditorCameraPlugin)
        .add_plugins(RotateTowardsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(
            PostUpdate,
            (draw_axes, draw_forward).after(TransformSystems::Propagate),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera_transform =
        Transform::from_xyz(0.7, 0.7, 1.0).looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y);

    let mut controller = EditorCameraController {
        walk_speed: 1.0,
        run_speed: 2.0,
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
            EnvironmentMapLight {
                diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
                specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
                intensity: 250.0,
                ..default()
            },
        ))
        .id();

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
    ));
    commands.spawn((
        ShowAxes,
        ShowForward,
        SceneRoot(
            asset_server
                // Note: The helmet's forward direction is an arrow pointing
                // from the center of the helmet model in the direction towards
                // the back of the helmet model. This is pointing in the
                // direction of the negative Z axis.
                //
                // So when we have flip_vertical: false below (without my
                // change), the front of the helmet faces away from the camera
                // because the forward direction is being "made to" point
                // towards the camera.
                //
                // When we have flip_vertical: true below (with my change), the
                // front of the helmet faces the camera because the forward
                // direction is being "made to" point away from the camera.
                //
                // So flip_vertical is also actually the wrong term here. It
                // should be something like "invert_forward_direction".
                .load(GltfAssetLabel::Scene(0).from_asset("models/FlightHelmet/FlightHelmet.gltf")),
        ),
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

        gizmos.arrow(t, t + transform.forward() * 10.0, PINK_400);
    }

    // Also let's draw 3 arrows starting at the origin in each of the basis
    // directions
    let origin = Vec3::ZERO;
    gizmos.arrow(origin, origin + Vec3::X * 5.0, RED_400);
    gizmos.arrow(origin, origin + Vec3::Y * 5.0, GREEN_400);
    gizmos.arrow(origin, origin + Vec3::Z * 5.0, BLUE_400);
}
