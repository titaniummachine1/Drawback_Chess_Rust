use bevy::prelude::*;
use super::events::{MakeMoveEvent, GameOverEvent};
use super::state::{GameState, TurnState}; // Keep GameState/TurnState imports
use super::systems::apply_move;

pub struct GameLogicPlugin;

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

        // --- Systems ---
        // Apply moves when event occurs
        app.add_systems(Update, apply_move.run_if(on_event::<MakeMoveEvent>()));

        // System to handle GameOverEvent (Placeholder)
        // app.add_systems(OnEnter(TurnState::GameOver), handle_game_over_display);
    }
} 