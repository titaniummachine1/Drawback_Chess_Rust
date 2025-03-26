use bevy::prelude::*;
use shakmaty::{Color as ChessColor, Role, Square, Position};
use crate::constants::TILE_SIZE;
use crate::game_logic::state::GameState;
use super::components::Piece;

pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pieces);
    }
}

/// Spawns chess pieces based on the current game state
pub fn spawn_pieces(
    mut commands: Commands,
    game_state: Res<GameState>,
    asset_server: Res<AssetServer>,
) {
    println!("Spawning chess pieces...");
    
    // Iterate through all squares on the board
    for square in Square::ALL {
        if let Some(piece) = game_state.board.board().piece_at(square) {
            // Calculate position based on square
            let file = square.file().char() as u8 - b'a'; // 0-7 for files a-h
            
            // For visualization: convert rank 1-8 to position on screen
            // We need to flip the rank value to get white at the bottom and black at the top
            // Rank 1 (white's first rank) should be at the bottom (y=7 in our board)
            // Rank 8 (black's first rank) should be at the top (y=0 in our board)
            let rank = square.rank().char() as u8 - b'1'; // 0-7 for ranks 1-8
            
            let position = Vec3::new(
                (file as f32 - 3.5) * TILE_SIZE, // Center the board horizontally
                ((7 - rank) as f32 - 3.5) * TILE_SIZE, // Flip the rank to match chess board orientation
                0.1, // Place slightly above board and highlights
            );

            println!("Placing piece at square: {:?}, position: {:?}", square, position);

            // Determine piece image path based on color and role
            let color_prefix = match piece.color {
                ChessColor::White => "w",
                ChessColor::Black => "b",
            };
            
            let role_suffix = match piece.role {
                Role::King => "K",
                Role::Queen => "Q",
                Role::Rook => "R",
                Role::Bishop => "B",
                Role::Knight => "N",
                Role::Pawn => "P",
            };
            
            // Updated path to use assets directory
            let image_path = format!("images/{}{}.png", color_prefix, role_suffix);
            println!("Loading piece image: {}", image_path);
            
            // Spawn piece with image
            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(&image_path),
                    transform: Transform::from_translation(position),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 0.9)),
                        ..default()
                    },
                    ..default()
                },
                Piece {
                    pos: square,
                    color: piece.color,
                    role: piece.role,
                },
            ));
        }
    }
} 