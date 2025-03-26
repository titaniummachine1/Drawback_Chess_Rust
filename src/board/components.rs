use bevy::prelude::*;

/// Component for a board square
#[derive(Component, Debug)]
pub struct BoardSquare {
    pub x: usize,
    pub y: usize,
    pub is_white: bool,
}

/// Component for visual board square
#[derive(Component, Debug)]
pub struct BoardSquareVisual; 