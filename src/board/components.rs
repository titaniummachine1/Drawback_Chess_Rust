use bevy::prelude::*;
use shakmaty::Square;

/// Component for a board square
#[derive(Component, Debug)]
pub struct BoardSquare {
    pub x: usize,
    pub y: usize,
    pub is_white: bool,
    pub square: Square,
}

/// Component for visual board square
#[derive(Component, Debug)]
pub struct BoardSquareVisual; 