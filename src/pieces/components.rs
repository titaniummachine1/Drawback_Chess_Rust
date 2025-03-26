use bevy::prelude::*;
use shakmaty::{Square, Role, Color};

/// Represents a chess piece in the Bevy ECS.
#[derive(Component, Debug)]
pub struct Piece {
    pub role: Role,
    pub color: Color,
    pub pos: Square,
} 