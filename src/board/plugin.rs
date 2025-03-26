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
            let is_white = (x + y) % 2 == 0;
            let position = Vec3::new(
                (x as f32 - 3.5) * TILE_SIZE,
                (y as f32 - 3.5) * TILE_SIZE,
                0.0,
            );
            
            // Convert x,y coordinates to shakmaty File/Rank
            // Note: In shakmaty, Rank 1 is at the bottom (y=0), File A is on the left (x=0)
            let file = File::from_index(x).expect("Invalid file index");
            let rank = Rank::from_index(y).expect("Invalid rank index");
            
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