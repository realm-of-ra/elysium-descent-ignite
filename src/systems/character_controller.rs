use avian3d::{math::*, prelude::*};
use bevy::{ecs::query::Has, prelude::*};
use crate::{rendering::cameras::player_camera::FlyCam, Player};

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastInputDirection>()
            .add_event::<MovementAction>()
            .add_systems(
                Update,
                (
                    keyboard_input,
                    gamepad_input,
                    update_grounded,
                    movement,
                    apply_movement_damping,
                    camera_follow_player_system,
                )
                    .chain(),
            );
    }
}

/// An event sent for a movement input action.
#[derive(Event)]
pub enum MovementAction {
    Move(Vector2),
    Jump,
}

/// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

/// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
/// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

/// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

/// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(Scalar);

/// A bundle that contains the components needed for a basic
/// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    movement: MovementBundle,
}

/// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
}

impl MovementBundle {
    pub const fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDampingFactor(damping),
            jump_impulse: JumpImpulse(jump_impulse),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(112.5, 0.9, 7.0)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(
                caster_shape,
                Vector::ZERO,
                Quaternion::default(),
                Dir3::NEG_Y,
            )
            .with_max_distance(0.2),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(acceleration, damping, jump_impulse);
        self
    }
}

/// Add a resource to store the last movement input for camera rotation
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct LastInputDirection(pub Vec2);

/// Sends [`MovementAction`] events based on keyboard input.
fn keyboard_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut last_input: ResMut<LastInputDirection>,
) {
    let up = keyboard_input.any_pressed([KeyCode::KeyW, KeyCode::ArrowUp]);
    let down = keyboard_input.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]);
    let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
    let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;
    let direction = Vector2::new(horizontal as Scalar, vertical as Scalar).clamp_length_max(1.0);

    if direction != Vector2::ZERO {
        movement_event_writer.write(MovementAction::Move(direction));
        last_input.0 = direction.as_dvec2().as_vec2();
    }

    if keyboard_input.just_pressed(KeyCode::Space) {
        movement_event_writer.write(MovementAction::Jump);
    }
}

/// Sends [`MovementAction`] events based on gamepad input.
fn gamepad_input(
    mut movement_event_writer: EventWriter<MovementAction>,
    gamepads: Query<&Gamepad>,
) {
    for gamepad in gamepads.iter() {
        if let (Some(x), Some(y)) = (
            gamepad.get(GamepadAxis::LeftStickX),
            gamepad.get(GamepadAxis::LeftStickY),
        ) {
            movement_event_writer.write(MovementAction::Move(
                Vector2::new(x as Scalar, y as Scalar).clamp_length_max(1.0),
            ));
        }

        if gamepad.just_pressed(GamepadButton::South) {
            movement_event_writer.write(MovementAction::Jump);
        }
    }
}

/// Updates the [`Grounded`] status for character controllers.
fn update_grounded(
    mut commands: Commands,
    mut query: Query<(Entity, &LinearVelocity), With<CharacterController>>,
) {
    for (entity, velocity) in &mut query {
        // Consider grounded only when velocity is very close to zero
        let is_grounded = velocity.y.abs() < 0.1; // Check if we're actually on the ground

        if is_grounded {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

/// Responds to [`MovementAction`] events and moves character controllers accordingly.
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<(
        &MovementAcceleration,
        &JumpImpulse,
        &mut LinearVelocity,
        &mut Transform,
        Has<Grounded>,
    )>,
) {
    let delta_time = time.delta_secs_f64().adjust_precision();
    let rotation_speed = 2.5;

    for event in movement_event_reader.read() {
        for (movement_acceleration, jump_impulse, mut linear_velocity, mut transform, is_grounded) in
            &mut controllers
        {
            match event {
                MovementAction::Move(direction) => {
                    // Rotate player based on horizontal input
                    if direction.x != 0.0 {
                        let rotation_amount = direction.x * rotation_speed * delta_time as f32;
                        transform.rotate_y(-rotation_amount);
                    }

                    // Get player's forward and right vectors
                    let forward = transform.forward();
                    let _right = transform.right();

                    // Calculate movement direction relative to player's rotation (inverted)
                    let movement_direction = (forward * -direction.y as f32).as_dvec3().as_vec3();
                    
                    // Apply movement in the direction the player is facing
                    linear_velocity.x += movement_direction.x * movement_acceleration.0 * delta_time;
                    linear_velocity.z += movement_direction.z * movement_acceleration.0 * delta_time;
                }
                MovementAction::Jump => {
                    // Only jump if grounded
                    if is_grounded {
                        linear_velocity.y = jump_impulse.0;
                    }
                }
            }
        }
    }
}

/// Slows down movement in the XZ plane and prevents unwanted vertical movement
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
    for (damping_factor, mut linear_velocity) in &mut query {
        // Dampen horizontal movement
        linear_velocity.x *= damping_factor.0;
        linear_velocity.z *= damping_factor.0;
        
        // Apply gravity if not grounded
        if linear_velocity.y.abs() >= 0.1 { // Apply gravity when not on ground
            linear_velocity.y -= 9.8 * 0.016; // Apply gravity
        }
    }
}

/// Update camera_follow_player_system to strictly follow player rotation
fn camera_follow_player_system(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<FlyCam>, Without<Player>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for mut camera_transform in camera_query.iter_mut() {
            let player_pos = player_transform.translation;
            let camera_distance = 18.0;
            let camera_height = 4.0;
            
            // Get player's forward direction
            let player_forward = player_transform.forward();
            
            // Calculate camera position in front of player (opposite of before)
            let offset = Vec3::new(
                player_forward.x * camera_distance,  // Removed negative sign
                camera_height,
                player_forward.z * camera_distance,  // Removed negative sign
            );
            
            let target_pos = player_pos + offset;
            
            // Smoothly move camera to new position
            camera_transform.translation = camera_transform.translation.lerp(
                target_pos,
                (5.0 * time.delta_secs()).min(1.0),
            );
            
            // Make camera look at player
            camera_transform.look_at(player_pos, Vec3::Y);
        }
    }
} 