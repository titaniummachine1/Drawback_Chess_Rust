use bevy::prelude::*;

pub const SPRITE_SIZE: f32 = 64.0;

// Game constants
pub const NUM_COLS: usize = 8;
pub const NUM_ROWS: usize = 8;
pub const BOARD_SIZE: f32 = SPRITE_SIZE * NUM_COLS as f32;

// Size of each tile on the chess board
pub const TILE_SIZE: f32 = 80.0;

// Size of the whole board in pixels
pub const BOARD_SIZE_PX: f32 = TILE_SIZE * NUM_COLS as f32;

// Colors for the chess board
pub const WHITE_SQUARE_COLOR: Color = Color::rgb(0.93, 0.79, 0.69); // Light brown
pub const BLACK_SQUARE_COLOR: Color = Color::rgb(0.46, 0.33, 0.28); // Dark brown

// Colors for piece selection and move highlighting
pub const SELECTED_COLOR: Color = Color::rgba(0.0, 0.5, 1.0, 0.5);  // Blue, semi-transparent
pub const LEGAL_MOVE_COLOR: Color = Color::rgba(0.2, 0.8, 0.2, 0.7); // Bright green, more opaque
pub const HOVER_COLOR: Color = Color::rgba(0.0, 0.0, 1.0, 0.3);    // Blue, more transparent

// Z-index constants for proper layering
pub const Z_BOARD: f32 = 0.0;      // Base layer - board squares
pub const Z_HIGHLIGHT: f32 = 0.1;  // Selection highlight
pub const Z_LEGAL_MOVES: f32 = 0.2; // Legal move indicators
pub const Z_PIECES: f32 = 0.3;     // Chess pieces
pub const Z_UI_ELEMENTS: f32 = 0.4; // UI elements like promotion options
pub const Z_DRAGGING: f32 = 0.5;   // Pieces while being dragged

// These colors are defined for potential future use
// pub const ACTIVE_COLOR: Color = Color::rgba(0.0, 1.0, 0.0, 0.7);
// pub const FROM_SQUARE_COLOR: Color = Color::rgba(1.0, 1.0, 0.0, 0.5); 