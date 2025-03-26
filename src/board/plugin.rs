use bevy::prelude::*;
use crate::constants::*;
use super::components::*;
use shakmaty::{Square, File, Rank};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_board);
    }
}

fn setup_board(mut commands: Commands) {
    println!("Setting up chess board...");
    
    // Create board squares
    for y in 0..8 {
        for x in 0..8 {
            // In chess, h1 (bottom right) should be a light square
            // This means (x+y) should be even for light squares
            let is_white = (x + y) % 2 == 0;
            
            // Position the squares in world space
            // (0,0) is at the center, with the board centered on it
            let position = Vec3::new(
                (x as f32 - 3.5) * TILE_SIZE, // Center the board horizontally
                ((7 - y) as f32 - 3.5) * TILE_SIZE, // Flip the y-coordinate so rank 1 is at bottom
                0.0, // Place at z=0 as the background
            );
            
            // Convert x,y coordinates to shakmaty File/Rank
            // With flipped board, File A is still on the left (x=0), but now:
            // y=0 (top row in our drawing) -> Rank::First (White's pieces with flipped board) 
            // y=7 (bottom row in our drawing) -> Rank::Eighth (Black's pieces with flipped board)
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
            
            // Map our grid coordinates to chess rank - FLIPPED BOARD VERSION
            // With flipped board, the rank numbering is reversed
            let rank = match y {
                7 => Rank::Eighth, // BOTTOM row is Rank 8 (black's first rank)
                6 => Rank::Seventh,
                5 => Rank::Sixth,
                4 => Rank::Fifth,
                3 => Rank::Fourth,
                2 => Rank::Third,
                1 => Rank::Second,
                0 => Rank::First, // TOP row is Rank 1 (white's first rank)
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