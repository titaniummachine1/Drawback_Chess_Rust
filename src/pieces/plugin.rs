use bevy::prelude::*;
use shakmaty::{Color as ChessColor, Role, Square, Position};
use crate::constants::TILE_SIZE;
use crate::game_logic::state::GameState;
use crate::game_logic::events::MakeMoveEvent;
use super::components::Piece;

pub struct PiecesPlugin;

impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_pieces)
           .add_systems(Update, update_piece_positions);
    }
}

/// Update piece positions when moves are made
pub fn update_piece_positions(
    mut commands: Commands,
    mut pieces: Query<(Entity, &mut Piece, &mut Transform)>,
    mut ev_make_move: EventReader<MakeMoveEvent>,
    _game_state: Res<GameState>,
) {
    for ev in ev_make_move.read() {
        let chess_move = &ev.0;
        println!("Updating piece positions for move: {:?}", chess_move);
        
        // Get source and destination squares
        if let Some(from) = chess_move.from() {
            let to = chess_move.to();
            
            // First, remove any captured pieces at the destination
            // Handle captured pieces at the destination
            for (entity, piece, _) in pieces.iter() {
                if piece.pos == to && piece.pos != from {
                    println!("Removing captured piece at {:?}", to);
                    commands.entity(entity).despawn();
                }
            }
            
            // Then find and update the piece we're moving
            let mut found = false;
            for (entity, mut piece, mut transform) in pieces.iter_mut() {
                if piece.pos == from {
                    // Update the piece's square
                    println!("Moving piece from {:?} to {:?}", from, to);
                    piece.pos = to;
                    
                    // Update its visual position
                    let file = to.file().char() as u8 - b'a';
                    let rank = to.rank().char() as u8 - b'1';
                    
                    let new_position = Vec3::new(
                        (file as f32 - 3.5) * TILE_SIZE,
                        ((7 - rank) as f32 - 3.5) * TILE_SIZE,
                        0.1
                    );
                    
                    transform.translation = new_position;
                    found = true;
                    break;
                }
            }
            
            if !found {
                println!("Warning: Couldn't find piece at source square {:?}", from);
            }
        }
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
            // File is horizontal (A-H, left to right)
            let file = square.file().char() as u8 - b'a'; // 0-7 for files a-h
            
            // Rank is vertical (1-8, bottom to top)
            // In chess, rank 1 is at the bottom (white's first rank)
            // and rank 8 is at the top (black's first rank)
            let rank = square.rank().char() as u8 - b'1'; // 0-7 for ranks 1-8
            
            // Debug information about coordinate mapping
            println!(
                "Mapping square {:?}: file={}, rank={}, file_char={}, rank_char={}", 
                square, 
                file, 
                rank, 
                square.file().char(),
                square.rank().char()
            );
            
            // In our screen coordinates:
            // - x increases from left to right (matches chess files)
            // - y increases from bottom to top (matches chess ranks)
            // - (0,0) is the center of the screen
            let position = Vec3::new(
                (file as f32 - 3.5) * TILE_SIZE, // Center the board horizontally
                ((7 - rank) as f32 - 3.5) * TILE_SIZE, // Flip the rank to get 0 at the bottom
                0.1, // Place slightly above board and highlights
            );

            println!("Placing piece at square: {:?}, position: {:?}", square, position);

            // Determine piece image path based on color and role
            let color_prefix = match piece.color {
                ChessColor::White => "b",
                ChessColor::Black => "w",
            };
            
            let role_suffix = match piece.role {
                Role::King => "K",
                Role::Queen => "Q",
                Role::Rook => "R",
                Role::Bishop => "B",
                Role::Knight => "N",
                Role::Pawn => "P",
            };
            
            // Load the piece image
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