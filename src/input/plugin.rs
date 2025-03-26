use bevy::prelude::*;
use super::systems::*;
use crate::game_logic::state::TurnState;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_piece_selection.run_if(in_state(TurnState::PlayerTurn)));
    }
} 