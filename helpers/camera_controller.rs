use core::{f32::consts::*, fmt};

use bevy::{input::mouse::*, prelude::*, window::*};

pub(crate) struct EditorCameraPlugin;

impl Plugin for EditorCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, run_camera_controller);
    }
}

#[derive(Clone, Component, Copy, Reflect)]
#[reflect(Component)]
pub(crate) struct EditorCameraController {
    pub(crate) enabled: bool,
    pub(crate) initialized: bool,
    pub(crate) sensitivity: f32,
    pub(crate) key_forward: KeyCode,
    pub(crate) key_back: KeyCode,
    pub(crate) key_left: KeyCode,
    pub(crate) key_right: KeyCode,
    pub(crate) key_up: KeyCode,
    pub(crate) key_down: KeyCode,
    pub(crate) key_run: KeyCode,
    pub(crate) mouse_key_cursor_grab: MouseButton,
    pub(crate) keyboard_key_toggle_cursor_grab: KeyCode,
    pub(crate) walk_speed: f32,
    pub(crate) run_speed: f32,
    pub(crate) scroll_factor: f32,
    pub(crate) friction: f32,
    pub(crate) pitch: f32,
    pub(crate) yaw: f32,
    pub(crate) velocity: Vec3,
}

impl Default for EditorCameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 1.0,
            key_forward: KeyCode::KeyW,
            key_back: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_up: KeyCode::KeyE,
            key_down: KeyCode::KeyQ,
            key_run: KeyCode::ShiftLeft,
            mouse_key_cursor_grab: MouseButton::Right,
            keyboard_key_toggle_cursor_grab: KeyCode::KeyN,
            walk_speed: 2.0,
            run_speed: 4.0,
            scroll_factor: 0.1,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
        }
    }
}

impl fmt::Display for EditorCameraController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "
Freecam Controls:
    Mouse\t- Move camera orientation
    Scroll\t- Adjust movement speed
    {:?}\t- Hold to grab cursor
    {:?}\t- Toggle cursor grab
    {:?} & {:?}\t- Fly forward & backwards
    {:?} & {:?}\t- Fly sideways left & right
    {:?} & {:?}\t- Fly up & down
    {:?}\t- Fly faster while held",
            self.mouse_key_cursor_grab,
            self.keyboard_key_toggle_cursor_grab,
            self.key_forward,
            self.key_back,
            self.key_left,
            self.key_right,
            self.key_up,
            self.key_down,
            self.key_run,
        )
    }
}

/// Based on Valorant's default sensitivity, not entirely sure why it is exactly
/// 1.0 / 180.0, but I'm guessing it is a misunderstanding between
/// degrees/radians and then sticking with it because it felt nice.
const RADIANS_PER_DOT: f32 = 1.0 / 180.0;

fn run_camera_controller(
    time: Res<Time>,
    mut window_query: Query<(&Window, &mut CursorOptions)>,
    mut mouse_events: MessageReader<MouseMotion>,
    mut scroll_events: MessageReader<MouseWheel>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut toggle_cursor_grab: Local<bool>,
    mut mouse_cursor_grab: Local<bool>,
    mut query: Query<(&mut Transform, &mut EditorCameraController), With<Camera>>,
) {
    let dt = time.delta_secs();

    if let Ok((mut transform, mut controller)) = query.single_mut() {
        if !controller.initialized {
            let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
            controller.yaw = yaw;
            controller.pitch = pitch;
            controller.initialized = true;
            info!("{}", *controller);
        }
        if !controller.enabled {
            mouse_events.clear();
            return;
        }

        let mut scroll = 0.0;
        for scroll_event in scroll_events.read() {
            let amount = match scroll_event.unit {
                MouseScrollUnit::Line => scroll_event.y,
                MouseScrollUnit::Pixel => scroll_event.y / 16.0,
            };
            scroll += amount;
        }

        const MIN_WALK_SPEED: f32 = 5.0; // avoids turning it off
        const MAX_WALK_SPEED: f32 = 100.0; // avoids impossibly fast speeds

        controller.walk_speed += scroll * controller.scroll_factor * controller.walk_speed;
        controller.walk_speed = controller.walk_speed.clamp(MIN_WALK_SPEED, MAX_WALK_SPEED);

        controller.run_speed = controller.walk_speed * 3.0;

        // Handle key input
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(controller.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(controller.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(controller.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(controller.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(controller.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(controller.key_down) {
            axis_input.y -= 1.0;
        }

        let mut cursor_grab_change = false;
        if key_input.just_pressed(controller.keyboard_key_toggle_cursor_grab) {
            *toggle_cursor_grab = !*toggle_cursor_grab;
            cursor_grab_change = true;
        }
        if mouse_button_input.just_pressed(controller.mouse_key_cursor_grab) {
            *mouse_cursor_grab = true;
            cursor_grab_change = true;
        }
        if mouse_button_input.just_released(controller.mouse_key_cursor_grab) {
            *mouse_cursor_grab = false;
            cursor_grab_change = true;
        }
        let cursor_grab = *mouse_cursor_grab || *toggle_cursor_grab;

        // Apply movement update
        if axis_input != Vec3::ZERO {
            let max_speed = if key_input.pressed(controller.key_run) {
                controller.run_speed
            } else {
                controller.walk_speed
            };
            controller.velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = controller.friction.clamp(0.0, 1.0);
            controller.velocity *= 1.0 - friction;
            if controller.velocity.length_squared() < 1e-6 {
                controller.velocity = Vec3::ZERO;
            }
        }
        let forward = *transform.forward();
        let right = *transform.right();
        transform.translation += controller.velocity.x * dt * right
            + controller.velocity.y * dt * Vec3::Y
            + controller.velocity.z * dt * forward;

        // Handle cursor grab
        if cursor_grab_change {
            if cursor_grab {
                for (window, mut cursor_options) in &mut window_query {
                    if !window.focused {
                        continue;
                    }

                    cursor_options.grab_mode = CursorGrabMode::Locked;
                    cursor_options.visible = false;
                }
            } else {
                for (_, mut cursor_options) in &mut window_query {
                    cursor_options.grab_mode = CursorGrabMode::None;
                    cursor_options.visible = true;
                }
            }
        }

        // Handle mouse input
        let mut mouse_delta = Vec2::ZERO;
        if cursor_grab {
            for mouse_event in mouse_events.read() {
                mouse_delta += mouse_event.delta;
            }
        } else {
            mouse_events.clear();
        }

        if mouse_delta != Vec2::ZERO {
            // Apply look update
            controller.pitch = (controller.pitch
                - mouse_delta.y * RADIANS_PER_DOT * controller.sensitivity)
                .clamp(-PI / 2.0, PI / 2.0);
            controller.yaw -= mouse_delta.x * RADIANS_PER_DOT * controller.sensitivity;
            transform.rotation =
                Quat::from_euler(EulerRot::ZYX, 0.0, controller.yaw, controller.pitch);
        }
    }
}
