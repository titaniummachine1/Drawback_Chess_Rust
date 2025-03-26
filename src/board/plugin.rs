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
            // In chess, traditionally white squares are at coordinates where x+y is even
            let is_white = (x + y) % 2 == 0;
            let position = Vec3::new(
                (x as f32 - 3.5) * TILE_SIZE,
                ((7 - y) as f32 - 3.5) * TILE_SIZE, // Flip the y-coordinate to match chess board orientation
                -0.2, // Below the pieces
            );
            
            // Convert x,y coordinates to shakmaty File/Rank
            // In shakmaty, File A is on the left (x=0), Rank 1 is at the bottom (y=0)
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
            
            // When we draw the board, we flip the y-coordinate, so we need to map
            // y=0 (top row in our drawing) to Rank::Eighth
            // y=7 (bottom row in our drawing) to Rank::First
            let rank = match y {
                0 => Rank::Eighth,
                1 => Rank::Seventh,
                2 => Rank::Sixth,
                3 => Rank::Fifth,
                4 => Rank::Fourth,
                5 => Rank::Third,
                6 => Rank::Second,
                7 => Rank::First,
                _ => panic!("Invalid rank index"),
            };
            
            // Create the shakmaty Square
            let square = Square::from_coords(file, rank);
            
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
                        ..Default::default()
                    },
                    transform: Transform::from_translation(position),
                    ..Default::default()
                },
            ));
        }
    }
} 