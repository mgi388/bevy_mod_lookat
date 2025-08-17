use bevy::prelude::*;

#[derive(Clone, Component, Debug, Reflect)]
#[reflect(Component, Debug)]
/// When this component is added on an entity, [`Transform::forward()`] direction points towards the selected
/// entity
pub struct RotateTo {
    /// entity to target, the Targeted entity must have a [`GlobalTransform`]
    pub entity: Entity,
    /// The rotated entity will match its [`Transform::up()`] according to this
    pub updir: UpDirection,
    /// Whether to flip the object along the vertical axis (180-degree rotation around the up direction)
    pub flip_vertical: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Debug, PartialEq)]
/// The rotated entity will try to have its [`Transform::up()`] direction matching this selection
pub enum UpDirection {
    /// Will synchronize the direction of UP towards the UP direction of the target
    /// Useful when rotating towards the camera and wanting the direction to be up for example
    Target,
    /// Keeps the up-direction the same as for the parent of this entity
    /// useful when you want it rotated in relation to what this entity is attached to
    /// Note: if there is no parent, the up direction will fallback to be Vec3::Y
    Parent,
    /// Keeps a static direction of UP set to this value
    /// useful when you want to decide what is up for the entity under rotation
    Dir(Dir3),
}

/// Set enum for the systems relating to rotation towards a target.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum RotateTowardsSystems {
    /// System that rotates entities towards its target.
    ApplyRotation,
}

/// Plugin that constantly rotates entities towards a selected target when they have the [`RotateTo`] component on them
/// if you only want the math for calculating the local rotation needed to look at a target, see [`calculate_local_rotation_to_target`]
pub struct RotateTowardsPlugin;

impl Plugin for RotateTowardsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<RotateTo>();

        app.add_systems(
            PostUpdate,
            (rotate_towards, update_global_transforms)
                .chain()
                .in_set(RotateTowardsSystems::ApplyRotation)
                .after(TransformSystem::TransformPropagate),
        );
    }
}

fn rotate_towards(
    global_transforms: Query<&GlobalTransform>,
    mut rotators: Query<(&GlobalTransform, &mut Transform, Option<&Parent>, &RotateTo)>,
) {
    for (rotator_gt, mut rotator_t, parent, target) in rotators.iter_mut() {
        let Ok(target_gt) = global_transforms.get(target.entity) else {
            bevy::log::error!("Entity used as target was not found: {}", target.entity);
            continue;
        };

        let parent_gt = if let Some(parent_e) = parent {
            global_transforms.get(parent_e.get()).ok()
        } else {
            None
        };

        let updir = match target.updir {
            UpDirection::Target => target_gt.up(),
            UpDirection::Dir(dir) => dir,
            UpDirection::Parent => {
                if let Some(parent_gt) = parent_gt {
                    parent_gt.up()
                } else {
                    // if there is no parent, fallback to bevy up direction
                    Dir3::Y
                }
            }
        };

        let rotation = calculate_local_rotation_to_target(
            rotator_gt,
            target_gt,
            parent_gt,
            updir,
            target.flip_vertical,
        );

        const EPSILON: f32 = 1e-6;
        if !rotation.abs_diff_eq(rotator_t.rotation, EPSILON) {
            rotator_t.rotation = rotation;
        }
    }
}

fn update_global_transforms(
    transform_helper: TransformHelper,
    mut query: Query<(Entity, &mut GlobalTransform), (With<RotateTo>, Changed<Transform>)>,
) {
    for (entity, mut global_transform) in query.iter_mut() {
        // Update the global transform to match the new rotation.
        let gt = transform_helper
            .compute_global_transform(entity)
            .expect("Failed to compute global transform");
        *global_transform = gt;
    }
}

/// Calculates the local rotation on a rotator towards a target, adjusting for rotations of eventual parents, with the selected rotator up direction.
pub fn calculate_local_rotation_to_target(
    rotator_gt: &GlobalTransform,
    target_gt: &GlobalTransform,
    parent_gt: Option<&GlobalTransform>,
    updir: Dir3,
    flip_vertical: bool,
) -> Quat {
    let target_gt_computed = target_gt.compute_transform();
    let parent_gt_computed: Option<Transform> = parent_gt.map(|p| p.compute_transform());

    let mut rotation = rotator_gt
        .compute_transform()
        .looking_at(target_gt_computed.translation, updir)
        .rotation;

    if flip_vertical {
        // Apply a 180-degree rotation around the up direction to flip the object vertically.
        rotation = Quat::from_axis_angle(updir.normalize(), std::f32::consts::PI) * rotation;
    }

    if let Some(parent_gt_computed) = parent_gt_computed {
        rotation = parent_gt_computed.rotation.inverse() * rotation;
    }
    rotation
}
