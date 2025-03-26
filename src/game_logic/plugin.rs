use bevy::prelude::*;
use super::events::{MakeMoveEvent, GameOverEvent};
use super::state::{GameState, TurnState}; // Import GameState
use super::systems::apply_move;

pub struct GameLogicPlugin;

// Standard chess starting position FEN
const STANDARD_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// Initialize game with standard FEN
fn init_game_state(mut commands: Commands) {
    match GameState::from_fen(STANDARD_FEN) {
        Ok(state) => {
            println!("Initialized chess board with standard FEN");
            commands.insert_resource(state);
        },
        Err(err) => {
            println!("Error initializing from FEN: {:?}, using default", err);
            commands.init_resource::<GameState>();
        }
    }
}

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        // --- Resources ---
        // GameState initialized here with defaults
        app.init_resource::<GameState>();

        // --- States ---
        app.init_state::<TurnState>();

        // --- Events ---
        app.add_event::<MakeMoveEvent>();
        app.add_event::<GameOverEvent>();

        // --- Initialize Game State ---
        app.add_systems(Startup, init_game_state);

        // --- Systems ---
        // Apply moves when event occurs
        app.add_systems(Update, apply_move.run_if(on_event::<MakeMoveEvent>()));

        // System to handle GameOverEvent (Placeholder)
        // app.add_systems(OnEnter(TurnState::GameOver), handle_game_over_display);
    }
} 