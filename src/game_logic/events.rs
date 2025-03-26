use bevy::prelude::*;
use shakmaty::{Move, Color as ChessColor};

/// Event triggered to request a move
pub struct MakeMoveEvent(pub Move);

/// Event triggered when the game is over
pub struct GameOverEvent(pub String); // Contains reason for game over (e.g., "Checkmate", "Stalemate")

// Implement Event traits for our custom events
impl Event for MakeMoveEvent {}
impl Event for GameOverEvent {} 