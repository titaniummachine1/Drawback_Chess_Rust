use bevy::prelude::*;
use crate::constants::*;
use super::components::*;
use shakmaty::{Square, File, Rank};
use crate::game_logic::state::GameState;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_board)
           .add_systems(Update, handle_board_flip);
    }
}

// System to handle board flipping with the 'R' key
fn handle_board_flip(
    keys: Res<Input<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut board_squares: Query<(&mut Transform, &BoardSquare)>,
    mut pieces: Query<(&mut Transform, &crate::pieces::components::Piece), Without<BoardSquare>>,
) {
    if keys.just_pressed(KeyCode::R) {
        // Toggle the board flipped state
        game_state.board_flipped = !game_state.board_flipped;
        println!("Board flipped: {}", game_state.board_flipped);
        
        // Update all board square positions
        for (mut transform, square) in board_squares.iter_mut() {
            let (x, y) = (square.x, square.y);
            let position = calculate_square_position(x, y, game_state.board_flipped);
            transform.translation = position;
        }
        
        // Update all piece positions
        for (mut transform, piece) in pieces.iter_mut() {
            let file = piece.pos.file().char() as u8 - b'a';
            let rank = piece.pos.rank().char() as u8 - b'1';
            
            let (x, y) = (file as usize, rank as usize);
            let position = calculate_square_position(x, y, game_state.board_flipped);
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            // Keep the z-coordinate (pieces should remain above the board)
            transform.translation.z = Z_PIECES;
        }
    }
}

// Helper function to calculate square positions based on board orientation
fn calculate_square_position(x: usize, y: usize, flipped: bool) -> Vec3 {
    if flipped {
        // Flipped board (white at top)
        Vec3::new(
            (x as f32 - 3.5) * TILE_SIZE,
            (y as f32 - 3.5) * TILE_SIZE,
            0.0
        )
    } else {
        // Standard board (white at bottom)
        Vec3::new(
            (x as f32 - 3.5) * TILE_SIZE,
            ((7 - y) as f32 - 3.5) * TILE_SIZE,
            0.0
        )
    }
}

fn setup_board(mut commands: Commands) {
    println!("Setting up chess board...");
    
    // Create board squares
    for y in 0..8 {
        for x in 0..8 {
            // In standard chess, a1 (bottom left) should be a dark square
            // This means (x+y) should be odd for dark squares
            let is_white = (x + y) % 2 == 1; // Swapped the condition from (x+y)%2==0
            
            // Position the squares in world space
            // (0,0) is at the center, with the board centered on it
            let position = Vec3::new(
                (x as f32 - 3.5) * TILE_SIZE, // Center the board horizontally
                ((7 - y) as f32 - 3.5) * TILE_SIZE, // Standard orientation (white at bottom)
                0.0, // Place at z=0 as the background
            );
            
            // Convert x,y coordinates to shakmaty File/Rank
            let file = match x {
                0 => File::A,
                1 => File::B,
                2 => File::C,
                3 => File::D,
                4 => File::E,
                5 => File::F,
                6 => File::G,
                7 => File::H,
                _ => panic!("Invalid file index"),
            };
            
            // Map our grid coordinates to chess rank - standard orientation
            let rank = match y {
                0 => Rank::First,  // Bottom row is Rank 1 (white's first rank)
                1 => Rank::Second,
                2 => Rank::Third,
                3 => Rank::Fourth,
                4 => Rank::Fifth,
                5 => Rank::Sixth,
                6 => Rank::Seventh,
                7 => Rank::Eighth, // Top row is Rank 8 (black's first rank)
                _ => panic!("Invalid rank index"),
            };
            
            // Create the shakmaty Square
            let square = Square::from_coords(file, rank);
            
            println!("Creating board square: {:?} at position {:?}, color: {}", 
                     square, position, if is_white { "white" } else { "black" });
            
            commands.spawn((
                BoardSquare { 
                    x, 
                    y, 
                    is_white,
                    square,
                },
                BoardSquareVisual,
                SpriteBundle {
                    sprite: Sprite {
                        color: if is_white { WHITE_SQUARE_COLOR } else { BLACK_SQUARE_COLOR },
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_translation(position),
                    ..default()
                },
            ));
        }
    }
} 