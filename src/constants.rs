use bevy::prelude::*;

pub const SPRITE_SIZE: f32 = 64.0;

// Game constants
pub const NUM_COLS: usize = 8;
// pub const NUM_ROWS: usize = 8; // Commented out since it's unused but might be needed later
pub const BOARD_SIZE: f32 = SPRITE_SIZE * NUM_COLS as f32;

// Size of each tile on the chess board
pub const TILE_SIZE: f32 = 80.0;

// Size of the whole board in pixels
pub const BOARD_SIZE_PX: f32 = TILE_SIZE * NUM_COLS as f32;

// Colors for the chess board
pub const WHITE_SQUARE_COLOR: Color = Color::rgb(0.93, 0.93, 0.83); // Light cream color
pub const BLACK_SQUARE_COLOR: Color = Color::rgb(0.3, 0.4, 0.3); // Dark green color

// Colors for piece selection and move highlighting
pub const SELECTED_COLOR: Color = Color::rgba(1.0, 1.0, 0.0, 0.5);
pub const LEGAL_MOVE_COLOR: Color = Color::rgba(0.0, 1.0, 0.0, 0.7);
pub const HOVER_COLOR: Color = Color::rgba(0.0, 0.0, 1.0, 0.3);

// These colors are defined for potential future use
// pub const ACTIVE_COLOR: Color = Color::rgba(0.0, 1.0, 0.0, 0.7);
// pub const FROM_SQUARE_COLOR: Color = Color::rgba(1.0, 1.0, 0.0, 0.5); 