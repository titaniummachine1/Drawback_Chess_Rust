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
    asset_server: Res<AssetServer>,
    game_state: Res<GameState>,
) {
    println!("Spawning chess pieces...");
    
    // Iterate through all squares on the board
    for square in Square::ALL {
        if let Some(piece) = game_state.board.board().piece_at(square) {
            // Calculate position based on square
            let file = square.file().char() as u8 - b'a';
            let rank = square.rank().char() as u8 - b'1';
            let position = Vec3::new(
                (file as f32 - 3.5) * TILE_SIZE, // Center the board
                (rank as f32 - 3.5) * TILE_SIZE,
                0.0,
            );

            // Determine sprite path based on piece color and role
            let sprite_path = match (piece.color, piece.role) {
                (ChessColor::White, Role::King) => "pieces/white_king.png",
                (ChessColor::White, Role::Queen) => "pieces/white_queen.png",
                (ChessColor::White, Role::Bishop) => "pieces/white_bishop.png",
                (ChessColor::White, Role::Knight) => "pieces/white_knight.png",
                (ChessColor::White, Role::Rook) => "pieces/white_rook.png",
                (ChessColor::White, Role::Pawn) => "pieces/white_pawn.png",
                (ChessColor::Black, Role::King) => "pieces/black_king.png",
                (ChessColor::Black, Role::Queen) => "pieces/black_king.png", // Using king placeholder
                (ChessColor::Black, Role::Bishop) => "pieces/black_king.png", // Using king placeholder
                (ChessColor::Black, Role::Knight) => "pieces/black_king.png", // Using king placeholder
                (ChessColor::Black, Role::Rook) => "pieces/black_king.png", // Using king placeholder
                (ChessColor::Black, Role::Pawn) => "pieces/black_king.png", // Using king placeholder
            };

            // Create a bundle based on whether the image exists or not
            let color = if piece.color == ChessColor::White {
                Color::WHITE
            } else {
                Color::BLACK
            };

            commands.spawn((
                SpriteBundle {
                    texture: asset_server.load(sprite_path),
                    transform: Transform::from_translation(position),
                    sprite: Sprite {
                        color,
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