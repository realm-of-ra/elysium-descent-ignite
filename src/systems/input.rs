use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode}; // Added MonitorSelection
use bevy_enhanced_input::prelude::*;

use crate::game::components::PlayerInput;
use crate::screens::Screen;
use crate::systems::character_controller::CharacterControllerPlugin;

/// Plugin responsible for handling input in the Elysium game
pub struct ElysiumInputPlugin;

impl Plugin for ElysiumInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<ElysiumInput>()
            .add_input_context::<GameCreation>()
            .add_systems(Startup, setup_input)
            .add_observer(binding)
            .add_observer(handle_toggle_fullscreen)
            .add_observer(handle_return_to_menu)
            .add_observer(handle_jump) // New observer for Jump
            .add_observer(handle_sprint) // New observer for Sprint
            .add_observer(handle_crouch) // New observer for Crouch
            .add_observer(handle_interact) // New observer for Interact
            .add_observer(handle_primary_attack) // New observer for Primary Attack
            .add_observer(handle_inventory) // New observer for Inventory
            .add_observer(pre_gameplay_binding)
            .add_observer(toggle_game_creation)
            .add_plugins(CharacterControllerPlugin); // Register the avian3d character controller plugin
    }
}

/// Input context for the Elysium game
#[derive(InputContext)]
pub struct ElysiumInput;

// --- Core Game Actions ---

/// Action for toggling between fullscreen and windowed mode
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct ToggleFullScreen;

/// Action for returning to the main menu
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct ReturnToMainMenu;

impl ReturnToMainMenu {
    const KEY: KeyCode = KeyCode::Escape;
}

/// Action for movement (WASD, Arrow Keys, Left Stick)
#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Move;

/// Action for jumping
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Jump;

impl Jump {
    const KEY: KeyCode = KeyCode::Space;
}

/// Action for sprinting
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Sprint;

impl Sprint {
    const KEY: KeyCode = KeyCode::ShiftLeft; // Left Shift for sprint
}

/// Action for crouching
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Crouch;

impl Crouch {
    const KEY: KeyCode = KeyCode::ControlRight; // Left Ctrl for crouch
}

/// Action for interacting with objects
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct Interact;

impl Interact {
    const KEY: KeyCode = KeyCode::KeyE; // E for interact
}

/// Action for primary attack/fire
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct PrimaryAttack;

impl PrimaryAttack {
    const KEY: KeyCode = KeyCode::KeyF; // Left Mouse Button for primary attack
}

/// Action for opening inventory
#[derive(Debug, InputAction)]
#[input_action(output = bool)]
struct OpenInventory;

impl OpenInventory {
    const KEY: KeyCode = KeyCode::KeyI; // I for Inventory
    // Could also use KeyCode::Tab, but I is more explicit for 'Inventory'
}

// --- Setup and Binding Systems ---

fn setup_input(mut commands: Commands) {
    commands.spawn((PlayerInput, Actions::<ElysiumInput>::default()));
}

fn binding(
    trigger: Trigger<Binding<ElysiumInput>>,
    mut actions: Query<&mut Actions<ElysiumInput>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();

    // Toggle Fullscreen
    actions
        .bind::<ToggleFullScreen>()
        .to((KeyCode::F11, (KeyCode::AltLeft, KeyCode::Enter)))
        .with_conditions(Press::default());

    // Return to Main Menu
    actions
        .bind::<ReturnToMainMenu>()
        .to(ReturnToMainMenu::KEY)
        .with_conditions(Press::default());

    // Movement (WASD, Arrow Keys, Gamepad Left Stick)
    actions.bind::<Move>().to((
        Cardinal::wasd_keys(),
        Axial::left_stick(),
        Cardinal::arrow_keys(),
    ));

    // Jump (Spacebar)
    actions
        .bind::<Jump>()
        .to(Jump::KEY)
        .with_conditions(Press::default());

    // Sprint (Left Shift)
    actions
        .bind::<Sprint>()
        .to(Sprint::KEY)
        .with_conditions(Hold::new(0.0)); // Use Hold for continuous sprint

    // Crouch (Left Ctrl)
    actions
        .bind::<Crouch>()
        .to(Crouch::KEY)
        .with_conditions(Hold::new(0.0)); // Use Hold for continuous crouch

    // Interact (E Key)
    actions
        .bind::<Interact>()
        .to(Interact::KEY)
        .with_conditions(Hold::new(0.0));

    // Primary Attack (Left Mouse Button)
    actions
        .bind::<PrimaryAttack>()
        .to(PrimaryAttack::KEY)
        .with_conditions(Press::default()); // Or Hold for continuous fire

    // Open Inventory (I Key)
    actions
        .bind::<OpenInventory>()
        .to(OpenInventory::KEY)
        .with_conditions(Press::default());

    // Existing: Toggle Game Creation Mode (N Key)
    // This binding is also present in the GameCreation context, which might be fine if you want to toggle it from both.
    // If 'N' is ONLY for toggling GameCreation, consider if it should only be bound in one context.
    actions.bind::<ToggleGameCreationMode>().to(KeyCode::KeyN);
}

// --- Action Handling Systems ---

fn handle_toggle_fullscreen(
    trigger: Trigger<Started<ToggleFullScreen>>,
    mut windows: Query<&mut Window>,
) {
    if trigger.value {
        if let Ok(mut window) = windows.single_mut() {
            window.mode = match window.mode {
                WindowMode::Windowed => {
                    info!("Switching to fullscreen");
                    WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                }
                _ => {
                    info!("Switching to windowed");
                    WindowMode::Windowed
                }
            };
        } else {
            error!("Failed to get window");
        }
    }
}

fn handle_return_to_menu(
    trigger: Trigger<Started<ReturnToMainMenu>>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    if trigger.value {
        info!("Returning to main menu");
        next_state.set(Screen::MainMenu);
    }
}

// --- New Action Handlers (Placeholders) ---

fn handle_jump(
    trigger: Trigger<Started<Jump>>,
    // mut players: Query<&mut Transform, With<Player>>,
) {
    if trigger.value {
        info!("Player jumped!");
    }
}

fn handle_sprint(trigger: Trigger<Fired<Sprint>>) {
    if trigger.value {
        info!("Player is sprinting...");
    }
}

fn handle_crouch(trigger: Trigger<Fired<Sprint>>) {
    if trigger.value {
        info!("Player is crouching...");
    }
}

fn handle_interact(
    trigger: Trigger<Started<Interact>>,
    // You'd query for nearby interactable objects here
) {
    if trigger.value {
        info!("Player interacted!");
        // Logic to check for nearby interactable objects and activate them
    }
}

fn handle_primary_attack(
    trigger: Trigger<Started<PrimaryAttack>>,
    // Query for player's weapon component or attack logic
) {
    if trigger.value {
        info!("Player attacked!");
        // Logic for shooting, melee attack, etc.
    }
}

fn handle_inventory(
    trigger: Trigger<Started<OpenInventory>>,
    mut next_state: ResMut<NextState<Screen>>,
    current_screen_state: Res<State<Screen>>, // Get current screen state
) {
    if trigger.value {
        info!("Toggling Inventory");
        // Example: Toggle between InGame and Inventory screens
        if *current_screen_state.get() == Screen::GamePlay {
            next_state.set(Screen::Inventory); // Assuming you have an Inventory screen
        } else if *current_screen_state.get() == Screen::Inventory {
            next_state.set(Screen::GamePlay);
        }
    }
}

// --- Game Creation Specific Input ---

#[derive(Debug, InputAction)]
#[input_action(output = bool, require_reset = true)]
struct ToggleGameCreationMode;

#[derive(InputContext)]
pub struct GameCreation;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub struct StartGame;

fn pre_gameplay_binding(
    trigger: Trigger<Binding<GameCreation>>,
    mut actions: Query<&mut Actions<GameCreation>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions.bind::<StartGame>().to(KeyCode::Space);
    actions.bind::<ToggleGameCreationMode>().to(KeyCode::KeyN);
}

fn toggle_game_creation(
    trigger: Trigger<Started<ToggleGameCreationMode>>,
    mut commands: Commands,
    elysium_query: Query<(), With<Actions<ElysiumInput>>>,
    game_creation_query: Query<(), With<Actions<GameCreation>>>,
) {
    let has_elysium_input = elysium_query.contains(trigger.target());
    let has_game_creation = game_creation_query.contains(trigger.target());

    if has_elysium_input {
        info!("Switching to game creation mode");
        commands
            .entity(trigger.target())
            .remove::<Actions<ElysiumInput>>()
            .insert(Actions::<GameCreation>::default());
    } else if has_game_creation {
        info!("Exiting game creation mode");
        commands
            .entity(trigger.target())
            .remove::<Actions<GameCreation>>()
            .insert(Actions::<ElysiumInput>::default());
    } else {
        info!("Initializing default input mode");
        commands
            .entity(trigger.target())
            .insert(Actions::<ElysiumInput>::default());
    }
}
